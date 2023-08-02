use serde::{Deserialize, Serialize};

pub type Topic = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message<E>
where
    E: std::fmt::Debug + Clone + Send + Sync + Eq + PartialEq,
{
    pub id: uuid::Uuid,
    pub timestamp: i64,
    pub data: MessageData<E>,
    pub topic: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageData<E>
where
    E: std::fmt::Debug + Clone + Send + Sync + Eq + PartialEq,
{
    #[default]
    Empty,
    Data(E),
    StopSignal,
}

impl<E> Message<E>
where
    E: std::fmt::Debug + Clone + Send + Sync + Eq + PartialEq,
{
    pub fn new(topic: Option<Topic>, data: E) -> Self {
        let id = uuid::Uuid::new_v4();
        let timestamp = chrono::Utc::now().timestamp();

        Self {
            id,
            timestamp,
            data: MessageData::Data(data),
            topic,
        }
    }

    pub fn new_with_id(id: uuid::Uuid, data: E, topic: Option<Topic>) -> Self {
        let timestamp = chrono::Utc::now().timestamp();

        Self {
            id,
            timestamp,
            data: MessageData::Data(data),
            topic,
        }
    }

    pub fn stop_signal(topic: Option<Topic>) -> Self {
        let id = uuid::Uuid::new_v4();
        let timestamp = chrono::Utc::now().timestamp();

        Self {
            id,
            timestamp,
            data: MessageData::StopSignal,
            topic,
        }
    }
}
