//! Domain logic for the application.

use crate::{
    error::Error,
    event_bus::EventBus,
    model::{Event, Product, ProductRange},
    store::{StoreDelete, StoreGet, StoreGetAll, StorePut},
};

pub async fn get_products(
    store: &dyn StoreGetAll,
    next: Option<&str>,
) -> Result<ProductRange, Error> {
    store.all(next).await
}

pub async fn get_product(store: &dyn StoreGet, id: &str) -> Result<Option<Product>, Error> {
    store.get(id).await
}

pub async fn put_product(store: &dyn StorePut, product: &Product) -> Result<(), Error> {
    // Round price to 2 decimal digits
    let mut product = product.clone();
    product.price = (product.price * 100.0).round() / 100.0;

    store.put(&product).await
}

pub async fn delete_product(store: &dyn StoreDelete, id: &str) -> Result<(), Error> {
    store.delete(id).await
}

pub async fn send_events(
    event_bus: &dyn EventBus<E = Event>,
    events: &[Event],
) -> Result<(), Error> {
    event_bus.send_events(events).await
}
