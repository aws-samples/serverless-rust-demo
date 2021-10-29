//! Testing resources on AWS
//! ========================
//!
//! This file contains the tests for the AWS resources.
//!
//! This assumes that there is an environment variable called `REST_API`
//! which points to the endpoint of the Amazon API Gateway API.

use float_cmp::approx_eq;
use products::{Product, ProductRange};
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use reqwest::StatusCode;
use std::env;

type E = Box<dyn std::error::Error + Send + Sync + 'static>;

fn get_random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    return (0..length)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();
}

#[tokio::test]
async fn test_flow() -> Result<(), E> {
    let client = reqwest::Client::new();
    let rest_api: String = env::var("REST_API").expect("REST_API not set");

    let mut rng = rand::thread_rng();

    let product = Product {
        id: get_random_string(16),
        name: get_random_string(16),
        price: rng.gen::<f64>() * 256.0,
    };

    // Put new product
    println!("PUT new product");
    let res = client
        .put(format!("{}/{}", rest_api, product.id))
        .json(&product)
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::CREATED);

    // Get product
    println!("GET product");
    let res = client
        .get(format!("{}/{}", rest_api, product.id))
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::OK);
    let res_product: Product = res.json().await?;
    assert_eq!(res_product.id, product.id);
    assert_eq!(res_product.name, product.name);
    assert!(approx_eq!(f64, res_product.price, product.price));

    // Get all products
    println!("GET all products");
    let res = client.get(&rest_api).send().await?;
    assert_eq!(res.status(), StatusCode::OK);
    let res_products: ProductRange = res.json().await?;
    // At least one product should be returned
    assert!(res_products.products.len() >= 1);

    // Delete product
    println!("DELETE product");
    let res = client
        .delete(format!("{}/{}", rest_api, product.id))
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::OK);

    // Get product again
    println!("GET product again");
    let res = client
        .get(format!("{}/{}", rest_api, product.id))
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    Ok(())
}
