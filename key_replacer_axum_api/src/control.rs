use diesel::prelude::*;
use diesel::dsl::sql_query;
use diesel::sql_types::Text;
// use diesel_async::AsyncMysqlConnection;
use diesel_async::{RunQueryDsl, AsyncConnection, AsyncMysqlConnection};

#[derive(QueryableByName, PartialEq, Debug)]
struct Char {
    // #[sql_type = "Text"]
    #[diesel(sql_type = Text)]
    dr_id: String,
    // #[sql_type = "Text"]
    #[diesel(sql_type = Text)]
    name: String,
}

#[derive(QueryableByName, PartialEq, Debug)]
struct News{
    // #[sql_type = "Text"]
    #[diesel(sql_type = Text)]
    news_id: String,
    // #[sql_type = "Text"]
    #[diesel(sql_type = Text)]
    content: String,
}

async fn query_char_news(conn: &mut AsyncMysqlConnection) -> QueryResult<()> {
    let char_sql = "SELECT dr_id, `name` from `char` LIMIT 10";
    let news_sql = "SELECT news_id, content from news limit 10";
    // let sql = "select * from char limit 10";
    let chars = sql_query(char_sql).load::<Char>(conn).await?;
    let news = sql_query(news_sql).load::<News>(conn).await?;

    for char in chars {
        println!("Name: {}", char.name);
    }

    for new in news {
        println!("Content: {}", new.content);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use dotenvy::dotenv;
    use super::*;
    use crate::aw;

    #[test]
    fn test_query_char_news() {
        dotenv().ok();
        let mut async_conn = aw!(AsyncMysqlConnection::establish(std::env::var("QUERY_DATABASE_URL").unwrap().as_str())).unwrap();
        let result = aw!(query_char_news(&mut async_conn));
        result.unwrap()
    }

    #[tokio::test]
    async fn test_async_query_char_news() {
        dotenv().ok();
        let mut async_conn = AsyncMysqlConnection::establish(std::env::var("QUERY_DATABASE_URL").unwrap().as_str()).await.unwrap();
        let result = query_char_news(&mut async_conn).await;
        result.unwrap()
    }
}