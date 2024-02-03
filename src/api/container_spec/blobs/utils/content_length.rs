use rocket::{
    http::Status,
    request::{self, FromRequest},
    Request,
};

use crate::{
    api::container_spec::CONTENT_LENGTH_HEADER_NAME,
    registry_error::{RegistryError, RegistryResult},
};

pub struct ContentLength {
    pub length: usize,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ContentLength {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        info!("ALL HEADERS {:?}", req.headers());
        let Some(content_length) = req.headers().get_one(CONTENT_LENGTH_HEADER_NAME) else {
            return request::Outcome::Error((
                Status::BadRequest,
                format!("Missing {CONTENT_LENGTH_HEADER_NAME} header"),
            ));
        };

        let length: usize = match content_length.parse() {
            Ok(l) => l,
            Err(err) => {
                warn!("Received invalid content-length header {content_length} (err: {err:?})");
                return request::Outcome::Error((
                    Status::BadRequest,
                    format!("Invalid {CONTENT_LENGTH_HEADER_NAME} header"),
                ));
            }
        };

        request::Outcome::Success(ContentLength { length })
    }
}

impl ContentLength {
    pub fn validate_blob_length(&self, blob_length: usize) -> RegistryResult<()> {
        if self.length != blob_length {
            warn!(
                "Got invalid content_length value ({}) when blob length was ({})",
                self.length, blob_length
            );
            return Err(RegistryError::InvalidContentLength);
        }
        Ok(())
    }
}
