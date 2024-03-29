use std::collections::HashMap;
use std::ops::Deref;

use html2md::parse_html;
use rbatis::Rbatis;
use reqwest::Client;
use select::document::Document;
use select::node::Node;
use select::predicate::{Name, Predicate};
use tokio::io::split;

use scraper_collections::{
    extract_nodes, get_custom_headers, get_page,
    get_publish_date, kv_pair_to_query_string, rustcc_daily_models::{DailyPageItem, PageContentItem},
    split_rustcc_daily_content, time_it
};
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

/*
1. 虽然使用了 tokio::main 宏，这表示 main 函数运行在 Tokio 运行时环境中，可以使用 Rust 的 async/await 语法来进行异步编程。
2. 代码中虽然使用了异步函数（如 page_extractor, page_content_extractor, DbPageContent::new, new_content.insert_or_exists_id 等），但都是使用 .await 立即等待这些异步操作完成。这种方式被称为"串行异步"，即在一个异步操作完成之前，其他的异步操作不会开始。
3. 尽管在一个异步函数中，但由于并没有创建新的任务，所以所有操作都是顺序执行的。
4. 对每一个页面的处理必须等待前一个页面处理完毕，每一个内容的处理也必须等待前一个内容处理完毕。这样会导致程序花费大量的时间等待 IO（网络请求和数据库查询），而 CPU 大部分时间都闲置。
 */
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let section_id = "f4703117-7e6b-4caf-aa22-a3ad3db6898f";
    let mut page = 1;
    let mut rb = rbatis_init().await;
    while page < 5 {
        let page_params = vec![
            ("current_page".to_string(), page.to_string()),
            ("id".to_string(), section_id.to_string()),
        ];
        let query_string = kv_pair_to_query_string(page_params);
        let daily_section_page_url = format!("{}/section?{}", BASIC_URL, query_string);
        let exist_dailys = page_extractor(daily_section_page_url.as_str(), &client).await.unwrap();
        for daily in exist_dailys {
            // println!("get_node_item: {}\n", serde_json::to_string(&node).unwrap());

            let page_content = page_content_extractor(&daily, &client).await.unwrap();
            for (index, content_node) in page_content.iter().enumerate() {
                // split_rustcc_daily_content(content)
                // println!("{index}.{}\n{}", daily.url, serde_json::to_string(&page_content).unwrap());
                let new_content = DbPageContent::new(content_node, &mut rb).await;
                let content_inserted_id = new_content.insert_or_exists_id(&mut rb).await.unwrap();
                println!("{index}.{content_inserted_id}\n\n{}", serde_json::to_string(&new_content).unwrap());
            }
        }
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