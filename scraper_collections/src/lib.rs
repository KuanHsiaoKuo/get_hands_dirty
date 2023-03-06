use std::borrow::Cow;
use regex::Regex;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use select::{document::Document, document::Find, predicate::Attr};
use select::node::Node;
use select::selection::Selection;
use serde::{Deserialize, Serialize};

pub fn kv_pair_to_query_string(params: Vec<(String, String)>) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("&")
}

pub fn kv_map_to_query_string(params: Vec<(&str, &str)>) -> String {
    todo!()
}

pub fn get_custom_headers() -> HeaderMap {
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
    headers
}

pub fn get_publish_date(title: &str) -> String {
    let date_re = Regex::new(r"(\d{4}-\d{2}-\d{2})").unwrap();
    return match date_re.captures(title) {
        Some(captured) => captured.get(1).unwrap().as_str().to_string(), // 这里unwrap()之后只有as_str()方法, 没有to_string()
        None => format!("Unable to extract date from {}", title)
    };
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DailyItem {
    pub title: String,
    // 日报标题
    pub url: String,
    // url 地址
    pub publish_date: String,    // 日报日期
}

pub async fn get_page(target_url: &str, client: &Client) -> Result<Document, reqwest::Error> {
    let res = client.get(target_url).headers(get_custom_headers()).send().await?;
    // println!("url: {}", &res.url().as_str());
    let body = res.text().await?;
    // println!("get res body: \n{}", body.as_str());
    Ok(Document::from(body.as_str()))
}

// pub fn extract_nodes(document: Document, class_desc: &str) -> Option<Find<Attr<&str, &str>>> {
//     let exist_elements = document.find(Attr("class", class_desc));
//     return match document.find(Attr("class", class_desc)).collect::<Vec<_>>().len() {
//         exist_count if exist_count == 0 => {
//             println!("No elements found with `{}`", class_desc);
//             None
//         },
//         exist_count if exist_count > 0 => Some(exist_elements),
//         _ => None
//     };
// }
// pub fn extract_nodes<'a>(document: Document, class_desc: &str) -> Option<Find<Attr<String, String>>> {
//     let exist_elements = document.find(Attr("class".to_string(), class_desc.to_string()));
//     return match document.find(Attr("class", class_desc)).collect::<Vec<_>>().len() {
//         exist_count if exist_count == 0 => {
//             println!("No elements found with `{}`", class_desc);
//             None
//         },
//         exist_count if exist_count > 0 => Some(exist_elements),
//         _ => None
//     };
// }
// pub fn extract_nodes(document: Document, class_desc: &str) -> Option<Cow<Selection>> {
//     let exist_elements = document.find(Attr("class", class_desc)).into_selection();
//     return match document.find(Attr("class", class_desc)).collect::<Vec<_>>().len() {
//         exist_count if exist_count == 0 => {
//             println!("No elements found with `{}`", class_desc);
//             None
//         },
//         exist_count if exist_count > 0 => Some(Cow::Owned(exist_elements)),
//         _ => None
//     };
// }
// pub fn extract_nodes(document: Document, class_desc: &str) -> Option<Cow<Find<Attr<&str, &str>>>> {
//     let exist_elements = document.find(Attr("class", class_desc));
//     return match document.find(Attr("class", class_desc)).collect::<Vec<_>>().len() {
//         exist_count if exist_count == 0 => {
//             println!("No elements found with `{}`", class_desc);
//             None
//         },
//         exist_count if exist_count > 0 => Some(Cow::Borrowed(exist_elements.as_ref())),
//         _ => None
//     };
// }
pub async fn extract_nodes<F, N>(
    client: &Client,
    target_url: &str,
    attr_name: &str,
    attr_desc: &str,
    node_process: F) -> Option<Vec<N>>
    where F: Fn(Vec<Node>) -> Vec<N>
{
    let document = get_page(target_url, client).await.unwrap();
    let exist_elements = document.find(Attr(attr_name, attr_desc)).collect::<Vec<_>>();
    match exist_elements.len() {
        exist_count if exist_count == 0 => {
            println!("No elements found with `{attr_desc}` in `{attr_name}`");
            None
        },
        exist_count if exist_count > 0 => {
            Some(node_process(exist_elements))
        },
        _ => None
    }
}