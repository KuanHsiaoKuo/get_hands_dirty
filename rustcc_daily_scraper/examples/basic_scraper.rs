use reqwest::Client;
use scraper::{Html, Selector};

fn main() -> Result<()> {

    // Create a new client
    let client = Client::new();

    // Send a GET request to the website
    let mut res = client.get("http://books.toscrape.com/").send().await;

    // Extract the HTML from the response
    let body = res.text().unwrap();

    // Parse the HTML into a document
    let document = Html::parse_document(&body);

    // Create a selector for the book titles
    let book_title_selector = Selector::parse("h3 > a").unwrap();

    // Iterate over the book titles
    for book_title in document.select(&book_title_selector) {
        let title = book_title.text().collect::<Vec<_>>();
        println!("Title: {}", title[0]);
    }

    // Create a selector for the book prices
    let book_price_selector = Selector::parse(".price_color").unwrap();

// Iterate over the book prices
    for book_price in document.select(&book_price_selector) {
        let price = book_price.text().collect::<Vec<_>>();
        println!("Price: {}", price[0]);
    }
    Ok(())
}
