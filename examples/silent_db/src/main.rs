use silent_db::mysql::base::TableUtils;
use sqlx::MySqlPool;

mod test_table;

#[tokio::main]
async fn main() {
    let pool = MySqlPool::connect("mysql://root:123456@127.0.0.1:3306/test")
        .await
        .expect("Connect to mysql failed");
    let migrate = silent_db::Migrate::new("migrations".to_string(), pool.clone());
    migrate.generate(Box::new(TableUtils::new())).await.unwrap();
}
