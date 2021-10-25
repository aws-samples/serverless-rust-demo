use aws_sdk_dynamodb::model::AttributeValue;
use aws_smithy_http::result::SdkError;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    InitError(String),
    ClientError(String),
    InternalError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::InitError(msg) => write!(f, "InitError: {}", msg),
            Error::ClientError(msg) => write!(f, "ClientError: {}", msg),
            Error::InternalError(msg) => write!(f, "InternalError: {}", msg),
        }
    }
}

impl error::Error for Error {}

impl From<std::num::ParseFloatError> for Error {
    fn from(_: std::num::ParseFloatError) -> Error {
        Error::InternalError("Unable to parse float".to_string())
    }
}

impl From<&AttributeValue> for Error {
    fn from(_: &AttributeValue) -> Error {
        Error::InternalError("Invalid value type".to_string())
    }
}

impl<E> From<SdkError<E>> for Error
where
    E: error::Error,
{
    fn from(value: SdkError<E>) -> Error {
        Error::InternalError(format!("AWS Failure: {:?}", value))
    }
}
