use regex::Regex;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use select::{document::Document, document::Find, predicate::Attr};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
struct DailyItem {
    pub title: String,
    // 日报标题
    pub url: String,
    // url 地址
    pub publish_date: String,    // 日报日期
}

async fn get_page(url: &str, client: &Client, page: &dyn ToString) -> Result<Document, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert("Accept", HeaderValue::from_str("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9").unwrap());
    // headers.insert("Accept-Encoding", HeaderValue::from_str("gzip, deflate, br").unwrap());
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
    // let res = client.get(url).send().await?;
    // let page_params = [("current_page", page.to_string().as_str())];
    // let page_params = vec![("current_page", page.to_string().as_str())];
    let page_params = vec![("current_page".to_string(), page.to_string())];
    // let query_string = encode(page_params.iter().map(|(k, v)| (k.to_owned(), v.to_string())));
    let query_string = kv_pair_to_query_string(page_params);
    println!("{}", query_string);
    // let page_url = format!("{}?{}", url, reqwest::get_params(page_params));
    let res = client.get(url).headers(headers).send().await?;
    println!("url: {}", &res.url().as_str());
    let body = res.text().await?;
    println!("get res body: \n{}", body.as_str());
    Ok(Document::from(body.as_str()))
}

// fn kv_pair_to_query_string(params: Vec<(&str, &str)>) -> String {
fn kv_pair_to_query_string(params: Vec<(String, String)>) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("&")
}

fn get_publish_date(title: &str) -> String{
    let date_re = Regex::new(r"(\d{4}-\d{2}-\d{2})").unwrap();
    // let publish_date = date_re.captures(title.as_str()).unwrap().get(1).unwrap().as_str();
    // ----------------------------------------------------^^^^^^ here to match.
    let publish_date = match date_re.captures(title) {
        Some(captured) => captured.get(1).unwrap().as_str().to_string(), // 这里unwrap()之后只有as_str()方法, 没有to_string()
        // None => format!("Unable to extract date from {}", title).as_str() // temporary value is freed at the end of this statement
        // None => "Unable to extract date from {title}" // str is equal to &'static str ?
        None => format!("Unable to extract date from {}", title)
    };
    publish_date
}

async fn page_extractor<'a>(document: Document, page: i32) -> Option<Vec<DailyItem>>{
    let title_class = "title left";
    let exist_elements = document.find(Attr("class", title_class));
    let exist_ele_vec = document.find(Attr("class", title_class)).collect::<Vec<_>>();
    let exist_count = exist_ele_vec.len();
    if exist_count == 0 {
        // if exist_elements.clone().count() == 0 {
        println!("No elements found with `{}` in page {}", title_class, page);
        None
    } else {
        let mut exist_nodes = Vec::new();
        // println!("found {exist_count} elements in page {page}");
        // Some(exist_elements)
        for node in exist_elements {
            let title = node.text();
            let url = node.attr("href").unwrap_or("");
            let publish_date = get_publish_date(title.as_str());
            println!("Title: {}\nLink: {}\n", title, url);
            let node_item = DailyItem { title, url: url.to_string(), publish_date: publish_date.to_string() };
            println!("node_item: {}\n", serde_json::to_string(&node_item).unwrap());
            exist_nodes.push(node_item);
        }
        Some(exist_nodes)
    }
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let base_url = "https://rustcc.cn/section?id=f4703117-7e6b-4caf-aa22-a3ad3db6898f";
    let mut page = 1;
    while page < 60 {
        let document = get_page(base_url, &client, &page).await.unwrap();
        let exist_elements = page_extractor(document, page.clone()).await.unwrap();
        for node in exist_elements{
            println!("get_node_item: {}\n", serde_json::to_string(&node).unwrap());
        }
        page += 1;
    }

    Ok(())
}