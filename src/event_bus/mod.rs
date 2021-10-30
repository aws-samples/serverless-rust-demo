use crate::Error;
use async_trait::async_trait;

mod eventbridge;
mod void;

pub use eventbridge::EventBridgeBus;
pub use void::VoidBus;

#[async_trait]
pub trait EventBus {
    type E;

    async fn send_event(&self, event: &Self::E) -> Result<(), Error>;
    async fn send_events(&self, events: &[Self::E]) -> Result<(), Error>;
}
