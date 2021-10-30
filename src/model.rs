//! Data models
//!
//! This module contains the representations of the products.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f64,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ProductRange {
    pub products: Vec<Product>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Event {
    Created { product: Product },
    Updated { old: Product, new: Product },
    Deleted { product: Product },
}

impl Event {
    pub fn id(&self) -> &str {
        match self {
            Event::Created { product } => product.id.as_str(),
            Event::Updated { new, .. } => new.id.as_str(),
            Event::Deleted { product } => product.id.as_str(),
        }
    }
}
