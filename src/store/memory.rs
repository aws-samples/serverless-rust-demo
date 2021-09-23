use super::{AllResponse, Store};
use crate::{Error, Product};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

// In-memory store
#[derive(Default)]
pub struct MemoryStore {
    data: RwLock<HashMap<String, Product>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Default::default()
    }
}

#[async_trait]
impl Store for MemoryStore {
    async fn all(&self, _: Option<&str>) -> Result<AllResponse, Error> {
        Ok(AllResponse {
            products: self
                .data
                .read()
                .unwrap()
                .iter()
                .map(|(_, v)| v.clone())
                .collect(),
            next: None,
        })
    }

    async fn get(&self, id: &str) -> Result<Option<Product>, Error> {
        Ok(self.data.read().unwrap().get(id).cloned())
    }

    async fn put(&self, product: &Product) -> Result<(), Error> {
        self.data
            .write()
            .unwrap()
            .insert(product.id.clone(), product.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), Error> {
        self.data.write().unwrap().remove(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{MemoryStore, Product};
    use crate::{Error, Store};

    #[tokio::test]
    async fn test_all() -> Result<(), Error> {
        let store = MemoryStore::new();
        let product0 = Product {
            id: "1".to_owned(),
            name: "foo".to_owned(),
            price: 10.0,
        };
        let product1 = Product {
            id: "2".to_owned(),
            name: "foo".to_owned(),
            price: 10.0,
        };

        // Put
        store.put(&product0).await?;
        assert_eq!(store.all(None).await?.products[0], product0);

        store.put(&product1).await?;

        // All
        let products = store.all(None).await?.products;
        assert_eq!(products.len(), 2);

        assert!(products.contains(&product0));
        assert!(products.contains(&product1));

        // Get
        assert_eq!(store.get(&product0.id).await?.unwrap(), product0);

        // Delete
        store.delete(&product1.id).await?;
        assert_eq!(store.get(&product1.id).await?, None);

        Ok(())
    }
}
