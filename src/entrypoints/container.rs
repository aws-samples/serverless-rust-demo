use crate::{domain, store};
use rocket::State;
use serde_json::json;
use tracing::{error, info, instrument};

pub struct Config {
    store: store::DynamoDBStore,
}

impl Config {
    pub fn new(store: store::DynamoDBStore) -> Self {
        Self { store }
    }
}

#[rocket::delete("/<id>")]
#[instrument(skip(state))]
pub async fn delete_product(state: &State<Config>, id: String) -> String {

    // Delete product
    info!("Deleting product {}", id);
    let res = domain::delete_product(&state.store, &id).await;

    match res {
        Ok(_) => {
            info!("Product {} deleted", id);
            json!({ "message": "Product deleted" })
        }
        Err(err) => {
            error!("Error deleting the product {}: {}", id, err);
            json!({ "message": "Failed to delete product" })
        }
    }
    .to_string()
}

#[rocket::get("/<id>")]
#[instrument(skip(state))]
pub async fn get_product(state: &State<Config>, id: String) -> String {
    let res = domain::get_product(&state.store, &id).await;

    match res {
        Ok(product) => json!(product),
        Err(err) => {
            error!("Error getting the product {}: {}", id, err);
            json!({ "message": "Failed to get product" })
        }
    }
    .to_string()
}

#[rocket::get("/")]
#[instrument(skip(state))]
pub async fn get_products(state: &State<Config>) -> String {
    let res = domain::get_products(&state.store, None).await;

    match res {
        Ok(res) => json!(res),
        Err(err) => {
            error!("Something went wrong: {:?}", err);
            json!({ "message": format!("Something went wrong: {:?}", err) })
        }
    }
    .to_string()
}

#[rocket::put("/<id>", data = "<product>")]
#[instrument(skip(state))]
pub async fn put_product(state: &State<Config>, id: String, product: String) -> String {
    // TODO: Validate the product
    let product = serde_json::from_str(&product).unwrap();

    let res = domain::put_product(&state.store, &product).await;

    match res {
        Ok(_) => {
            info!("Created product {:?}", product.id);
            json!({ "message": "Product created" })
        }
        Err(err) => {
            error!("Failed to create product {}: {}", product.id, err);
            json!({ "message": "Failed to create product" })
        }
    }
    .to_string()
}
