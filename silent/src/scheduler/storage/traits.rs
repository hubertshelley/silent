use async_trait::async_trait;

#[allow(dead_code)]
/// 定时任务存储Trait
#[async_trait]
trait Storage {
    async fn load(&mut self);
    async fn save(&mut self);
}
