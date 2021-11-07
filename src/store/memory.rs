//! # In-memory store implementation
//!
//! This is a simple in-memory store implementation. It is not intended to be
//! used in production, but rather as a simple implementation for local
//! testing purposes.

use super::{Store, StoreDelete, StoreGet, StoreGetAll, StorePut};
use crate::{Error, Product, ProductRange};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Default)]
pub struct MemoryStore {
    data: RwLock<HashMap<String, Product>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Store for MemoryStore {}

#[async_trait]
impl StoreGetAll for MemoryStore {
    async fn all(&self, _: Option<&str>) -> Result<ProductRange, Error> {
        Ok(ProductRange {
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
}

#[async_trait]
impl StoreGet for MemoryStore {
    async fn get(&self, id: &str) -> Result<Option<Product>, Error> {
        Ok(self.data.read().unwrap().get(id).cloned())
    }
}

#[async_trait]
impl StorePut for MemoryStore {
    async fn put(&self, product: &Product) -> Result<(), Error> {
        self.data
            .write()
            .unwrap()
            .insert(product.id.clone(), product.clone());
        Ok(())
    }
}

#[async_trait]
impl StoreDelete for MemoryStore {
    async fn delete(&self, id: &str) -> Result<(), Error> {
        self.data.write().unwrap().remove(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    struct ConstProduct<'a> {
        id: &'a str,
        name: &'a str,
        price: f64,
    }

    impl Into<Product> for ConstProduct<'_> {
        fn into(self) -> Product {
            Product {
                id: self.id.to_string(),
                name: self.name.to_string(),
                price: self.price,
            }
        }
    }

    const PRODUCT_0: ConstProduct = ConstProduct {
        id: "1",
        name: "foo",
        price: 10.0,
    };
    const PRODUCT_1: ConstProduct = ConstProduct {
        id: "2",
        name: "foo",
        price: 10.0,
    };

    #[tokio::test]
    async fn test_new() -> Result<(), Error> {
        // GIVEN an empty store
        let store = MemoryStore::new();

        // WHEN we get the length of all products
        // THEN we get 0
        assert_eq!(store.data.read().unwrap().len(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_all_empty() -> Result<(), Error> {
        // GIVEN an empty store
        let store = MemoryStore::new();

        // WHEN we get all products
        let all = store.all(None).await?;

        // THEN we get an empty list
        assert_eq!(all.products.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_all1() -> Result<(), Error> {
        // GIVEN a store with one product
        let product0: Product = PRODUCT_0.into();
        let store = MemoryStore::new();
        {
            let mut data = store.data.write().unwrap();
            data.insert(product0.id.clone(), product0.clone());
        }

        // WHEN we get all products
        let all = store.all(None).await?;

        // THEN we get the product
        assert_eq!(all.products.len(), 1);
        assert_eq!(all.products[0], product0);

        Ok(())
    }

    #[tokio::test]
    async fn test_all2() -> Result<(), Error> {
        // GIVEN a store with two products
        let product0: Product = PRODUCT_0.into();
        let product1: Product = PRODUCT_1.into();
        let store = MemoryStore::new();
        {
            let mut data = store.data.write().unwrap();
            data.insert(product0.id.clone(), product0.clone());
            data.insert(product1.id.clone(), product1.clone());
        }

        // WHEN we get all products
        let all = store.all(None).await?;

        // THEN we get the products
        assert_eq!(all.products.len(), 2);
        assert!(all.products.contains(&product0));
        assert!(all.products.contains(&product1));

        Ok(())
    }

    #[tokio::test]
    async fn test_delete() -> Result<(), Error> {
        // GIVEN a store with a product
        let product0: Product = PRODUCT_0.into();
        let store = MemoryStore::new();
        {
            let mut data = store.data.write().unwrap();
            data.insert(product0.id.clone(), product0.clone());
        }

        // WHEN deleting the product
        store.delete(&product0.id).await?;

        // THEN the length of the store is 0
        assert_eq!(store.data.read().unwrap().len(), 0);
        // AND the product is not returned
        assert_eq!(store.get(&product0.id).await?, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete2() -> Result<(), Error> {
        // GIVEN a store with two products
        let product0: Product = PRODUCT_0.into();
        let product1: Product = PRODUCT_1.into();
        let store = MemoryStore::new();
        {
            let mut data = store.data.write().unwrap();
            data.insert(product0.id.clone(), product0.clone());
            data.insert(product1.id.clone(), product1.clone());
        }

        // WHEN deleting the first product
        store.delete(&product0.id).await?;

        // THEN the length of the store is 1
        assert_eq!(store.data.read().unwrap().len(), 1);
        // AND the product is not returned
        assert_eq!(store.get(&product0.id).await?, None);
        // AND the second product is returned
        assert_eq!(store.get(&product1.id).await?, Some(product1));

        Ok(())
    }

    #[tokio::test]
    async fn test_get() -> Result<(), Error> {
        // GIVEN a store with a product
        let product0: Product = PRODUCT_0.into();
        let store = MemoryStore::new();
        {
            let mut data = store.data.write().unwrap();
            data.insert(product0.id.clone(), product0.clone());
        }

        // WHEN getting the product
        let product = store.get(&product0.id).await?;

        // THEN the product is returned
        assert_eq!(product, Some(product0));

        Ok(())
    }

    #[tokio::test]
    async fn test_put() -> Result<(), Error> {
        // GIVEN an empty store and a product
        let store = MemoryStore::new();
        let product0: Product = PRODUCT_0.into();

        // WHEN inserting a product
        store.put(&product0).await?;

        // THEN the length of the store is 1
        assert_eq!(store.data.read().unwrap().len(), 1);
        // AND the product is returned
        assert_eq!(store.get(&product0.id).await?, Some(product0));

        Ok(())
    }

    #[tokio::test]
    async fn test_put2() -> Result<(), Error> {
        // GIVEN an empty store and two products
        let store = MemoryStore::new();
        let product0: Product = PRODUCT_0.into();
        let product1: Product = PRODUCT_1.into();

        // WHEN inserting two products
        store.put(&product0).await?;
        store.put(&product1).await?;

        // THEN the length of the store is 2
        assert_eq!(store.data.read().unwrap().len(), 2);
        // AND the products are returned
        assert_eq!(store.get(&product0.id).await?, Some(product0));
        assert_eq!(store.get(&product1.id).await?, Some(product1));

        Ok(())
    }
}
