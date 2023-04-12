use std::collections::HashMap;

use tokio::sync::{
    broadcast::{self, Receiver, Sender},
    mpsc::Receiver as BoundedReceiver,
};
use tracing::error;

use crate::{
    result::{Error, Result},
    Message, MessageData, Topic,
};

pub const DEFAULT_BUFFER: usize = 1;
pub const DEFAULT_TOPIC: &str = "default-topic-queue";

impl<E> Default for Router<E>
where
    E: std::fmt::Debug + Clone + Send + Sync + Eq + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Router is a message bus that coordinates interaction
/// among async tasks.
#[derive(Debug)]
pub struct Router<E>
where
    E: std::fmt::Debug + Clone + Send + Sync + Eq + PartialEq,
{
    // NOTE: short for "Message Queues"
    mqs: HashMap<Topic, Sender<Message<E>>>,
}

impl<E> Router<E>
where
    E: std::fmt::Debug + Clone + Send + Sync + Eq + PartialEq,
{
    pub fn new() -> Self {
        let mut mqs = HashMap::new();

        let (sender, _) = broadcast::channel(DEFAULT_BUFFER);

        mqs.insert(DEFAULT_TOPIC.to_string(), sender);

        Self { mqs }
    }

    pub fn new_with_topics(topics: &[(Topic, usize)]) -> Self {
        let mut mqs = HashMap::new();
        for (topic_name, size) in topics {
            let (sender, _) = broadcast::channel(*size);

            mqs.insert(topic_name.to_string(), sender);
        }

        Self { mqs }
    }

    pub fn add_topic(&mut self, topic: Topic, buffer: Option<usize>) {
        let buffer = buffer.unwrap_or(DEFAULT_BUFFER);
        let (sender, _) = broadcast::channel(buffer);

        self.mqs.insert(topic, sender);
    }

    pub fn subscribe(&self, topic: Option<Topic>) -> Result<Receiver<Message<E>>> {
        let topic_name = {
            let default_topic_name = DEFAULT_TOPIC.to_string();
            topic.unwrap_or(default_topic_name)
        };

        if let Some(sender) = self.mqs.get(&topic_name) {
            Ok(sender.subscribe())
        } else {
            Err(Error::Other(format!(
                "unable to subscribe to {topic_name:?}"
            )))
        }
    }

    /// Starts the router, distributing all incomming messages to subscribers
    pub async fn start(&mut self, message_rx: &mut BoundedReceiver<Message<E>>) {
        while let Some(message) = message_rx.recv().await {
            if matches!(message.data, MessageData::StopSignal) {
                return;
            }

            self.fan_out_message(message);
        }
    }

    fn fan_out_message(&mut self, message: Message<E>) {
        let topic = message.topic.clone().unwrap_or(DEFAULT_TOPIC.to_string());

        if let Some(topic_sender) = self.mqs.get_mut(&topic) {
            if let Err(err) = topic_sender.send(message.clone()) {
                error!("failed to send event {message:?} to topic {topic:?}: {err:?}");
            }
        }
    }
}
