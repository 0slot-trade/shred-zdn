use std::time::Duration;
use tokio::time::sleep;

pub async fn retry<
    T, E: std::fmt::Debug,
    Fut: std::future::Future<Output = Result<T, E>>,
>(    
    delay: impl IntoIterator<Item = Duration>,
    retries: usize,    
    operation: impl Fn(usize) -> Fut,
) -> Result<T, E> {
    let delay = delay.into_iter().collect::<Vec<_>>();
    let mut count = 0;
    loop {
        match operation(count).await {
            Ok(value) => return Ok(value), 
            Err(_) if count < retries => {}, 
            Err(err) => return Err(err), 
        }
        sleep( 
            delay.get(count.min(delay.len() - 1))
                .copied().unwrap_or(Duration::ZERO)
        ).await;
        count += 1;
    }
}