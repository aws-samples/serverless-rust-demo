use crate::{Error, Event};
use async_trait::async_trait;

mod eventbridge;

pub use eventbridge::EventBridgeBus;

#[async_trait]
pub trait EventBus {
    async fn send_event(&self, event: &Event) -> Result<(), Error>;
    async fn send_events(&self, events: &[&Event]) -> Result<(), Error>;
}
