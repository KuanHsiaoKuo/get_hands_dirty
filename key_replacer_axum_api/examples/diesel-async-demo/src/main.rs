use diesel::dsl::sql_query;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};

use diesel_async::{AsyncConnection, AsyncMysqlConnection, RunQueryDsl};

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

pub async fn query_char_news(conn: &mut AsyncMysqlConnection) -> QueryResult<()> {
    let char_sql = "SELECT id, `name` from `char` LIMIT 10";
    let news_sql = "SELECT id, news_id, title, content from news limit 10";
    // let sql = "select * from char limit 10";
    let chars = sql_query(char_sql).load::<QueryChar>(conn).await?;
    let news = sql_query(news_sql).load::<QueryNews>(conn).await?;

    for char in chars {
        println!("Name: {}", char.name);
    }

    for new in news {
        println!("Content: {}", new.content);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let mut async_conn = AsyncMysqlConnection::establish(std::env::var("QUERY_DATABASE_URL").unwrap().as_str()).await.unwrap();
    query_char_news(&mut async_conn).await.expect("TODO: panic message");
}
