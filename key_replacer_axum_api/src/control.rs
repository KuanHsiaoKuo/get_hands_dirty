use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use deadpool::managed::{Object, Pool};
use diesel::dsl::sql_query;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
// use diesel_async::AsyncMysqlConnection;
use diesel_async::{AsyncConnection, AsyncMysqlConnection, RunQueryDsl};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use dotenvy::dotenv;
use tracing::log;

use crate::models::{NewPost, Post};
use crate::schema::posts;

#[derive(QueryableByName, PartialEq, Debug)]
pub struct QueryChar {
    // #[sql_type = "Text"]
    #[diesel(sql_type = Integer)]
    id: i32,
    // #[sql_type = "Text"]
    #[diesel(sql_type = Text)]
    name: String,
}

#[derive(QueryableByName, PartialEq, Debug)]
pub struct QueryNews {
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

type ConnectionPoolOne = Object<AsyncDieselConnectionManager<AsyncMysqlConnection>>;
// type ConnectionPool = Pool<AsyncDieselConnectionManager<AsyncMysqlConnection>, Object<AsyncDieselConnectionManager<AsyncMysqlConnection>>>;
type ConnectionPool = Pool<AsyncDieselConnectionManager<AsyncMysqlConnection>, ConnectionPoolOne>;

pub async fn query_char_news(
    conn: &mut ConnectionPoolOne
) -> QueryResult<()> {
    let char_sql = "SELECT dr_id, `name` from `char` LIMIT 10";
    let news_sql = "SELECT news_id, content from news limit 10";
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

pub async fn query_chunk_chars(
    conn: &mut ConnectionPoolOne,
    limit: usize, offset: usize) -> QueryResult<HashMap<i32, String>> {
    // http://<ip>:<port>/observe/people/<id>
    let all_char_sql = format!("SELECT id, name from `char` LIMIT {} OFFSET {}", limit, offset);
    let chars = sql_query(all_char_sql).load::<QueryChar>(conn).await?;

    // let mut char_map = HashMap::new();
    // for char in chars{
    //     char_map.insert(char.id, char.name);
    // }
    let char_map = chars.into_iter().map(|char| (char.id, char.name)).collect();

    Ok(char_map)
}

pub async fn query_chunk_posts(
    conn: &mut ConnectionPoolOne,
    limit: usize, offset: usize) -> QueryResult<Vec<QueryNews>> {
    // http://<ip>:<port>/observe/people/<id>
    let all_news_sql = format!("SELECT id, news_id, title, content from `news` LIMIT {} OFFSET {}", limit, offset);
    let query_news = sql_query(all_news_sql).load::<QueryNews>(conn).await?;

    // let news_map = query_news.into_iter().map(|item| (item.news_id, item.content)).collect();

    // Ok(news_map)
    Ok(query_news)
}

// Function to compare news and chars
pub async fn char_replaced_post(exist_chars: &HashMap<i32, String>, content: String) -> String {
    let mut replaced_content = content.clone();
    for (_, char_name) in exist_chars.iter() {
        if replaced_content.contains(char_name) {
            replaced_content = replaced_content.replace(char_name, &format!("<a>{}</a>", char_name));
        }
    }
    replaced_content
}

// pub async fn total_update_process(conn: &mut AsyncMysqlConnection, chunk_size: usize) -> QueryResult<()> {
//     let all_chars = query_chunk_chars(conn, 10, 0).await?.into_iter().collect();
//     let mut offset = 0;
//     loop {
//         let chunk_news = query_chunk_posts(conn, chunk_size, offset).await?;
//         if chunk_news.is_empty() {
//             break;
//         }
//         let chunk_chars = all_chars.clone();
//         let mut insert_conn = AsyncMysqlConnection::establish(std::env::var("DATABASE_URL").unwrap().as_str()).await.unwrap();
//         tokio::spawn(async move {
//             compare_news_chars_insert(&chunk_chars, &chunk_news, &mut insert_conn).await;
//         });
//         offset += chunk_size;
//     }
//     Ok(())
// }

pub async fn total_update_process_with_arc_rw(
    query_conn_pool: &mut ConnectionPool,
    insert_conn_pool: &mut ConnectionPool,
    chunk_size: usize) -> QueryResult<()> {
    /**
    In this solution, all_chars is wrapped in an Arc and RwLock to allow multiple threads to access it.
    The Arc allows multiple threads to share ownership of the HashMap, and the RwLock allows multiple
    threads to read the HashMap at the same time.

    In the closure of tokio::spawn, we first clone the Arc using Arc::clone, which creates a new Arc
    that points to the same HashMap. Then we call read on the RwLock to get a read lock on the HashMap.
    The read lock allows multiple threads to read the HashMap at the same time. Finally, we pass the read
    lock to compare_news_chars to compare the news and characters.
    **/
    log::info!("Start total_update_process_with_arc_rw");
    // let mut query_conn = query_conn_pool.get().await.unwrap();
    let all_chars = Arc::new(RwLock::new(
        query_chunk_chars(&mut query_conn_pool.get().await.unwrap(), 10, 0).await?.into_iter().collect()));
    let mut offset = 0;
    loop {
        let chunk_posts = query_chunk_posts(&mut query_conn_pool.get().await.unwrap(), chunk_size, offset).await?;
        if chunk_posts.is_empty() {
            break;
        }
        let chunk_chars = Arc::clone(&all_chars);
        let read_lock: RwLockReadGuard<HashMap<i32, String>> = chunk_chars.read().unwrap(); // the compiler need infer type to HashMap<i32, String>
        let read_lock_clone = read_lock.clone(); // the async move block need
        let mut insert_conn = insert_conn_pool.get().await.unwrap(); // the async move block need
        tokio::spawn(async move {
            log::info!("compare_news_chars start");
            for post in chunk_posts {
                let replaced_content = char_replaced_post(&read_lock_clone, post.content).await;
                transaction_insert_post(&mut insert_conn, &post.id, post.title.as_str(), replaced_content.as_str());
            }
            log::info!("compare_news_chars end");
        });
        offset += chunk_size;
    }
    log::info!("End total_update_process_with_arc_rw");
    Ok(())
}

pub async fn insert_post(conn: &mut AsyncMysqlConnection, id: &i32, title: &str, content: &str) -> Post {
    let new_post = NewPost { id, title, content };
    diesel::insert_into(posts::table)
        .values(&new_post)
        .execute(conn).await.unwrap();
    posts::table
        .order(posts::id.desc())
        .select(Post::as_select())
        .first(conn).await.unwrap()
}

pub async fn transaction_insert_post(
    insert_conn: &mut ConnectionPoolOne,
    id: &i32, title: &str, content: &str) -> Post {
    let new_post = NewPost { id, title, content };

    insert_conn.transaction::<_, diesel::result::Error, _>(|insert_conn| {
        Box::pin(
            async move {
                diesel::insert_into(posts::table)
                    .values(&new_post)
                    .execute(insert_conn).await?;

                let created_post = posts::table
                    .order(posts::id.desc())
                    .select(Post::as_select())
                    .first(insert_conn).await?;

                Ok(created_post)
            }
        )
    }).await.unwrap()
}


#[cfg(test)]
mod tests {
    use std::io::stdin;
    use std::time::Duration;

    use dotenvy::dotenv;

    use crate::aw;

    use super::*;

// #[test]
    // fn test_query_char_news() {
    //     dotenv().ok();
    //     let mut query_conn = aw!(AsyncMysqlConnection::establish(std::env::var("QUERY_DATABASE_URL").unwrap().as_str())).unwrap();
    //     let result = aw!(query_char_news(&mut query_conn));
    //     result.unwrap()
    // }

    #[tokio::test]
    async fn test_query_chunk_chars() {
        dotenv().ok();
        let database_url = std::env::var("QUERY_DATABASE_URL").unwrap();
        let config = AsyncDieselConnectionManager::<AsyncMysqlConnection>::new(database_url);
        // 这里要主动声明用自定义类型别名，否则rust不会自己去联系。
        let query_pool: ConnectionPool = Pool::builder(config).build().unwrap();
        let mut query_conn = query_pool.get().await.unwrap();
        let result = query_chunk_chars(&mut query_conn, 2, 0).await.unwrap();

        for (id, name) in result {
            println!("{}: {}", id, name);
        }
    }

    #[tokio::test]
    async fn test_transaction_insert_post() {
        dotenv().ok();
        let database_url = std::env::var("INSERT_DATABASE_URL").unwrap();
        let config = AsyncDieselConnectionManager::<AsyncMysqlConnection>::new(database_url);
        let pool: ConnectionPool = Pool::builder(config).build().unwrap();
        let mut conn = pool.get().await.unwrap();
        let id = 11;
        let title = "test title";
        let content = "test content";
        let result = transaction_insert_post(&mut conn, &id, title, content).await;
        assert_eq!(result.title, title);
        assert_eq!(result.content, content);
    }


    #[tokio::test]
    async fn test_total_update_process_with_arc_rw() {
        dotenv().ok();
        let query_database_url = std::env::var("QUERY_DATABASE_URL").unwrap();
        let insert_database_url = std::env::var("INSERT_DATABASE_URL").unwrap();
        let query_config = AsyncDieselConnectionManager::<AsyncMysqlConnection>::new(query_database_url);
        let insert_config = AsyncDieselConnectionManager::<AsyncMysqlConnection>::new(insert_database_url);
        let query_pool: ConnectionPool = Pool::builder(query_config).build().unwrap();
        let insert_pool: ConnectionPool = Pool::builder(insert_config).build().unwrap();
        let result = total_update_process_with_arc_rw(&mut query_pool.clone(), &mut insert_pool.clone(), 2).await;
        result.unwrap();
    }
}