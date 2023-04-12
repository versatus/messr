pub mod message;
pub mod result;
pub mod router;

pub use message::*;
pub use result::*;
pub use router::*;

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc::channel;

    use super::*;

    #[tokio::test]
    async fn should_stop_when_issued_stop_event() {
        let (event_tx, mut event_rx) = channel::<Message<&str>>(1);
        let mut router = Router::default();

        let mut subscriber_rx = router.subscribe(None).unwrap();

        let handle = tokio::spawn(async move {
            router.start(&mut event_rx).await;
        });

        let message = Message::new(None, "lorem ipsum dolor sit amet");

        event_tx.send(message.clone()).await.unwrap();

        let message = Message::stop_signal(None);

        event_tx.send(message.clone()).await.unwrap();

        handle.await.unwrap();
    }
}
