use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use diesel::dsl::sql_query;
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
// use diesel_async::AsyncMysqlConnection;
use diesel_async::{AsyncConnection, AsyncMysqlConnection, RunQueryDsl};

use crate::models::{NewPost, Post};
use crate::schema::posts;

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
    #[diesel(sql_type = Text)]
    news_id: String,
    // #[sql_type = "Text"]
    #[diesel(sql_type = Text)]
    content: String,
}

pub async fn query_char_news(conn: &mut AsyncMysqlConnection) -> QueryResult<()> {
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

pub async fn query_all_chars(conn: &mut AsyncMysqlConnection) -> QueryResult<HashMap<i32, String>> {
    // http://<ip>:<port>/observe/people/<id>
    let all_char_sql = "SELECT id, name from `char`";
    let chars = sql_query(all_char_sql).load::<QueryChar>(conn).await?;

    // let mut char_map = HashMap::new();
    // for char in chars{
    //     char_map.insert(char.id, char.name);
    // }
    let char_map = chars.into_iter().map(|char| (char.id, char.name)).collect();

    Ok(char_map)
}

pub async fn query_news_chunk(conn: &mut AsyncMysqlConnection, limit: usize, offset: usize) -> QueryResult<HashMap<String, String>> {
    // http://<ip>:<port>/observe/people/<id>
    let all_news_sql = format!("SELECT news_id, content from `news` LIMIT {} OFFSET {}", limit, offset);
    let query_news = sql_query(all_news_sql).load::<QueryNews>(conn).await?;

    let news_map = query_news.into_iter().map(|item| (item.news_id, item.content)).collect();

    Ok(news_map)
}

// Function to compare news and chars
pub fn compare_news_chars(exist_chars: &HashMap<i32, String>, exist_news: &HashMap<String, String>) {
    exist_news.iter().for_each(|(_, news_content)| {
        exist_chars.iter().for_each(|(_, char_name)| {
            if news_content.contains(char_name) {
                println!("Character '{}' found in news: {}", char_name, news_content);
            }
        });
    });
}

pub async fn total_update_process(conn: &mut AsyncMysqlConnection, chunk_size: usize) -> QueryResult<()> {
    let all_chars = query_all_chars(conn).await?;
    let mut offset = 0;
    loop {
        let chunk_news = query_news_chunk(conn, chunk_size, offset).await?;
        if chunk_news.is_empty() {
            break;
        }
        let chunk_chars = all_chars.clone();
        tokio::spawn(async move {
            compare_news_chars(&chunk_chars, &chunk_news);
        });
        offset += chunk_size;
    }
    Ok(())
}

pub async fn total_update_process_with_arc_rw(conn: &mut AsyncMysqlConnection, chunk_size: usize) -> QueryResult<()> {
    /**
    In this solution, all_chars is wrapped in an Arc and RwLock to allow multiple threads to access it.
    The Arc allows multiple threads to share ownership of the HashMap, and the RwLock allows multiple
    threads to read the HashMap at the same time.

    In the closure of tokio::spawn, we first clone the Arc using Arc::clone, which creates a new Arc
    that points to the same HashMap. Then we call read on the RwLock to get a read lock on the HashMap.
    The read lock allows multiple threads to read the HashMap at the same time. Finally, we pass the read
    lock to compare_news_chars to compare the news and characters.
    **/
    let all_chars = Arc::new(RwLock::new(query_all_chars(conn).await?));
    let mut offset = 0;
    loop {
        let chunk_news = query_news_chunk(conn, chunk_size, offset).await?;
        if chunk_news.is_empty() {
            break;
        }
        let chunk_chars = Arc::clone(&all_chars);
        tokio::spawn(async move {
            let read_lock = chunk_chars.read().unwrap();
            compare_news_chars(&read_lock, &chunk_news);
        });
        offset += chunk_size;
    }
    Ok(())
}


pub async fn create_char(conn: &mut AsyncMysqlConnection, title: &str, content: &str) -> Post {
    let new_post = NewPost { title, content };
    diesel::insert_into(posts::table)
        .values(&new_post)
        .execute(conn).await.unwrap();
    posts::table
        .order(posts::id.desc())
        .select(Post::as_select())
        .first(conn).await.unwrap()
}

pub async fn transaction_create_char(conn: &mut AsyncMysqlConnection, title: &str, content: &str) -> Post {
    let new_post = NewPost { title, content };

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        Box::pin(
            async move {
                diesel::insert_into(posts::table)
                    .values(&new_post)
                    .execute(conn).await?;

                let created_post = posts::table
                    .order(posts::id.desc())
                    .select(Post::as_select())
                    .first(conn).await?;

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


    #[tokio::test]
    async fn test_create_char() {
        dotenv().ok();
        let mut async_conn = AsyncMysqlConnection::establish(std::env::var("DATABASE_URL").unwrap().as_str()).await.unwrap();

        // let mut title = String::new();
        // let mut content = String::new();
        //
        // println!("What would you like your title to be?");
        // stdin().read_line(&mut title).unwrap();
        // let title = title.trim_end(); // Remove the trailing newline

        // println!("\nOk! Let's write {title} (Press {EOF} when finished)\n",);
        // stdin().read_to_string(&mut body).unwrap();
        // stdin().read_line(&mut content).unwrap();

        // let new_char = create_char(&mut async_conn, title, &content);
        let title = "title";
        let content = "content";
        let new_char = create_char(&mut async_conn, title, content).await;
        println!("\nSaved draft {title} with id {}", new_char.id);
    }

    #[tokio::test]
    async fn test_transaction_create_char() {
        dotenv().ok();
        let mut async_conn = AsyncMysqlConnection::establish(std::env::var("DATABASE_URL").unwrap().as_str()).await.unwrap();
        let title = "title";
        let content = "content";
        let new_char = transaction_create_char(&mut async_conn, title, content).await;
        println!("\nSaved draft {title} with id {}", new_char.id);
    }

    #[tokio::test]
    async fn test_query_all_chars() {
        dotenv().ok();
        let mut async_conn = AsyncMysqlConnection::establish(std::env::var("QUERY_DATABASE_URL").unwrap().as_str()).await.unwrap();
        let result = query_all_chars(&mut async_conn).await.unwrap();

        for (id, name) in result {
            println!("{}: {}", id, name);
        }
    }

    #[tokio::test]
    async fn test_compare_news_chars() {
        let exist_chars = HashMap::from([(1, String::from("Alice")), (2, String::from("Bob"))]);
        let exist_news = HashMap::from([(String::from("news1"), String::from("Alice and Bob are friends.")), (String::from("news2"), String::from("Alice moved to a new city."))]);

        compare_news_chars(&exist_chars, &exist_news);
    }

    #[tokio::test]
    async fn test_unit_compare_news_chars() {
        dotenv().ok();
        let mut async_conn = AsyncMysqlConnection::establish(std::env::var("QUERY_DATABASE_URL").unwrap().as_str()).await.unwrap();
        let exist_chars = query_all_chars(&mut async_conn).await.unwrap();
        let exist_news = query_news_chunk(&mut async_conn, 2, 0).await.unwrap();

        compare_news_chars(&exist_chars, &exist_news);
    }

    #[tokio::test]
    async fn test_unit_total_update_process() {
        dotenv().ok();
        let mut async_conn = AsyncMysqlConnection::establish(std::env::var("QUERY_DATABASE_URL").unwrap().as_str()).await.unwrap();
        let chunk_size = 2;
        let result = total_update_process_with_arc_rw(&mut async_conn, chunk_size).await;
        println!("Result: {:?}", result);
        // wait to see the println result
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}