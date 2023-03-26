use deadpool_diesel::mysql::{Runtime, Manager, Pool, Object};
use diesel::{prelude::*, select, sql_query, sql_types::{Text, Integer}};
use dotenvy::dotenv;

#[derive(QueryableByName, PartialEq, Debug)]
struct QueryChar {
    // #[sql_type = "Text"]
    #[diesel(sql_type = Integer)]
    id: i32,
    // #[sql_type = "Text"]
    #[diesel(sql_type = Text)]
    name: String,
}

#[derive(QueryableByName, PartialEq, Debug)]
struct QueryNews {
    // #[sql_type = "Text"]
    #[diesel(sql_type = Integer)]
    id: i32,
    #[diesel(sql_type = Text)]
    news_id: String,
    #[diesel(sql_type = Text)]
    title: String,
    // #[sql_type = "Text"]
    #[diesel(sql_type = Text)]
    content: String,
}


pub async fn query_char_news(conn: Object) -> Result<(), Box<dyn std::error::Error>> {
    let news_sql = "SELECT news_id, content from news limit 10";
    // let sql = "select * from char limit 10";
    // let chars = sql_query(char_sql).load::<QueryChar>(pool.get().await?).await?;
    // let news = sql_query(news_sql).load::<QueryNews>(pool.get().await?).await?;
    let chars = conn.interact(|conn|{
        let char_sql = "SELECT id, `name` from `char` LIMIT 10";
        sql_query(char_sql).load::<QueryChar>(conn)
    }).await??;

    for char in chars {
        println!("Name: {}", char.name);
    }

    // for new in news {
    //     println!("Content: {}", new.content);
    // }

    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = std::env::var("QUERY_DATABASE_URL").unwrap();
    let manager = Manager::new(database_url, Runtime::Tokio1);
    let pool = Pool::builder(manager)
        .max_size(8)
        .build()
        .unwrap();
    let conn = pool.get().await?;
    // let result = conn.interact(|conn| {
    //     let query = select("Hello world!".into_sql::<Text>());
    //     query.get_result::<String>(conn)
    // }).await??;
    // assert!(result == "Hello world!");
    // add await after query_char_news(conn); in the main function to wait for the function to complete and print the output.
    query_char_news(conn).await.expect("TODO: panic message");
    Ok(())
}