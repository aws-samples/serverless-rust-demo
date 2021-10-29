//! Trait to convert a Product from/to a DynamoDB item

use crate::{Error, Product};
use aws_sdk_dynamodb::model::AttributeValue;
use std::collections::HashMap;
use super::AttributeValuesExt;

pub trait ProductsExt {
    fn from_dynamodb(value: HashMap<String, AttributeValue>) -> Result<Product, Error>;
    fn to_dynamodb(&self) -> HashMap<String, AttributeValue>;
}

impl ProductsExt for Product {
    fn from_dynamodb(value: HashMap<String, AttributeValue>) -> Result<Product, Error> {
        Ok(Product {
            id: value.get_s("id").ok_or_else(|| Error::InternalError("Missing id"))?,
            name: value.get_s("name").ok_or_else(|| Error::InternalError("Missing name"))?,
            price: value.get_n("price").ok_or_else(|| Error::InternalError("Missing price"))?,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_dynamodb() {
        let mut value = HashMap::new();
        value.insert("id".to_owned(), AttributeValue::S("id".to_owned()));
        value.insert("name".to_owned(), AttributeValue::S("name".to_owned()));
        value.insert("price".to_owned(), AttributeValue::N("1.0".to_owned()));

        let product = Product::from_dynamodb(value).unwrap();
        assert_eq!(product.id, "id");
        assert_eq!(product.name, "name");
        assert_eq!(product.price, 1.0);
    }

    #[test]
    fn test_to_dynamodb() {
        let product = Product {
            id: "id".to_owned(),
            name: "name".to_owned(),
            price: 1.5,
        };

        let value = product.to_dynamodb();
        assert_eq!(value.get("id").unwrap().as_s().unwrap(), "id");
        assert_eq!(value.get("name").unwrap().as_s().unwrap(), "name");
        assert_eq!(value.get("price").unwrap().as_n().unwrap(), "1.5");
    }
}