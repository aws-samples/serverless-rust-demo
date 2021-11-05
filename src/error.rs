use aws_sdk_dynamodb::model::AttributeValue;
use aws_smithy_http::result::SdkError;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    InitError(&'static str),
    ClientError(&'static str),
    InternalError(&'static str),
    SdkError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::InitError(msg) => write!(f, "InitError: {}", msg),
            Error::ClientError(msg) => write!(f, "ClientError: {}", msg),
            Error::InternalError(msg) => write!(f, "InternalError: {}", msg),
            Error::SdkError(err) => write!(f, "SdkError: {}", err),
        }
    }
}

impl error::Error for Error {}

impl From<std::num::ParseFloatError> for Error {
    fn from(_: std::num::ParseFloatError) -> Error {
        Error::InternalError("Unable to parse float")
    }
}

impl From<&AttributeValue> for Error {
    fn from(_: &AttributeValue) -> Error {
        Error::InternalError("Invalid value type")
    }
}

impl<E> From<SdkError<E>> for Error
where
    E: error::Error,
{
    fn from(value: SdkError<E>) -> Error {
        Error::SdkError(format!("{}", value))
    }
}
