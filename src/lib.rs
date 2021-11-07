//! # Domain logic for the service

pub mod domain;
pub mod entrypoints;
mod error;
pub mod event_bus;
mod model;
pub mod store;
pub mod utils;

pub use error::Error;
use event_bus::EventBus;
pub use model::{Event, Product, ProductRange};

/// Event Service
///
/// This service takes events and publishes them to the event bus.
pub struct EventService {
    event_bus: Box<dyn EventBus<E = Event> + Send + Sync>,
}

impl EventService {
    pub fn new(event_bus: Box<dyn EventBus<E = Event> + Send + Sync>) -> Self {
        Self { event_bus }
    }

    // Send a batch of events
    pub async fn send_events(&self, events: &[Event]) -> Result<(), Error> {
        self.event_bus.send_events(events).await
    }
}
