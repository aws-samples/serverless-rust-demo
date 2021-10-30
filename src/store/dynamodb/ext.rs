//! # Extension traits for `DynamoDbStore`.

use crate::{Error, Product};
use aws_sdk_dynamodb::model::AttributeValue;
use std::collections::HashMap;

/// Trait to convert a Product from/to a DynamoDB item
///
/// This trait adds implementations to convert a Product to/from a DynamoDB item
/// (represented as a `HashMap<String, AttributeValue>`).
///
/// There are two crates that currently provide this feature (`serde_dynamo` and
/// `serde_dynamodb`) in a more generic way, but they are only compatible with
/// Rusoto's DynamoDB API at the moment. Once these crates support the AWS SDK,
/// this trait can be removed entirely.
pub trait ProductsExt {
    fn from_dynamodb(value: HashMap<String, AttributeValue>) -> Result<Product, Error>;
    fn to_dynamodb(&self) -> HashMap<String, AttributeValue>;
}

impl ProductsExt for Product {
    fn from_dynamodb(value: HashMap<String, AttributeValue>) -> Result<Product, Error> {
        Ok(Product {
            id: value
                .get_s("id")
                .ok_or(Error::InternalError("Missing id"))?,
            name: value
                .get_s("name")
                .ok_or(Error::InternalError("Missing name"))?,
            price: value
                .get_n("price")
                .ok_or(Error::InternalError("Missing price"))?,
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

/// Trait to extract concrete values from a DynamoDB item
///
/// The DynamoDB client returns AttributeValues, which are enums that contain
/// the concrete values. This trait provides additional methods to the HashMap
/// to extract those values.
pub trait AttributeValuesExt {
    fn get_s(&self, key: &str) -> Option<String>;
    fn get_n(&self, key: &str) -> Option<f64>;
}

impl AttributeValuesExt for HashMap<String, AttributeValue> {
    /// Return a string from an key
    ///
    /// E.g. if you run `get_s("id")` on a DynamoDB item structured like this,
    /// you will retrieve the value `"foo"`.
    ///
    /// ```json
    /// {
    ///   "id": {
    ///     "S": "foo"
    ///   }
    /// }
    /// ```
    fn get_s(&self, key: &str) -> Option<String> {
        Some(self.get(key)?.as_s().ok()?.to_owned())
    }

    /// Return a number from an key
    ///
    /// E.g. if you run `get_n("price")` on a DynamoDB item structured like this,
    /// you will retrieve the value `10.0`.
    ///
    /// ```json
    /// {
    ///  "price": {
    ///   "N": "10.0"
    ///   }
    /// }
    /// ```
    fn get_n(&self, key: &str) -> Option<f64> {
        self.get(key)?.as_n().ok()?.parse::<f64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_from_dynamodb() {
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
    fn product_to_dynamodb() {
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

    #[test]
    fn attributevalue_get_s() {
        let mut item = HashMap::new();
        item.insert("id".to_owned(), AttributeValue::S("foo".to_owned()));

        assert_eq!(item.get_s("id"), Some("foo".to_owned()));
    }

    #[test]
    fn attributevalue_get_s_missing() {
        let mut item = HashMap::new();
        item.insert("id".to_owned(), AttributeValue::S("foo".to_owned()));

        assert_eq!(item.get_s("foo"), None);
    }

    #[test]
    fn attributevalue_get_n() {
        let mut item = HashMap::new();
        item.insert("price".to_owned(), AttributeValue::N("10.0".to_owned()));

        assert_eq!(item.get_n("price"), Some(10.0));
    }

    #[test]
    fn attributevalue_get_n_missing() {
        let mut item = HashMap::new();
        item.insert("price".to_owned(), AttributeValue::N("10.0".to_owned()));

        assert_eq!(item.get_n("foo"), None);
    }
}
