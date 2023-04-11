use std::{
    collections::HashMap,
    hash::Hash,
    net::{Ipv4Addr, SocketAddr},
};

use serde::{Deserialize, Serialize};
use tokio::sync::{
    broadcast::{self, Receiver, Sender},
    mpsc::Receiver as BoundedReceiver,
};
use tracing::{error, info};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<E>
where
    E: std::fmt::Debug + Clone + Send + Sync,
{
    pub id: uuid::Uuid,
    pub timestamp: i64,
    pub data: E,
    pub topic: Option<String>,
}

impl<E> Message<E>
where
    E: std::fmt::Debug + Clone + Send + Sync,
{
    pub fn new(data: E, topic: Option<Topic>) -> Self {
        let id = uuid::Uuid::new_v4();
        let timestamp = chrono::Utc::now().timestamp();

        Self {
            id,
            timestamp,
            data,
            topic,
        }
    }

    pub fn new_with_id(id: uuid::Uuid, data: E, topic: Option<Topic>) -> Self {
        let id = uuid::Uuid::new_v4();
        let timestamp = chrono::Utc::now().timestamp();

        Self {
            id,
            timestamp,
            data,
            topic,
        }
    }
}

pub const DEFAULT_BUFFER: usize = 1;
pub const DEFAULT_TOPIC: &str = "default-topic-queue";

impl<E> Default for Router<E>
where
    E: std::fmt::Debug + Clone + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

pub type Topic = String;

/// Router is a message bus that coordinates interaction
/// among async tasks.
#[derive(Debug)]
pub struct Router<E>
where
    E: std::fmt::Debug + Clone + Send + Sync,
{
    // NOTE: short for "Message Queues"
    mqs: HashMap<Topic, Sender<Message<E>>>,
}

impl<E> Router<E>
where
    E: std::fmt::Debug + Clone + Send + Sync,
{
    pub fn new() -> Self {
        let mqs = HashMap::new();

        Self { mqs }
    }

    // pub fn new_with_topics<T: Iterator + Ord + PartialEq>(topics: Option<T>) -> Self {
    //     let mqs = HashMap::new();
    //     if let Some(topics) = topics {
    //         for topic in topics {
    //             let (sender, _) = broadcast::channel(DEFAULT_BUFFER);
    //
    //             mqs.insert(topic, sender);
    //         }
    //     }
    //
    //     Self { mqs }
    // }

    // pub fn add_topic(&mut self, topic: Topic, buffer: Option<usize>) -> Receiver<Message<E>> {
    pub fn add_topic(&mut self, topic: Topic, buffer: Option<usize>) {
        let buffer = buffer.unwrap_or(DEFAULT_BUFFER);
        let (sender, _) = broadcast::channel(buffer);

        self.mqs.insert(topic, sender);
    }

    pub fn subscribe(&self, topic: &Topic) -> Result<Receiver<Message<E>>> {
        if let Some(sender) = self.mqs.get(topic) {
            Ok(sender.subscribe())
        } else {
            Err(Error::Other(format!("unable to subscribe to {topic:?}")))
        }
    }

    /// Starts the event router, distributing all incomming messages to subscribers
    pub async fn start(&mut self, event_rx: &mut BoundedReceiver<Message<E>>) {
        while let Some(event) = event_rx.recv().await {
            if event == Event::Stop {
                tracing::info!("event router received stop signal");
                self.fan_out_message(Event::Stop);

                return;
            }

            self.fan_out_message(event);
        }
    }

    fn fan_out_message(&mut self, message: Message<E>) {
        let topic = message.topic.unwrap_or(DEFAULT_TOPIC.to_string());

        if let Some(topic_sender) = self.mqs.get_mut(&topic) {
            if let Err(err) = topic_sender.send(message.clone()) {
                error!("failed to send event {message:?} to topic {topic:?}: {err:?}");
            }
        }
    }
}
