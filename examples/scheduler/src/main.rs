use chrono::Utc;
use silent::prelude::*;
use std::sync::Arc;

fn main() {
    logger::fmt().with_max_level(Level::INFO).init();
    let route = Route::new("").get(|req| async move {
        let process_time = Utc::now() + chrono::Duration::seconds(5);
        let task = Task::create_with_action_async(
            "task_id".to_string(),
            process_time.try_into().unwrap(),
            "task description".to_string(),
            Arc::new(|| {
                Box::pin(async {
                    println!("task run: {:?}", Utc::now());
                    Ok(())
                })
            }),
        );
        req.scheduler().lock().await.add_task(task)?;
        Ok("hello world")
    });
    Server::new().run(route);
}
