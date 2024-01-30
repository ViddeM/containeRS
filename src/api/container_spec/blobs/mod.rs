use rocket::{
    data::{self, FromData},
    http::Status,
    request::{self, FromRequest},
    Data, Request,
};

use super::{
    APPLICATION_TYPE_OCTET_STREAM, CONTENT_LENGTH_HEADER_NAME, CONTENT_RANGE_HEADER_NAME,
    CONTENT_TYPE_HEADER_NAME,
};

pub mod blobs;
pub mod chunked;
pub mod monolithic;
pub mod read_blob;
pub mod streamed;

pub struct OctetStream {
    data: Vec<u8>,
}

#[rocket::async_trait]
impl<'r> FromData<'r> for OctetStream {
    type Error = String;

    async fn from_data(
        req: &'r Request<'_>,
        data: Data<'r>,
    ) -> data::Outcome<'r, Self, Self::Error> {
        let Some(content_type) = req.headers().get_one(CONTENT_TYPE_HEADER_NAME) else {
            return data::Outcome::Error((
                Status::BadRequest,
                format!("Missing {CONTENT_TYPE_HEADER_NAME} header"),
            ));
        };

        if content_type != APPLICATION_TYPE_OCTET_STREAM {
            return data::Outcome::Error((
                Status::BadRequest,
                format!("Invalid content-type for blob upload, expected {APPLICATION_TYPE_OCTET_STREAM}, got {content_type}")
            ));
        }

        let bytes = match Vec::<u8>::from_data(req, data).await {
            data::Outcome::Success(d) => d,
            data::Outcome::Forward(resp) => return data::Outcome::Forward(resp),
            rocket::outcome::Outcome::Error((_, err)) => {
                error!("Failed to read body as Vec<u8>, err: {err:?}");
                return rocket::outcome::Outcome::Error((
                    Status::BadRequest,
                    "Invalid request body".to_string(),
                ));
            }
        };

        data::Outcome::Success(OctetStream { data: bytes })
    }
}

#[derive(Debug)]
pub struct BlobUploadHeaders {
    pub content_length: usize,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BlobUploadHeaders {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let headers = req.headers();
        let Some(content_length) = headers.get_one(CONTENT_LENGTH_HEADER_NAME) else {
            return request::Outcome::Error((
                Status::BadRequest,
                format!("Missing {CONTENT_LENGTH_HEADER_NAME} header"),
            ));
        };

        let Ok(content_length) = content_length.parse() else {
            return request::Outcome::Error((
                Status::BadRequest,
                format!("Invalid {CONTENT_LENGTH_HEADER_NAME} header value"),
            ));
        };

        request::Outcome::Success(BlobUploadHeaders { content_length })
    }
}

#[derive(Debug)]
pub struct UploadRangeHeader {
    start: usize,
    end: usize,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UploadRangeHeader {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let Some(content_range) = req.headers().get_one(CONTENT_RANGE_HEADER_NAME) else {
            return request::Outcome::Error((
                Status::BadRequest,
                format!("Missing {CONTENT_RANGE_HEADER_NAME} header"),
            ));
        };

        let Some((start, end)) = content_range.split_once("-") else {
            return request::Outcome::Error((
                Status::BadRequest,
                format!("Invalid {CONTENT_RANGE_HEADER_NAME} header value"),
            ));
        };

        let Ok(start) = start.parse() else {
            return request::Outcome::Error((
                Status::BadRequest,
                format!("Invalid {CONTENT_RANGE_HEADER_NAME} header value"),
            ));
        };

        let Ok(end) = end.parse() else {
            return request::Outcome::Error((
                Status::BadRequest,
                format!("Invalid {CONTENT_RANGE_HEADER_NAME} header value"),
            ));
        };

        request::Outcome::Success(UploadRangeHeader { start, end })
    }
}
