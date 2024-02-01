use rocket::{
    http::Status,
    request::{self, FromRequest},
    Request,
};

use crate::{
    api::container_spec::CONTENT_RANGE_HEADER_NAME,
    registry_error::{RegistryError, RegistryResult},
};

pub struct ContentRange {
    pub range_start: usize,
    pub range_end: usize,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ContentRange {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let headers = req.headers();

        let Some(content_range) = headers.get_one(CONTENT_RANGE_HEADER_NAME) else {
            warn!("Missing content-range header");
            return request::Outcome::Error((
                Status::BadRequest,
                format!("Missing {CONTENT_RANGE_HEADER_NAME} header"),
            ));
        };

        let Some((start, end)) = content_range.split_once("-") else {
            warn!("Invalid content-range header: {content_range}");
            return request::Outcome::Error((
                Status::BadRequest,
                format!("Invalid {CONTENT_RANGE_HEADER_NAME} header"),
            ));
        };

        let start: usize = match start.parse() {
            Ok(v) => v,
            Err(err) => {
                warn!("Failed to parse content-range start, err: {err:?}");
                return request::Outcome::Error((
                    Status::BadRequest,
                    format!("Invalid {CONTENT_RANGE_HEADER_NAME} header"),
                ));
            }
        };

        let end: usize = match end.parse() {
            Ok(v) => v,
            Err(err) => {
                warn!("Failed to parse content-range end, err: {err:?}");
                return request::Outcome::Error((
                    Status::BadRequest,
                    format!("Invalid {CONTENT_RANGE_HEADER_NAME} header"),
                ));
            }
        };

        if end < start {
            warn!("Content-range end ({end}) is less than start ({start})?");
            return request::Outcome::Error((
                Status::BadRequest,
                format!("Invalid {CONTENT_RANGE_HEADER_NAME} header"),
            ));
        }

        request::Outcome::Success(ContentRange {
            range_start: start,
            range_end: end,
        })
    }
}

impl ContentRange {
    pub fn expected_range(&self) -> RegistryResult<usize> {
        if self.range_end < self.range_start {
            return Err(RegistryError::InvalidContentRange);
        }

        Ok(self.range_end - self.range_start)
    }
}
