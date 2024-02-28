use silent_db::mysql::base::TableUtils;

mod models;

#[tokio::main]
async fn main() {
    let migrate = silent_db::Migrate::new(
        "migrations".to_string(),
        "mysql://root:123456@192.168.110.16:3306/test"
            // "mysql://hubert22:wangdaofeng@rm-bp1dt1ar4q08xwvz15o.mysql.rds.aliyuncs.com:3306/test"
            .parse()
            .unwrap(),
    );
    migrate.generate(Box::new(TableUtils::new())).await.unwrap();
    // migrate.migrate().await.unwrap();
    // migrate.rollback(1).await.unwrap();
}
