mod control;

use std::env;

use deadpool_diesel::{Manager, Pool, Runtime};
use diesel::prelude::*;
use dotenvy::dotenv;

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn establish_query_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("QUERY_DATABASE_URL").expect("QUERY_DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

// pub async fn establish_query_connection_pool() -> Pool<Manager<_>, _> {
//     let manager = Manager::new(":memory:", Runtime::Tokio1);
//     let pool = Pool::builder(manager).max_size(8).build().unwrap();
//     pool
// }

// for async fn test
#[macro_export]
macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }