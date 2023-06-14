use rocket::{
    http::{Header, Status},
    request::{self, FromRequest},
    Request, State,
};
use sqlx::Pool;

use crate::{config::Config, db::DB, services};

use super::{DOCKER_CONTENT_DIGEST_HEADER_NAME, LOCATION_HEADER_NAME};

const CONTENT_LENGTH_HEADER_NAME: &str = "Content-Length";
const CONTENT_TYPE_HEADER_NAME: &str = "Content-Type";

#[derive(Responder, Debug)]
pub struct PutManifestResponseData<'a> {
    response: &'a str,
    location: Header<'a>,
    docker_content_digest: Header<'a>,
}

#[derive(Responder, Debug)]
pub enum PutManifestResponse<'a> {
    #[response(status = 201)]
    Success(PutManifestResponseData<'a>),
    #[response(status = 400)]
    BadRequest(String),
    #[response(status = 500)]
    Failure(&'a str),
}

pub struct ManifestHeaders {
    content_length: usize,
    content_type: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ManifestHeaders {
    type Error = PutManifestResponse<'r>;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let content_length: usize = match req.headers().get_one(CONTENT_LENGTH_HEADER_NAME) {
            Some(h) => match h.parse() {
                Ok(val) => val,
                Err(e) => {
                    error!("Failed to parse content_length header, err: {e:?}");
                    return request::Outcome::Failure((
                        Status::BadRequest,
                        PutManifestResponse::BadRequest(format!(
                            "Invalid header {CONTENT_LENGTH_HEADER_NAME}"
                        )),
                    ));
                }
            },
            None => {
                error!("Missing content_length header");
                return request::Outcome::Failure((
                    Status::BadRequest,
                    PutManifestResponse::BadRequest(format!(
                        "Missing header {CONTENT_LENGTH_HEADER_NAME}"
                    )),
                ));
            }
        };

        let content_type = match req.headers().get_one(CONTENT_TYPE_HEADER_NAME) {
            Some(h) => h.to_string(),
            None => {
                error!("Missing content_type header");
                return request::Outcome::Failure((
                    Status::BadRequest,
                    PutManifestResponse::BadRequest(format!(
                        "Missing header {CONTENT_TYPE_HEADER_NAME}"
                    )),
                ));
            }
        };

        request::Outcome::Success(Self {
            content_length,
            content_type,
        })
    }
}

#[put("/v2/<name>/manifests/<reference>", data = "<manifest_data>")]
pub async fn put_manifest<'a>(
    name: &str,
    reference: &str,
    headers: ManifestHeaders,
    manifest_data: Vec<u8>,
    config: &State<Config>,
    db_pool: &State<Pool<DB>>,
) -> PutManifestResponse<'a> {
    match services::upload_manifest_service::upload_manifest(
        name.to_string(),
        reference.to_string(),
        headers.content_length,
        headers.content_type,
        manifest_data,
        config,
        db_pool,
    )
    .await
    {
        Ok((manifest_id, digest)) => PutManifestResponse::Success(PutManifestResponseData {
            response: "Upload manifest successful",
            location: Header::new(LOCATION_HEADER_NAME, format!("/{manifest_id}")),
            docker_content_digest: Header::new(DOCKER_CONTENT_DIGEST_HEADER_NAME, digest),
        }),
        Err(e) => {
            error!("Failed to upload manifest {e:?}");
            PutManifestResponse::Failure("Failed to upload manifest")
        }
    }
}
