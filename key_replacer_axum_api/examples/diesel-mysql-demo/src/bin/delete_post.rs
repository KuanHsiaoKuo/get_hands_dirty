use diesel::prelude::*;
use std::env::args;
use diesel_mysql_demo::establish_connection;
use diesel_mysql_demo::schema::posts::dsl::posts;
use diesel_mysql_demo::schema::posts::title;

fn main() {

    let target = args().nth(1).expect("Expected a target title to match against");
    let pattern = format!("%{target}%");

    let connection = &mut establish_connection();

    let num_deleted = diesel::delete(posts.filter(title.like(pattern)))
        .execute(connection)
        .expect("Error deleting posts");

    println!("Deleted {num_deleted} posts");
}
