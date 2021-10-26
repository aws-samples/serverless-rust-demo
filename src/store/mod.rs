use crate::{Error, Product, ProductRange};
use async_trait::async_trait;

mod dynamodb;
mod memory;

pub use dynamodb::DynamoDBStore;
pub use memory::MemoryStore;

#[async_trait]
pub trait Store {
    async fn all(&self, next: Option<&str>) -> Result<ProductRange, Error>;
    async fn get(&self, id: &str) -> Result<Option<Product>, Error>;
    async fn put(&self, product: &Product) -> Result<(), Error>;
    async fn delete(&self, id: &str) -> Result<(), Error>;
}
