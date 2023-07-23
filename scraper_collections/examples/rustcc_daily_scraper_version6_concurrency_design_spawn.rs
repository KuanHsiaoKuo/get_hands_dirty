use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::Mutex;

use html2md::parse_html;
use rbatis::Rbatis;
use reqwest::Client;
use select::document::Document;
use select::node::Node;
use select::predicate::{Name, Predicate};
use tokio::io::split;
use futures;

use scraper_collections::{
    extract_nodes, get_custom_headers, get_page, get_publish_date, kv_pair_to_query_string,
    rustcc_daily_models::{DailyPageItem, PageContentItem}, split_rustcc_daily_content, async_time_it};
use scraper_collections::rustcc_daily_models::{DbDailyPage, DbPageContent, init_rustcc_db};

const BASIC_URL: &str = "https://rustcc.cn";

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

async fn rbatis_init() -> Rbatis {
    fast_log::init(
        fast_log::Config::new()
            .console()
            .level(log::LevelFilter::Debug),
    )
        .expect("rbatis init fail");
    DbDailyPage::crud_methods_init();
    DbPageContent::crud_methods_init();
    let mut rb = init_rustcc_db().await;
    rb
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let section_id = "f4703117-7e6b-4caf-aa22-a3ad3db6898f";
    let mut page = 1;
    // 将 rbatis 的初始化移出循环, 避免连接数过多，而且Rbatis 对象可以跨多个线程共享
    // let mut rb = rbatis_init().await;
    let rb = Arc::new(Mutex::new(rbatis_init().await));
    while page < 5 {
        let client = Client::new();
        // 给页面使用
        // let page_rb = Arc::clone(&rb);  // clone the Arc, not the Rbatis
        let page_params = vec![
            ("current_page".to_string(), page.to_string()),
            ("id".to_string(), section_id.to_string()),
        ];
        let query_string = kv_pair_to_query_string(page_params);
        let daily_section_page_url = format!("{}/section?{}", BASIC_URL, query_string);

        // 将每页的任务包装成一个多线程并发任务
        // 每页并发里面是一个个异步处理
        let task = async_time_it!(tokio::spawn({

            // 在 tokio::spawn 调用之前克隆了 rb。这样，rb 就不会在 async move 块中被移动，你就可以在 while 循环的每次迭代中使用新的 rb 实例
            let page_rb = Arc::clone(&rb);  // clone the Arc, not the Rbatis
            async move {
                let exist_dailys = page_extractor(daily_section_page_url.as_str(), &client).await.unwrap();
                let mut content_tasks = vec![];
                for daily in exist_dailys {
                    // 在每个任务中克隆 `client` 和 `rb`, 因为会`async move`
                    // let mut task_rb = rb.clone();
                    // 给每个`daily`使用
                    let daily_rb = Arc::clone(&page_rb);
                    let task_client = client.clone();
                    let content_task = tokio::spawn(async move {
                        let mut task_rb = daily_rb.lock().await;  // lock the mutex to access Rbatis
                        let page_content = page_content_extractor(&daily, &task_client).await.unwrap();
                        for (index, content_node) in page_content.iter().enumerate() {
                            let new_content = DbPageContent::new(content_node, &mut task_rb).await;
                            let content_inserted_id = new_content.insert_or_exists_id(&mut task_rb).await.unwrap();
                            println!("{index}.{content_inserted_id}\n\n{}", serde_json::to_string(&new_content).unwrap());
                        }
                    });
                    content_tasks.push(content_task);
                }
                futures::future::join_all(content_tasks).await;
            }
        }));

        page += 1;
    }
    Ok(())
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