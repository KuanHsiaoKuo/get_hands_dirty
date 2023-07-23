use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{Mutex, mpsc};
use tokio::time::{sleep, Duration};

use html2md::parse_html;
use rbatis::Rbatis;
use reqwest::Client;
use select::document::Document;
use select::node::Node;
use select::predicate::{Name, Predicate};
use futures;

use scraper_collections::{
    extract_nodes, get_custom_headers, get_page, get_publish_date, kv_pair_to_query_string,
    rustcc_daily_models::{DailyPageItem, PageContentItem}, split_rustcc_daily_content, async_time_it};
use scraper_collections::rustcc_daily_models::{DbDailyPage, DbPageContent, init_rustcc_db};

const BASIC_URL: &str = "https://rustcc.cn";
const NUM_PRODUCERS: usize = 1;
const NUM_CONSUMERS: usize = 5;
const TOTAL_PAGE: usize = 5;
// 对于tokio的 mpsc channel，其实并没有直接的API能够获得当前队列中元素的数量。
// 一个可能的解决方案是引入一个全局的原子计数器来追踪生产者添加到队列中的元素数量和消费者从队列中移除的元素数量。
// 使用std::sync::atomic::AtomicUsize来实现这个功能
static QUEUE_SIZE: AtomicUsize = AtomicUsize::new(0); // 用于查看队列大小

async fn page_extractor(page_url: &str, client: &Client) -> Option<Vec<DailyPageItem>> {
    fn page_node_process(nodes: Vec<Node>) -> Vec<DailyPageItem> {
        let mut exist_nodes = Vec::new();
        for node in nodes {
            let title = node.text();
            let url = format!("{}{}", BASIC_URL, node.attr("href").unwrap_or(""));
            let publish_date = get_publish_date(title.as_str());
            let node_item = DailyPageItem { title, url: url.to_string(), publish_date: publish_date.to_string() };
            exist_nodes.push(node_item);
        }
        exist_nodes
    }
    let processed_nodes = extract_nodes(
        client, page_url,
        "class", "title left",
        page_node_process).await.unwrap();
    Some(processed_nodes)
}

async fn page_content_extractor(page: &DailyPageItem, client: &Client) -> Option<Vec<PageContentItem>> {
    let content_class = "detail-body "; // 注意后面的空格
    fn content_node_process(nodes: Vec<Node>) -> Vec<HashMap<&'static str, String>> {
        let mut exist_nodes = Vec::new();
        for node in nodes {
            let mut item_collect: Vec<String> = vec![];
            let mut need_collect = false;
            for dec in node.descendants() {
                if dec.html().starts_with("<h") && item_collect.len() > 0 {
                    let temp = item_collect.split_first().unwrap().1.join("\n");
                    // println!("{}", temp);
                    let mut content_item = HashMap::new();
                    content_item.insert("title", parse_html(item_collect.first().unwrap()));
                    content_item.insert("md_content", parse_html(temp.as_str()));
                    exist_nodes.push(content_item);
                    item_collect.clear();
                }
                let html_dec = dec.html();
                let start_flags = vec!["\n\t".to_string(), "\n".to_string()];
                if start_flags.contains(&html_dec) {
                    if !need_collect {
                        need_collect = true;
                        continue;
                    }
                }
                if need_collect {
                    item_collect.push(html_dec);
                    need_collect = false;
                }
            }
        }
        exist_nodes
    }
    let processed_nodes = extract_nodes(
        client,
        page.url.as_str(),
        "class",
        content_class,
        content_node_process).await.unwrap();
    let mut page_content_nodes = vec![];
    for node in processed_nodes {
        page_content_nodes.push(
            PageContentItem {
                title: node.get("title").unwrap().to_string(),
                md_content: node.get("md_content").unwrap().to_string(),
                publish_page: page.clone(),
            }
        );
    }
    Some(page_content_nodes)
}

async fn fast_log_init() {
    // 全局日志，只能初始化一个
    fast_log::init(
        fast_log::Config::new()
            // .console()
            .file("logs/test.log")
            .level(log::LevelFilter::Info),
    ).expect("rbatis fast_log init fail");
}

async fn rbatis_init() -> Rbatis {
    DbDailyPage::crud_methods_init();
    DbPageContent::crud_methods_init();
    let mut rb = init_rustcc_db().await;
    rb
}

