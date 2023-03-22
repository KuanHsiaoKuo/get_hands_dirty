// use crate::schema::char;
// use diesel::prelude::*;
//
//
// #[derive(Queryable, Selectable)]
// #[diesel(table_name = char)]
// // #[diesel(check_for_backend(diesel::mysql::Mysql))]
// pub struct Char {
//     pub id: i32,
//     pub title: String,
//     pub content: String,
//     pub published: bool,
// }
//
// #[derive(Insertable)]
// #[diesel(table_name = char)]
// pub struct NewChar<'a> {
//     pub title: &'a str,
//     pub content: &'a str,
// }

use diesel::prelude::*;

use crate::schema::posts;

#[derive(Queryable, Selectable)]
#[diesel(table_name = posts)]
// #[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub published: bool,
}

#[derive(Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost<'a> {
    pub title: &'a str,
    pub content: &'a str,
}
