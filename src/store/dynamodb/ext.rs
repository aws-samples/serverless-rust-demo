//! # Extension traits for `DynamoDbStore`.

use aws_sdk_dynamodb::model::AttributeValue;
use std::collections::HashMap;

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
    /// Return a string from a key
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

    /// Return a number from a key
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
