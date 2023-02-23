use html2md::parse_html;
use regex::Regex;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use select::{document::Document, predicate::Attr};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
struct DailyItem {
    pub title: String,
    // 日报标题
    pub url: String,
    // url 地址
    pub publish_date: String,    // 日报日期
}

async fn get_page(target_url: &str, client: &Client) -> Result<Document, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert("Accept", HeaderValue::from_str("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9").unwrap());
    headers.insert("Accept-Language", HeaderValue::from_str("en").unwrap());
    headers.insert("Cache-Control", HeaderValue::from_str("max-age=0").unwrap());
    headers.insert("Connection", HeaderValue::from_str("keep-alive").unwrap());
    headers.insert("Cookie", HeaderValue::from_str("Hm_lvt_1fd834970f3ad2bab2cb57d4aa2b2e5a=1675840155; Hm_lpvt_1fd834970f3ad2bab2cb57d4aa2b2e5a=1675953471").unwrap());
    headers.insert("Host", HeaderValue::from_str("rustcc.cn").unwrap());
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_str("document").unwrap());
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_str("navigate").unwrap());
    headers.insert("Sec-Fetch-Site", HeaderValue::from_str("none").unwrap());
    headers.insert("Sec-Fetch-User", HeaderValue::from_str("?1").unwrap());
    headers.insert("Upgrade-Insecure-Requests", HeaderValue::from_str("1").unwrap());
    headers.insert("User-Agent", HeaderValue::from_str("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.0.0.0 Safari/537.36").unwrap());
    headers.insert("sec-ch-ua", HeaderValue::from_str("\"Not_A Brand\";v=\"99\", \"Google Chrome\";v=\"109\", \"Chromium\";v=\"109\"").unwrap());
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_str("?0").unwrap());
    headers.insert("sec-ch-ua-platform", HeaderValue::from_str("\"macOS\"").unwrap());
    let res = client.get(target_url).headers(headers).send().await?;
    // println!("url: {}", &res.url().as_str());
    let body = res.text().await?;
    // println!("get res body: \n{}", body.as_str());
    Ok(Document::from(body.as_str()))
}

fn kv_pair_to_query_string(params: Vec<(String, String)>) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("&")
}

fn get_publish_date(title: &str) -> String {
    let date_re = Regex::new(r"(\d{4}-\d{2}-\d{2})").unwrap();
    let publish_date = match date_re.captures(title) {
        Some(captured) => captured.get(1).unwrap().as_str().to_string(), // 这里unwrap()之后只有as_str()方法, 没有to_string()
        None => format!("Unable to extract date from {}", title)
    };
    publish_date
}

async fn page_extractor<'a>(document: Document, page: i32) -> Option<Vec<DailyItem>> {
    let title_class = "title left";
    let exist_elements = document.find(Attr("class", title_class));
    let exist_ele_vec = document.find(Attr("class", title_class)).collect::<Vec<_>>();
    let exist_count = exist_ele_vec.len();
    if exist_count == 0 {
        println!("No elements found with `{}` in page {}", title_class, page);
        None
    } else {
        let mut exist_nodes = Vec::new();
        for node in exist_elements {
            let title = node.text();
            let url = node.attr("href").unwrap_or("");
            let publish_date = get_publish_date(title.as_str());
            // println!("Title: {}\nLink: {}\n", title, url);
            let node_item = DailyItem { title, url: url.to_string(), publish_date: publish_date.to_string() };
            // println!("node_item: {}\n", serde_json::to_string(&node_item).unwrap());
            exist_nodes.push(node_item);
        }
        Some(exist_nodes)
    }
}

async fn page_content_extractor(node_url: &str, client: &Client) -> Option<Vec<String>> {
    let content_document = get_page(node_url, client).await.unwrap();
    let content_class = "detail-body";
    let exist_elements = content_document.find(Attr("class", content_class));
    let exist_ele_vec = content_document.find(Attr("class", content_class)).collect::<Vec<_>>();
    let exist_count = exist_ele_vec.len();
    if exist_count == 0 {
        println!("No elements found with `{}` in page {}", content_class, node_url);
        None
    } else {
        let mut exist_nodes = Vec::new();
        for node in exist_elements {
            let content = node.text();
            let md_content = parse_html(content.as_str());
            println!("node_url: {}\nContent: {}\n", node_url, md_content);
            exist_nodes.push(content);
        }
        Some(exist_nodes)
    }
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
        let document = get_page(daily_section_page_url.as_str(), &client).await.unwrap();
        let exist_elements = page_extractor(document, page.clone()).await.unwrap();
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