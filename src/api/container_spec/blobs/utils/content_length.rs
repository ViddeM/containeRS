use rocket::{
    http::Status,
    request::{self, FromRequest},
    Request,
};

use crate::api::container_spec::CONTENT_RANGE_HEADER_NAME;

pub struct ContentLength {
    pub length: usize,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ContentLength {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let Some(content_length) = req.headers().get_one(CONTENT_RANGE_HEADER_NAME) else {
            return request::Outcome::Error((
                Status::BadRequest,
                format!("Missing {CONTENT_RANGE_HEADER_NAME} header"),
            ));
        };

        let length: usize = match content_length.parse() {
            Ok(l) => l,
            Err(err) => {
                warn!("Received invalid content-length header {content_length} (err: {err:?})");
                return request::Outcome::Error((
                    Status::BadRequest,
                    format!("Invalid {CONTENT_RANGE_HEADER_NAME} header"),
                ));
            }
        };

        request::Outcome::Success(ContentLength { length })
    }
}
