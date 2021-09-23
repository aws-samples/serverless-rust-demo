use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f64,
}
