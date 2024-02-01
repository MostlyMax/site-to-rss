use std::str::Utf8Error;
use async_openai::error::OpenAIError;
use aws_sdk_s3::primitives::ByteStreamError;
use aws_sdk_s3::Error as S3Error;
use rocket::response::Responder;
use std::fmt::Debug;


#[derive(Responder, Debug)]
pub enum Error {
    #[response(status = 400)]
    BadRequest(&'static str),
    #[response(status = 404)]
    NotFound(&'static str),
    #[response(status = 406)]
    UnhelpfulAI(String),
    #[response(status = 500)]
    Other(&'static str),
}

pub fn build_error_html(msg: &'static str) -> String {
    format!(
        r#"<div class="form-item error"><p>{}</p></div>"#,
        msg
    )
}

impl From<regex::Error> for Error {
    fn from(_: regex::Error) -> Self {
        Self::BadRequest("unable to parse regex")
    }
}

impl From<OpenAIError> for Error {
    fn from(_: OpenAIError) -> Self {
        Self::Other("openai api error")
        // match e {
        //     OpenAIError::Reqwest(_) => todo!(),
        //     OpenAIError::ApiError(_) => todo!(),
        //     OpenAIError::JSONDeserialize(_) => todo!(),
        //     OpenAIError::FileSaveError(_) => todo!(),
        //     OpenAIError::FileReadError(_) => todo!(),
        //     OpenAIError::StreamError(_) => todo!(),
        //     OpenAIError::InvalidArgument(_) => todo!(),
        // }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        let mut error = Error::Other("Unknown error");

        if e.is_connect() || e.is_redirect() || e.is_timeout() {
            error = Self::BadRequest("unable to connect to site url");
        }

        else if e.is_body() || e.is_decode() {
            error = Self::BadRequest("unable to decode site body");
        }

        error
    }
}

impl From<aws_sdk_s3::Error> for Error {
    fn from(e: aws_sdk_s3::Error) -> Self {
        let error = Error::Other("AWS SDK Error");

        match e {
            S3Error::NoSuchKey(_) => Error::NotFound("unable to find xml object"),
            S3Error::NoSuchBucket(_) => Error::NotFound("unable to find xml object"),
            S3Error::NotFound(_) => Error::NotFound("unable to find xml object"),
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
        let error = Error::Other("ByteStream Error");

        error
    }
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        let error = Error::Other("UTF-8 Error");

        error
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Self {
        let error = Error::Other("serialization Error");

        error
    }
}
