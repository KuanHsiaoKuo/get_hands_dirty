use diesel::{prelude::*, select, sql_query, sql_types::{Text, Integer}};
use diesel_async::{
    pooled_connection::{AsyncDieselConnectionManager, deadpool::Pool},
    AsyncMysqlConnection,
    RunQueryDsl
    };
use deadpool::managed::Object;


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


pub async fn query_char_news(conn: &mut Object<AsyncDieselConnectionManager<AsyncMysqlConnection>>) -> Result<(), Box<dyn std::error::Error>> {
    let news_sql = "SELECT news_id, content from news limit 10";
    let char_sql = "SELECT id, `name` from `char` LIMIT 10";
    // let sql = "select * from char limit 10";
    // let chars = sql_query(char_sql).load::<QueryChar>(pool.get().await?).await?;
    // let news = sql_query(news_sql).load::<QueryNews>(pool.get().await?).await?;
    let chars = sql_query(char_sql).load::<QueryChar>(conn).await?;

    for char in chars {
        println!("Name: {}", char.name);
    }

    // for new in news {
    //     println!("Content: {}", new.content);
    // }

    Ok(())
}


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("QUERY_DATABASE_URL").unwrap();
    let config = AsyncDieselConnectionManager::<AsyncMysqlConnection>::new(database_url);
    let pool = Pool::builder(config).build().unwrap();
    let mut conn = pool.get().await.unwrap();
    // let res = users::table.select(User::as_select()).load::(&mut conn).await.unwrap();
    query_char_news(&mut conn).await.expect("TODO: panic message");

}