async fn producer(
    queue: Arc<Mutex<mpsc::Sender<DailyPageItem>>>,
    id: i32, start_page: usize, pages: usize, section_id: &str,
    rb: Arc<Mutex<Rbatis>>) {
    let client = Client::new();
    for page in start_page..start_page + pages { // 使用 pages 而不是硬编码的值
        let page_params = vec![
            ("current_page".to_string(), page.to_string()),
            ("id".to_string(), section_id.to_string()),
        ];
        let query_string = kv_pair_to_query_string(page_params);
        let daily_section_page_url = format!("{}/section?{}", BASIC_URL, query_string);

        let exist_dailys = page_extractor(daily_section_page_url.as_str(), &client).await.unwrap();
        for daily in exist_dailys {
            queue.lock().await.send(daily).await.unwrap();
            // 实时查看当前队列大小
            let current_size = QUEUE_SIZE.fetch_add(1, Ordering::Relaxed);
            println!("Added item to queue. Current size: {}", current_size + 1);
        }
        sleep(Duration::from_secs(1)).await;  // 一秒钟生成一次任务
    }
}

async fn consumer(queue: Arc<Mutex<mpsc::Receiver<DailyPageItem>>>, id: i32, rb: Arc<Mutex<Rbatis>>) {
    let client = Client::new();
    while let Some(daily) = queue.lock().await.recv().await {
        // 实时查看当前队列大小
        let current_size = QUEUE_SIZE.fetch_sub(1, Ordering::Relaxed);
        println!("Removed item from queue. Current size: {}", current_size - 1);

        let task_client = client.clone();
        let daily_rb = Arc::clone(&rb);
        tokio::spawn(async move {
            let mut task_rb = daily_rb.lock().await;  // lock the mutex to access Rbatis
            let page_content = page_content_extractor(&daily, &task_client).await.unwrap();
            for (index, content_node) in page_content.iter().enumerate() {
                let new_content = DbPageContent::new(content_node, &mut task_rb).await;
                let content_inserted_id = new_content.insert_or_exists_id(&mut task_rb).await.unwrap();
                println!("{index}.{content_inserted_id}\n\n{}", serde_json::to_string(&new_content).unwrap());
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    fast_log_init().await;
    // 使用异步测量宏
    async_time_it! {
        async{ // 使用了 async 关键字来创建了一个异步的代码块。然后将这个异步的代码块传给 async_time_it 宏，这个宏会等待这个异步代码块完成，并且计算和打印出它的执行时间。
            let pages_per_producer = TOTAL_PAGE / NUM_PRODUCERS;
            let mut extra_pages = TOTAL_PAGE % NUM_PRODUCERS;
            let section_id = "f4703117-7e6b-4caf-aa22-a3ad3db6898f";
            let rb = Arc::new(Mutex::new(rbatis_init().await));
            // mpsc::channel 创建的是一个通道，然后我们将发送端（sender）传递给生产者，接收端（receiver）传递给消费者。
            let (tx, rx) = mpsc::channel(100);
            let producer_queue = Arc::new(Mutex::new(tx));
            let consumer_queue = Arc::new(Mutex::new(rx));

            let mut producer_tasks = Vec::new();
            for id in 1..NUM_PRODUCERS+1 {
                let start_page = id * pages_per_producer;
                let pages = if extra_pages > 0 {
                    extra_pages -= 1;
                    pages_per_producer + 1
                } else {
                    pages_per_producer
                };
                let producer_task = tokio::spawn(producer(Arc::clone(&producer_queue), id as i32, start_page, pages, section_id, Arc::clone(&rb)));
                producer_tasks.push(producer_task);

            }

            let mut consumer_tasks = Vec::new();
            for id in 0..NUM_CONSUMERS {
                let consumer_task = tokio::spawn(consumer(Arc::clone(&consumer_queue), id as i32, Arc::clone(&rb)));
                consumer_tasks.push(consumer_task);
            }

            let producer_results = futures::future::join_all(producer_tasks).await;
            let consumer_results = futures::future::join_all(consumer_tasks).await;

            // check if any of the tasks have resulted in an error
            for res in producer_results {
                if let Err(e) = res {
                    eprintln!("A producer task returned an error: {:?}", e);
                }
            }

            for res in consumer_results {
                if let Err(e) = res {
                    eprintln!("A consumer task returned an error: {:?}", e);
                }
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use scraper_collections::aw;

    use super::*;

// use crate::aw;

    #[test]
    fn test_page_content_extract() {
        let node_url = "https://rustcc.cn/article?id=c97d7a42-d014-4493-b582-82e016921a50";
        let client = Client::new();
        let page_content = aw!(page_content_extractor(node_url, &client)).unwrap();
        println!("get_node_content: {}\n", serde_json::to_string(&page_content[0]).unwrap());
    }
}