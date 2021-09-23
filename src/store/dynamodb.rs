use super::{AllResponse, Store};
use crate::{Error, Product};
use async_trait::async_trait;
use aws_sdk_dynamodb::{model::AttributeValue, Client};
use aws_types::config::Config;
use std::collections::HashMap;
use std::str;
use tracing::instrument;

pub struct DynamoDBStore {
    client: Client,
    table_name: String,
}

enum ValueType {
    N,
    S,
}

impl DynamoDBStore {
    pub fn new(config: &Config, table_name: &str) -> DynamoDBStore {
        DynamoDBStore {
            client: Client::new(config),
            table_name: table_name.to_owned(),
        }
    }
}

trait ProductDynamoDBStoreExt {
    fn from_dynamodb(value: HashMap<String, AttributeValue>) -> Result<Product, Error>;
    fn to_dynamodb(&self) -> HashMap<String, AttributeValue>;
}

impl ProductDynamoDBStoreExt for Product {
    fn from_dynamodb(value: HashMap<String, AttributeValue>) -> Result<Product, Error> {
        Ok(Product {
            id: get_key("id", ValueType::S, &value)?,
            name: get_key("name", ValueType::S, &value)?,
            price: get_key("price", ValueType::N, &value)?.parse::<f64>()?,
        })
    }

    fn to_dynamodb(&self) -> HashMap<String, AttributeValue> {
        let mut retval = HashMap::new();
        retval.insert("id".to_owned(), AttributeValue::S(self.id.clone()));
        retval.insert("name".to_owned(), AttributeValue::S(self.name.clone()));
        retval.insert(
            "price".to_owned(),
            AttributeValue::N(format!("{:}", self.price)),
        );

        retval
    }
}

fn get_key(
    key: &str,
    t: ValueType,
    item: &HashMap<String, AttributeValue>,
) -> Result<String, Error> {
    let v = item
        .get(key)
        .ok_or_else(|| Error::InternalError(format!("Missing '{}'", key)))?;

    Ok(match t {
        ValueType::N => v.as_n()?.to_owned(),
        ValueType::S => v.as_s()?.to_owned(),
    })
}

#[async_trait]
impl Store for DynamoDBStore {
    #[instrument(skip(self))]
    // Get all items
    async fn all(&self, next: Option<&str>) -> Result<AllResponse, Error> {
        // Scan DynamoDB table
        let mut req = self.client.scan().table_name(&self.table_name);
        req = if let Some(next) = next {
            req.exclusive_start_key("id", AttributeValue::S(next.to_owned()))
        } else {
            req
        };
        let res = req.send().await?;

        // Build response
        let products = match res.items {
            Some(items) => items
                .into_iter()
                .map(Product::from_dynamodb)
                .collect::<Result<Vec<Product>, Error>>()?,
            None => Vec::default(),
        };
        let next = res
            .last_evaluated_key
            .map(|m| get_key("id", ValueType::S, &m).unwrap());
        Ok(AllResponse { products, next })
    }
    // Get item
    async fn get(&self, id: &str) -> Result<Option<Product>, Error> {
        let res = self
            .client
            .get_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_owned()))
            .send()
            .await?;
        Ok(match res.item {
            Some(item) => Some(Product::from_dynamodb(item)?),
            None => None,
        })
    }
    // Create or update an item
    async fn put(&self, product: &Product) -> Result<(), Error> {
        self.client
            .put_item()
            .table_name(&self.table_name)
            .set_item(Some(product.to_dynamodb()))
            .send()
            .await?;

        Ok(())
    }
    // Delete item
    async fn delete(&self, id: &str) -> Result<(), Error> {
        self.client
            .delete_item()
            .table_name(&self.table_name)
            .key("id", AttributeValue::S(id.to_owned()))
            .send()
            .await?;

        Ok(())
    }
}
