use html2md::parse_html;
use reqwest::Client;
// use scraper::Node::Document;
use select::document::Document;
use select::node::Node;
use select::predicate::{Name, Predicate};
use tokio::io::split;

use scraper_collections::{
    aw, DailyPageItem, PageContentItem, extract_nodes,
    get_custom_headers, get_page, get_publish_date,
    kv_pair_to_query_string, split_rustcc_daily_content,
};

async fn page_extractor(page_url: &str, client: &Client) -> Option<Vec<DailyPageItem>> {
    fn page_node_process(nodes: Vec<Node>) -> Vec<DailyPageItem> {
        let mut exist_nodes = Vec::new();
        for node in nodes {
            let title = node.text();
            let url = node.attr("href").unwrap_or("");
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

async fn page_content_extractor(node_url: &str, client: &Client) -> Option<Vec<PageContentItem>> {
    let content_class = "detail-body "; // 注意后面的空格
    fn content_node_process(nodes: Vec<Node>) -> Vec<PageContentItem> {
        let mut exist_nodes = Vec::new();
        for node in nodes {
            // let content = node.text();
            // for item in node.descendant(Name("h3")) {
            //     println!("{}", item.text())
            // }
            let content = node.html();
            // let node_doc = Document::from(content.as_str());
            let mut item_collect: Vec<String> = vec![];
            for dec in node.descendants() {
                // match dec.html().starts_with("<h") {
                //     start if start => item_collect.push(dec.html()),
                //     start if !start => {
                //         println!("{}", dec.html());
                //     }
                // }
                if dec.html().starts_with("<h") {
                    let temp = item_collect.split_first().unwrap().1.join("\n");
                    // println!("{}", temp);
                    let md_temp = parse_html(temp.as_str());
                    let content_item = PageContentItem{
                        title: parse_html(item_collect.first().unwrap()),
                        md_content: md_temp,
                        publish_page: Default::default(),
                    };
                    exist_nodes.push(content_item);
                    item_collect.clear();
                }
                item_collect.push(dec.html());
            }
            // for item in node.find(Name("h3")){
            //     println!("{}", item.text())
            // }
            // let md_content = parse_html(content.as_str());
            // println!("Content: {md_content}");
            // exist_nodes.push(md_content);
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
        let daily_section_page_url = format!("{}/section?{}", basic_url, query_string);
        let exist_elements = page_extractor(daily_section_page_url.as_str(), &client).await.unwrap();
        for node in exist_elements {
            // println!("get_node_item: {}\n", serde_json::to_string(&node).unwrap());
            let daily_url = format!("{}{}", basic_url, node.url.as_str());
            let page_content = page_content_extractor(daily_url.as_str(), &client).await.unwrap();
            for (index, content_node) in page_content.iter().enumerate() {
                // split_rustcc_daily_content(content)
                // content_node.publish_page = node.clone();
                println!("{index}.{daily_url}\n{}", serde_json::to_string(&page_content).unwrap())
            }
        }
        page += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_content_extract() {
        let node_url = "https://rustcc.cn/article?id=c97d7a42-d014-4493-b582-82e016921a50";
        let client = Client::new();
        let page_content = aw!(page_content_extractor(node_url, &client)).unwrap();
        println!("get_node_content: {}\n", page_content[0]);
    }
}