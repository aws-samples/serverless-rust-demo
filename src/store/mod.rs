use crate::{Error, Product};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

mod dynamodb;
mod memory;

pub use dynamodb::DynamoDBStore;
pub use memory::MemoryStore;

#[derive(Default, Deserialize, Serialize)]
pub struct AllResponse {
    pub products: Vec<Product>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

#[async_trait]
pub trait Store {
    async fn all(&self, next: Option<&str>) -> Result<AllResponse, Error>;
    async fn get(&self, id: &str) -> Result<Option<Product>, Error>;
    async fn put(&self, product: &Product) -> Result<(), Error>;
    async fn delete(&self, id: &str) -> Result<(), Error>;
}
