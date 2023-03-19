use diesel::prelude::*;
use diesel_mysql_demo::establish_connection;
use diesel_mysql_demo::models::Post;
use diesel_mysql_demo::schema::posts::dsl::*;
use diesel_mysql_demo::schema::posts::published;

fn main() {

    let connection = &mut establish_connection();
    let results = posts
        .filter(published.eq(true))
        .limit(5)
        .select(Post::as_select())
        .load(connection)
        .expect("Error loading posts");

    println!("Displaying {} posts", results.len());
    for post in results {
        println!("{}", post.title);
        println!("-----------\n");
        println!("{}", post.body);
    }
}
