use async_trait::async_trait;

// 定时任务存储Trait
#[allow(dead_code)]
#[async_trait]
trait Storage {
    async fn load(&mut self);
    async fn save(&mut self);
}
