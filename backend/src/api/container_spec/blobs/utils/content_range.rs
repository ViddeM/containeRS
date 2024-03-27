use rocket::{
    http::Status,
    request::{self, FromRequest},
    Request,
};

use crate::{
    api::container_spec::CONTENT_RANGE_HEADER_NAME,
    registry_error::{RegistryError, RegistryResult},
};

use super::content_length::ContentLength;

#[derive(Debug)]
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
            return request::Outcome::Forward(Status::BadRequest);
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
    pub fn validate(&self, content_length: &ContentLength) -> RegistryResult<()> {
        let Some(length) = content_length.length else {
            return Ok(());
        };

        let expected_size = self
            .range_end
            .checked_sub(self.range_start)
            .ok_or_else(|| {
                warn!("Range end was less than range start for content range {self:?}");
                RegistryError::InvalidContentRange
            })?
            + 1; // Plus 1 because the length should be inclusive

        if expected_size != length {
            warn!(
                "Content range expected size {expected_size} did not match content_length {}",
                length
            );
            return Err(RegistryError::InvalidContentRange);
        }

        Ok(())
    }
}
