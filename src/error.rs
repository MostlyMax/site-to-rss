use std::str::Utf8Error;
use aws_sdk_s3::primitives::ByteStreamError;
use aws_sdk_s3::Error as S3Error;
use rocket::response::Responder;
use std::fmt::Debug;

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



impl From<aws_sdk_s3::Error> for Error {
    fn from(e: aws_sdk_s3::Error) -> Self {
        let error = Error::Other("AWS SDK Error".to_owned());

        match e {
            S3Error::NoSuchKey(_) => Error::NotFound("unable to find xml object".to_owned()),
            S3Error::NoSuchBucket(_) => Error::NotFound("unable to find xml object".to_owned()),
            S3Error::NotFound(_) => Error::NotFound("unable to find xml object".to_owned()),
            _ => error,
        }
    }
}

impl<E, R> From<aws_sdk_s3::error::SdkError<E, R>> for Error
    where aws_sdk_s3::Error: From<aws_sdk_s3::error::SdkError<E, R>> {

    fn from(e: aws_sdk_s3::error::SdkError<E, R>) -> Self {
        let error = aws_sdk_s3::Error::from(e);
        error.into()
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
