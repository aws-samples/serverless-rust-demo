//! # Domain logic for the service

pub mod entrypoints;
mod error;
pub mod event_bus;
mod model;
pub mod store;
pub mod utils;

pub use error::Error;
use event_bus::EventBus;
pub use model::{Event, Product, ProductRange};
use store::Store;

/// CRUD Service
/// 
/// This service handles CRUD operations for products.
/// 
/// Since this only fetches and return data from DynamoDB, the functions here
/// are very simple. They just fetch the data from the store and return it.
/// 
/// In a real application, you would probably want to add some business logic
/// here, such as validating the data, or adding additional data to the response.
pub struct CrudService {
    store: Box<dyn Store + Send + Sync>,
}

impl CrudService {
    pub fn new(
        store: Box<dyn Store + Send + Sync>,
    ) -> Self {
        Self { store }
    }

    // Get a product by its ID
    pub async fn get_product(&self, id: &str) -> Result<Option<Product>, Error> {
        self.store.get(id).await
    }

    // Get a list of products
    pub async fn get_products(&self, next: Option<&str>) -> Result<ProductRange, Error> {
        self.store.all(next).await
    }

    // Create or update product
    pub async fn put_product(&self, product: &Product) -> Result<(), Error> {
        self.store.put(product).await
    }

    // Delete a product
    pub async fn delete_product(&self, id: &str) -> Result<(), Error> {
        self.store.delete(id).await
    }
}


/// Event Service
/// 
/// This service takes events and publishes them to the event bus.
pub struct EventService {
    event_bus: Box<dyn EventBus<E = Event> + Send + Sync>,
}

impl EventService {
    pub fn new(
        event_bus: Box<dyn EventBus<E = Event> + Send + Sync>,
    ) -> Self {
        Self { event_bus }
    }

    // Send a batch of events
    pub async fn send_events(&self, events: &[Event]) -> Result<(), Error> {
        self.event_bus.send_events(events).await
    }
}
