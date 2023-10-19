use log::LevelFilter;
use rbatis::{crud, impl_delete, impl_select, impl_select_page, impl_update, Rbatis};
use rbatis::dark_std::defer;
use rbatis::rbdc::db::ExecResult;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::aw;

#[derive(serde::Serialize, Deserialize, Debug, Default, Clone)]
pub struct DailyPageItem {
    // 日报标题
    pub title: String,
    // url 地址
    pub url: String,
    pub publish_date: String,    // 日报日期
}


#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PageContentItem {
    // 标题
    pub title: String,
    // 具体内容
    pub md_content: String,
    // 关联页面
    pub publish_page: DailyPageItem,
}

/// example table
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbDailyPage {
    pub id: Option<i32>,
    // 日报标题
    pub title: Option<String>,
    // url 地址
    pub url: Option<String>,
    pub publish_date: Option<String>,    // 日报日期
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbPageContent {
    pub id: Option<i32>,
    // 标题
    pub title: Option<String>,
    // 具体内容
    pub md_content: Option<String>,
    // 关联页面
    pub publish_page: Option<i32>,
}

// trait RbatisOperations {
//     fn new<T>(item: T) -> Self;
// }

impl DbDailyPage {
    fn new(item: DailyPageItem) -> Self {
        DbDailyPage {
            id: None,
            title: Some(item.title),
            url: Some(item.url),
            publish_date: Some(item.publish_date),
        }
    }

    async fn insert_or_exists_id(&self, rb: &mut Rbatis) -> Option<i32>{
        let exist = DbDailyPage::select_by_url(rb, "daily_page", self.url.as_ref().unwrap().as_str()).await;
        match exist {
            Ok(v) => match v {
                Some(v) => { // v is DbDailyPage type
                    // println!("query_results: {v:?}");
                    v.id
                },
                None => {
                    // println!("not exists");
                    let data = DbDailyPage::insert(rb, &self).await;
                    match data {
                        Ok(new_v) => { // v is ExecResult type
                            println!("insert_result: {new_v:?}");
                            Some(new_v.last_insert_id.as_u64().unwrap() as i32)
                        },
                        Err(e) => {
                            println!("error inserting: {e:?}");
                            None
                        }
                    }
                }
            }
            Err(e) => {
                println!("error querying: {e:?}");
                None
            }
        }
    }

    pub fn crud_methods_init() {
        //crud!(BizActivity {},"biz_activity");//custom table name
//impl_select!(BizActivity{select_all_by_id(table_name:&str,id:&str) => "`where id = #{id}`"}); //custom table name
        crud!(DbDailyPage {}, "daily_page");
        //         crud!(DbDailyPage {}, "RustCC_Daily");
        impl_select!(DbDailyPage{select_all_by_id(id:&str,title:&str) => "`where id = #{id} and title = #{title}`"});
        impl_select!(DbDailyPage{select_by_id(id:&str) -> Option => "`where id = #{id} limit 1`"});
        impl_select!(DbDailyPage{select_by_url(table_name:&str, url:&str) -> Option => "`where url = #{url} limit 1`"});
        // impl_select!(DbDailyPage{select_by_url(table_name:&str, url:&str) -> Option => "`where url = #{url}`"});
        impl_update!(DbDailyPage{update_by_title(title:&str) => "`where title = '#{title}'`"});
        impl_delete!(DbDailyPage{delete_by_title(title:&str) => "`where name= '#{title}'`"});
        impl_select_page!(DbDailyPage{select_page() =>"
     if !sql.contains('count'):
       `order by create_time desc`"});
        impl_select_page!(DbDailyPage{select_page_by_title(title:&str) =>"
     if title != null && title != '':
       `where name != #{title}`
     if title == '':
       `where title != ''`"});
    }
}

impl DbPageContent{
    pub async fn new(content: &PageContentItem, rb: &mut Rbatis) -> Self {
        let page_id = DbDailyPage::new(content.publish_page.clone()).insert_or_exists_id(rb).await;
        DbPageContent {
            id: None,
            title: Some(content.title.clone()),
            md_content: Some(content.md_content.clone()),
            publish_page: page_id,
        }
    }

    pub async fn insert_or_exists_id(&self, rb: &mut Rbatis) -> Option<i32>{
        let exist = DbPageContent::select_by_title(rb, "daily_page_content", self.title.as_ref().unwrap().as_str()).await;
        match exist {
            Ok(v) => match v {
                Some(v) => { // v is DbPageContent type
                    println!("query_results: {v:?}");
                    v.id
                },
                None => {
                    println!("not exists");
                    let data = DbPageContent::insert(rb, &self).await;
                    match data {
                        Ok(new_v) => { // v is ExecResult type
                            println!("insert_result: {new_v:?}");
                            Some(new_v.last_insert_id.as_u64().unwrap() as i32)
                        },
                        Err(e) => {
                            println!("error inserting: {e:?}");
                            None
                        }
                    }
                }
            }
            Err(e) => {
                println!("error querying: {e:?}");
                None
            }
        }
    }

    pub fn crud_methods_init() {
        //crud!(BizActivity {},"biz_activity");//custom table name
//impl_select!(BizActivity{select_all_by_id(table_name:&str,id:&str) => "`where id = #{id}`"}); //custom table name
        crud!(DbPageContent {}, "daily_page_content");
        //         crud!(DbPageContent {}, "RustCC_Daily");
        impl_select!(DbPageContent{select_all_by_id(id:&str,title:&str) => "`where id = #{id} and title = #{title}`"});
        impl_select!(DbPageContent{select_by_id(id:&str) -> Option => "`where id = #{id} limit 1`"});
        impl_select!(DbPageContent{select_by_title(table_name:&str, title:&str) -> Option => "`where title = #{title} limit 1`"});
        // impl_select!(DbPageContent{select_by_url(table_name:&str, url:&str) -> Option => "`where url = #{url}`"});
        impl_update!(DbPageContent{update_by_title(title:&str) => "`where title = '#{title}'`"});
        impl_delete!(DbPageContent{delete_by_title(title:&str) => "`where name= '#{title}'`"});
        impl_select_page!(DbPageContent{select_page() =>"
     if !sql.contains('count'):
       `order by create_time desc`"});
        impl_select_page!(DbPageContent{select_page_by_title(title:&str) =>"
     if title != null && title != '':
       `where name != #{title}`
     if title == '':
       `where title != ''`"});
    }
}

pub async fn init_rustcc_db() -> Rbatis {
    let rb = Rbatis::new();
    // ------------choose database driver------------
    // rb.init(rbdc_mysql::driver::MysqlDriver {}, "mysql://root:123456@localhost:3306/test").unwrap();
    // rb.init(rbdc_pg::driver::PgDriver {}, "postgres://postgres:123456@localhost:5432/postgres").unwrap();
    // rb.init(rbdc_mssql::driver::MssqlDriver {}, "mssql://SA:TestPass!123456@localhost:1433/test").unwrap();
    rb.init(
        rbdc_sqlite::driver::SqliteDriver {},
        "sqlite://target/sqlite.db",
    )
        .unwrap();

    // ------------create tables way 2------------
    let sql = std::fs::read_to_string("examples/sqls/rustcc_table_sqlite.sql").unwrap();
    let raw = fast_log::LOGGER.get_level().clone();
    fast_log::LOGGER.set_level(LevelFilter::Off);
    defer!(||{
         fast_log::LOGGER.set_level(raw);
    });
    let _ = rb.exec(&sql, vec![]).await;
    // ------------create tables way 2 end------------
    rb
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*; // 当前文件中的, mod上一级
    use crate::aw; // 从lib.rs开始

    #[test]
    fn test_db_daily_page_operations() {
        fast_log::init(
            fast_log::Config::new()
                .console()
                .level(log::LevelFilter::Debug),
        )
            .expect("rbatis init fail");
        let mut rb = aw!(init_rustcc_db());
        DbDailyPage::crud_methods_init();
        DbPageContent::crud_methods_init();
        let test_daily_page = DailyPageItem {
            title: "title".to_string(),
            url: "url1kk2llk3k4".to_string(),
            publish_date: "publish_data".to_string(),
        };
        let test_page_content = PageContentItem{
            title: "content_title".to_string(),
            md_content: "md_content".to_string(),
            // publish_page: test_daily_page,
            publish_page: test_daily_page.clone()
        };
        let test_db_daily_page = DbDailyPage::new(test_daily_page);
        let test_db_page_content = aw!(DbPageContent::new(&test_page_content, &mut rb));
        // let tables = [
        //     test_db_daily_page.clone(),
        //     {
        //         let mut t3 = test_db_daily_page.clone();
        //         t3
        //     }
        // ];
        // let data = aw!(DbDailyPage::insert(&mut rb, &test_db_daily_page));
        // // let exist = aw!(DbDailyPage::select_by_url(&mut rb, "daily_page", test_db_daily_page.url.unwrap().as_str()));
        // let exist = aw!(DbDailyPage::select_by_url(&mut rb, "daily_page", "url_not_exist"));
        // match data {
        //     Ok(v) => println!("insert_result: {v:?}"),
        //     Err(e) => println!("error inserting: {e:?}")
        // }
        // match exist {
        //     Ok(v) => match v {
        //         Some(v) => println!("query_results: {v:?}"),
        //         None => println!("not exists")
        //     }
        //     Err(e) => println!("error querying: {e:?}")
        // }
        // let result = test_db_daily_page.insert_or_exists_id(&mut rb).unwrap();
        let result = aw!(test_db_page_content.insert_or_exists_id(&mut rb)).unwrap();
        println!("page insert result: {result:?}")
    }
}