use rocket::{
    data::{self, FromData},
    http::Status,
    Data, Request,
};

use crate::api::container_spec::{APPLICATION_TYPE_OCTET_STREAM, CONTENT_TYPE_HEADER_NAME};

pub struct OctetStream {
    pub data: Vec<u8>,
}

#[rocket::async_trait]
impl<'r> FromData<'r> for OctetStream {
    type Error = String;

    async fn from_data(
        req: &'r Request<'_>,
        data: Data<'r>,
    ) -> data::Outcome<'r, Self, Self::Error> {
        if let Some(content_type) = req.headers().get_one(CONTENT_TYPE_HEADER_NAME) {
            if content_type != APPLICATION_TYPE_OCTET_STREAM {
                warn!("Non-octet stream content type, got {content_type} expected {APPLICATION_TYPE_OCTET_STREAM}");
                return data::Outcome::Forward((data, Status::BadRequest));
            }
        } else {
            warn!("Missing content type header");
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
