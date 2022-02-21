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

fn get_random_product() -> Product {
    let mut rng = rand::thread_rng();
    Product {
        id: get_random_string(16),
        name: get_random_string(16),
        // Price with 2 decimal digits
        price: (rng.gen::<f64>() * 25600.0).round() / 100.0,
    }
}

#[tokio::test]
async fn test_flow() -> Result<(), E> {
    let client = reqwest::Client::new();
    let api_url: String = env::var("API_URL").expect("API_URL not set");

    let product = get_random_product();

    // Put new product
    println!("PUT new product");
    let res = client
        .put(format!("{}/{}", api_url, product.id))
        .json(&product)
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::CREATED);

    // Get product
    println!("GET product");
    let res = client
        .get(format!("{}/{}", api_url, product.id))
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::OK);
    let res_product: Product = res.json().await?;
    assert_eq!(res_product.id, product.id);
    assert_eq!(res_product.name, product.name);
    assert!(approx_eq!(f64, res_product.price, product.price));

    // Get all products
    println!("GET all products");
    let res = client.get(&api_url).send().await?;
    assert_eq!(res.status(), StatusCode::OK);
    let res_products: ProductRange = res.json().await?;
    // At least one product should be returned
    assert!(res_products.products.len() >= 1);

    // Delete product
    println!("DELETE product");
    let res = client
        .delete(format!("{}/{}", api_url, product.id))
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::OK);

    // Get product again
    println!("GET product again");
    let res = client
        .get(format!("{}/{}", api_url, product.id))
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    Ok(())
}

#[tokio::test]
async fn test_put_product_with_invalid_id() -> Result<(), E> {
    let client = reqwest::Client::new();
    let api_url: String = env::var("API_URL").expect("API_URL not set");

    let product = Product {
        id: "invalid id".to_string(),
        name: get_random_string(16),
        price: 0.0,
    };

    // Put new product
    println!("PUT new product");
    let res = client
        .put(format!("{}/not-the-same-id", api_url))
        .json(&product)
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(res
        .text()
        .await?
        .contains("Product ID in path does not match product ID in body"));

    Ok(())
}

#[tokio::test]
async fn test_put_product_empty() -> Result<(), E> {
    let client = reqwest::Client::new();
    let api_url: String = env::var("API_URL").expect("API_URL not set");

    // Put new product
    println!("PUT new product");
    let res = client.put(format!("{}/empty-id", api_url)).send().await?;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(res
        .text()
        .await?
        .contains("Missing product in request body"));

    Ok(())
}

#[tokio::test]
async fn test_put_product_invalid_body() -> Result<(), E> {
    let client = reqwest::Client::new();
    let api_url: String = env::var("API_URL").expect("API_URL not set");

    // Put new product
    println!("PUT new product");
    let res = client
        .put(format!("{}/invalid-body", api_url))
        .json(&"invalid body")
        .send()
        .await?;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(res
        .text()
        .await?
        .contains("Failed to parse product from request body"));

    Ok(())
}
