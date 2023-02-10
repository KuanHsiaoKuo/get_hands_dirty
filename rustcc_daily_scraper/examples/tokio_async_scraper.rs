use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use select::{document::Document, predicate::Attr};

async fn get_page(url: &str, client: &Client) -> Result<Document, reqwest::Error> {
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
    let res = client.get(url).headers(headers).send().await?;
    println!("url: {}", &res.url().as_str());
    let body = res.text().await?;
    println!("get res body: \n{}", body.as_str());
    Ok(Document::from(body.as_str()))
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let base_url = "https://rustcc.cn/section?id=f4703117-7e6b-4caf-aa22-a3ad3db6898f";

    let document = get_page(base_url, &client).await?;

    let title_class = "title left";
    // let exist_elements = document.find(Class(title_class));
    let exist_elements = document.find(Attr("class", title_class));
    // println!("get nodes length: {}", exist_elements.clone().count());
    if exist_elements.count() == 0 {
        println!("No elements found with `{}`", title_class);
    } else {
        println!("found");
        for node in document.find(Attr("class", title_class)) {
            let title = node.text();
            let link = node.attr("href").unwrap_or("");
            println!("Title: {}\nLink: {}\n", title, link);
        }
    }

    Ok(())
}