use crate::{Error, Product, ProductRange};
use async_trait::async_trait;

mod dynamodb;
mod memory;

pub use dynamodb::DynamoDBStore;
pub use memory::MemoryStore;

pub trait Store: StoreGetAll + StoreGet + StorePut + StoreDelete {}

/// Trait for retrieving all products
///
/// This trait is implemented by the different storage backends. It provides
/// the basic interface for retrieving all products.
///
/// A given store could return only a partial list of all the products. If
/// this is the case, the `next` parameter should be used to retrieve the
/// next page of products.
#[async_trait]
pub trait StoreGetAll: Send + Sync {
    async fn all(&self, next: Option<&str>) -> Result<ProductRange, Error>;
}

/// Trait for retrieving a single product
#[async_trait]
pub trait StoreGet: Send + Sync {
    async fn get(&self, id: &str) -> Result<Option<Product>, Error>;
}

/// Trait for storing a single product
#[async_trait]
pub trait StorePut: Send + Sync {
    async fn put(&self, product: &Product) -> Result<(), Error>;
}

/// Trait for deleting a single product
#[async_trait]
pub trait StoreDelete: Send + Sync {
    async fn delete(&self, id: &str) -> Result<(), Error>;
}
