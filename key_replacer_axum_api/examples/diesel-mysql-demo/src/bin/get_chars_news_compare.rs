use diesel::prelude::*;
use diesel::dsl::sql_query;
use diesel::sql_types::Text;
use diesel_mysql_demo::establish_query_connection;

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

fn main() -> QueryResult<()> {
    let conn = &mut establish_query_connection();

    let char_sql = "SELECT dr_id, `name` from `char` LIMIT 10";
    let news_sql = "SELECT news_id, content from news limit 10";
    // let sql = "select * from char limit 10";
    let chars = sql_query(char_sql).load::<Char>(conn)?;
    let news = sql_query(news_sql).load::<News>(conn)?;

    for char in chars {
        println!("Name: {}", char.name);
    }

    for new in news {
        println!("Content: {}", new.content);
    }

    Ok(())
}