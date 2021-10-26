mod error;
mod product;
pub mod store;
mod utils;

pub use error::Error;
pub use product::{Product, ProductRange};
pub use store::{DynamoDBStore, MemoryStore, Store};
pub use utils::{get_service, setup_tracing};

pub struct Service
{
    store: Box<dyn Store + Send + Sync>,
}

impl Service
{
    pub fn new(store: Box<dyn Store + Send + Sync>) -> Self {
        Self { store }
    }

    pub async fn get_product(&self, id: &str) -> Result<Option<Product>, Error> {
        self.store.get(id).await
    }

    pub async fn get_products(&self, next: Option<&str>) -> Result<ProductRange, Error> {
        self.store.all(next).await
    }

    pub async fn create_product(&self, product: &Product) -> Result<(), Error> {
        self.store.put(product).await
    }

    pub async fn delete_product(&self, id: &str) -> Result<(), Error> {
        self.store.delete(id).await
    }
}