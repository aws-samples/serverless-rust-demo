//! Product representations
//! 
//! This module contains the representations of the products.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f64,
}

#[derive(Default, Deserialize, Serialize)]
pub struct ProductRange {
    pub products: Vec<Product>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}
