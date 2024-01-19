use std::str::Utf8Error;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::primitives::ByteStreamError;
use rocket::response::Responder;

#[derive(Responder, Debug)]
pub enum Error {
    #[response(status = 401)]
    BadRequest(String),
    #[response(status = 404)]
    NotFound(String),
    #[response(status = 500)]
    Other(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        let mut error = Error::Other("Unknown error".to_owned());

        if e.is_connect() || e.is_redirect() || e.is_timeout() {
            error = Self::BadRequest("unable to connect to site url".to_owned());
        }

        else if e.is_body() || e.is_decode() {
            error = Self::BadRequest("unable to decode site body".to_owned());
        }

        error
    }
}

impl<E, R> From<SdkError<E, R>> for Error {
    fn from(_: SdkError<E, R>) -> Self {
        let error = Error::Other("AWS SDK Error".to_owned());

        error
    }
}

impl From<ByteStreamError> for Error {
    fn from(_: ByteStreamError) -> Self {
        let error = Error::Other("ByteStream Error".to_owned());

        error
    }
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        let error = Error::Other("UTF-8 Error".to_owned());

        error
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        let error = Error::Other("serialization Error".to_owned());

        error
    }
}
