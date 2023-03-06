use html2md::parse_html;
use reqwest::Client;
use select::node::Node;

use scraper_collections::{DailyItem, extract_nodes, get_custom_headers, get_page, get_publish_date, kv_pair_to_query_string};

async fn page_extractor(page_url: &str, client: &Client) -> Option<Vec<DailyItem>> {
    // let page_document = get_page(page_url, client).await.unwrap();
    fn page_node_process(nodes: Vec<Node>) -> Vec<DailyItem> {
        let mut exist_nodes = Vec::new();
        for node in nodes {
            let title = node.text();
            let url = node.attr("href").unwrap_or("");
            let publish_date = get_publish_date(title.as_str());
            // println!("Title: {}\nLink: {}\n", title, url);
            let node_item = DailyItem { title, url: url.to_string(), publish_date: publish_date.to_string() };
            // println!("node_item: {}\n", serde_json::to_string(&node_item).unwrap());
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

async fn page_content_extractor(node_url: &str, client: &Client) -> Option<Vec<String>> {
    // let content_document = get_page(node_url, client).await.unwrap();
    let content_class = "detail-body "; // 注意后面的空格
    fn content_node_process(nodes: Vec<Node>) -> Vec<String> {
        let mut exist_nodes = Vec::new();
        for node in nodes {
            let content = node.text();
            let md_content = parse_html(content.as_str());
            // println!("node_url: {}\nContent: {}\n", node_url, md_content);
            exist_nodes.push(md_content);
        }
        exist_nodes
    }
    let processed_nodes = extract_nodes(
        client,
        node_url,
        "class",
        content_class,
        content_node_process).await.unwrap();
    Some(processed_nodes)
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let basic_url = "https://rustcc.cn";
    let section_id = "f4703117-7e6b-4caf-aa22-a3ad3db6898f";
    let mut page = 1;
    while page < 60 {
        let page_params = vec![
            ("current_page".to_string(), page.to_string()),
            ("id".to_string(), section_id.to_string()),
        ];
        let query_string = kv_pair_to_query_string(page_params);
        // println!("{}", query_string);
        let daily_section_page_url = format!("{}/section?{}", basic_url, query_string);
        // println!("{}", daily_section_page_url);
        let exist_elements = page_extractor(daily_section_page_url.as_str(), &client).await.unwrap();
        for node in exist_elements {
            println!("get_node_item: {}\n", serde_json::to_string(&node).unwrap());
            let daily_url = format!("{}{}", basic_url, node.url.as_str());
            let page_content = page_content_extractor(daily_url.as_str(), &client).await.unwrap();
            // println!("get_node_content: {}\n", page_content[0]);
        }
        page += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // for async fn test
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test_get_publish_date() {
        let title = "【Rust日报】2023-02-22 ";
        let publish_date = get_publish_date(title);
        assert_eq!("2023-02-22", publish_date);
    }

    #[test]
    fn test_page_content_extract() {
        let node_url = "https://rustcc.cn/article?id=c97d7a42-d014-4493-b582-82e016921a50";
        let client = Client::new();
        let page_content = aw!(page_content_extractor(node_url, &client)).unwrap();
        println!("get_node_content: {}\n", page_content[0]);
    }
}