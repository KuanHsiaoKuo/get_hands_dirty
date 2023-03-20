use diesel::prelude::*;
use std::env::args;
use diesel_mysql_demo::establish_connection;
use diesel_mysql_demo::models::Post;
use diesel_mysql_demo::schema::posts::dsl::posts;
use diesel_mysql_demo::schema::posts::published;

fn main() {

    let id = args()
        .nth(1)
        .expect("publish_post requires a post id")
        .parse::<i32>()
        .expect("Invalid ID");
    let connection = &mut establish_connection();

    let post = connection
        .transaction(|connection| {
            let post = posts.find(id).select(Post::as_select()).first(connection)?;

            diesel::update(posts.find(id))
                .set(published.eq(true))
                .execute(connection)?;
            Ok(post)
        })
        .unwrap_or_else(|_: diesel::result::Error| panic!("Unable to find post {}", id));

    println!("Published post {}", post.title);
}