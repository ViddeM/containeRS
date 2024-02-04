use rocket::{
    http::Status,
    request::{self, FromRequest},
    Request,
};

use crate::api::container_spec::CONTENT_TYPE_HEADER_NAME;

pub struct ContentTypee {
    pub content_type: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ContentTypee {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let Some(content_type) = req.headers().get_one(CONTENT_TYPE_HEADER_NAME) else {
            warn!("Missing content-type header");
            return request::Outcome::Forward(Status::BadRequest);
        };

        request::Outcome::Success(ContentTypee {
            content_type: content_type.into(),
        })
    }
}
