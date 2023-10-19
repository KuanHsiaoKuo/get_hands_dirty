use std::sync::{Arc, Mutex};
use tokio::sync::Notify;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let cache = Arc::new(Mutex::new(Vec::<u32>::new()));
    let notify = Arc::new(Notify::new());

    // 生产者
    tokio::spawn(producer(cache.clone(), notify.clone()));

    // 消费者
    consumer(cache, notify).await;
}

async fn producer(cache: Arc<Mutex<Vec<u32>>>, notify: Arc<Notify>) {
    let mut count = 0;
    loop {
        tokio::time::sleep(Duration::from_secs(2)).await; // 模拟生产时间
        {
            let mut data = cache.lock().unwrap();
            data.push(count);
            count += 1;
            println!("Produced: {}", count);
        }
        // 通知消费者
        notify.notify_one();
    }
}

async fn consumer(cache: Arc<Mutex<Vec<u32>>>, notify: Arc<Notify>) {
    loop {
        // 等待被通知
        notify.notified().await;
        let data = {
            let mut data = cache.lock().unwrap();
            if !data.is_empty() {
                Some(data.remove(0))
            } else {
                None
            }
        };

        if let Some(val) = data {
            println!("Consumed: {}", val);
        }
    }
}
