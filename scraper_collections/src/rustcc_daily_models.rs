use log::LevelFilter;
use rbatis::{crud, impl_delete, impl_select, impl_select_page, impl_update, Rbatis};
use rbatis::dark_std::defer;
use serde::{Deserialize, Serialize};

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
    pub id: Option<String>,
    // 日报标题
    pub title: Option<String>,
    // url 地址
    pub url: Option<String>,
    pub publish_date: Option<String>,    // 日报日期
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DbPageContent {
    pub id: Option<String>,
    // 标题
    pub title: Option<String>,
    // 具体内容
    pub md_content: Option<String>,
    // 关联页面
    pub publish_page: Option<DailyPageItem>,
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
    fn crud_methods_init() {
        //crud!(BizActivity {},"biz_activity");//custom table name
//impl_select!(BizActivity{select_all_by_id(table_name:&str,id:&str) => "`where id = #{id}`"}); //custom table name
        crud!(DbDailyPage {}, "daily_page");
//         crud!(DbDailyPage {}, "RustCC_Daily");
        impl_select!(DbDailyPage{select_all_by_id(id:&str,title:&str) => "`where id = #{id} and title = #{title}`"});
        impl_select!(DbDailyPage{select_by_id(id:&str) -> Option => "`where id = #{id} limit 1`"});
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
    use serde_json::json;
    use crate::aw;
    use super::*;

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
        let test_daily_page = DailyPageItem{
            title: "title".to_string(),
            url: "url".to_string(),
            publish_date: "publish_data".to_string(),
        };
        let test_db_daily_page = DbDailyPage::new(test_daily_page);
        let tables = [
            test_db_daily_page.clone(),
            {
                let mut t3 = test_db_daily_page.clone();
                t3.id = "3".to_string().into();
                t3
            }
        ];
        let data = aw!(DbDailyPage::insert(&mut rb, &test_db_daily_page));
        println!("insert = {}", json!(data));
    }
}