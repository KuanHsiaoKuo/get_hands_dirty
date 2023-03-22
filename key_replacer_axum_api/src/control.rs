use diesel::prelude::*;
use diesel::dsl::sql_query;
use diesel::sql_types::Text;
// use diesel_async::AsyncMysqlConnection;
use diesel_async::{RunQueryDsl, AsyncConnection, AsyncMysqlConnection};
use crate::models::{Post, NewPost};
use crate::schema::posts;

#[derive(QueryableByName, PartialEq, Debug)]
struct QueryChar {
    // #[sql_type = "Text"]
    #[diesel(sql_type = Text)]
    dr_id: String,
    // #[sql_type = "Text"]
    #[diesel(sql_type = Text)]
    name: String,
}

#[derive(QueryableByName, PartialEq, Debug)]
struct QueryNews{
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

pub async fn create_char(conn: &mut AsyncMysqlConnection, title: &str, content: &str) -> Post {
    let new_post = NewPost { title, content };

    // conn.transaction(|conn| {
    //     diesel::insert_into(posts::table)
    //         .values(&new_post)
    //         .execute(conn)?;
    //
    //     char::table
    //         .order(posts::id.desc())
    //         .select(Post::as_select())
    //         .first(conn)
    // })
    //     .expect("Error while saving post")
    diesel::insert_into(posts::table)
        .values(&new_post)
        .execute(conn).await.unwrap();
    posts::table
        .order(posts::id.desc())
        .select(Post::as_select())
        .first(conn).await.unwrap()
}


#[cfg(test)]
mod tests {
    use std::io::stdin;
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


    #[tokio::test]
    async fn test_create_char(){
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
}