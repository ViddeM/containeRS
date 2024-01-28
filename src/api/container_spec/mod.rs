use rocket::{
    http::Status,
    request::{self, FromRequest},
    Request, State,
};

use crate::{
    api::container_spec::auth_service::accounts_rs::AccountsRsUserResponse, config::Config,
};

use self::errors::UnauthorizedResponse;

pub mod auth_service;
pub mod blobs;
pub mod errors;
pub mod manifests;

const CONTENT_TYPE_HEADER_NAME: &str = "Content-Type";
const CONTENT_RANGE_HEADER_NAME: &str = "Content-Range";
const CONTENT_LENGTH_HEADER_NAME: &str = "Content-Length";
const LOCATION_HEADER_NAME: &str = "Location";
const RANGE_HEADER_NAME: &str = "Range";
const DOCKER_CONTENT_DIGEST_HEADER_NAME: &str = "Docker-Content-Digest";

pub struct Auth {
    username: String,
}

#[derive(Debug)]
pub enum AuthFailure {
    Unauthorized(UnauthorizedResponse),
    InternalServerError(String),
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth {
    type Error = AuthFailure;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let config = match req.guard::<&State<Config>>().await {
            rocket::outcome::Outcome::Success(s) => s,
            _ => {
                return request::Outcome::Error((
                    Status::InternalServerError,
                    AuthFailure::InternalServerError("Failed to retrieve config!".to_string()),
                ))
            }
        };

        println!("HEADERS: {:?}", req.headers());
        let Some(auth_header) = req.headers().get_one("authorization") else {
            println!("Request missing authorization header");
            return request::Outcome::Error((
                Status::Unauthorized,
                AuthFailure::Unauthorized(UnauthorizedResponse::new(config)),
            ));
        };

        if !auth_header.starts_with("Bearer ") {
            println!("Auth header doesn't start with 'Bearer '?");
            return request::Outcome::Error((
                Status::Unauthorized,
                AuthFailure::Unauthorized(UnauthorizedResponse::new(config)),
            ));
        }

        let client = reqwest::Client::new();
        let resp = match client
            .get(&config.accounts_rs_me_endpoint)
            .header("Authorization", auth_header)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                println!("Failed to send user request to accounts service, err: {e:?}");
                return request::Outcome::Error((
                    Status::Unauthorized,
                    AuthFailure::Unauthorized(UnauthorizedResponse::new(config)),
                ));
            }
        };

        let resp_status = resp.status();
        if !resp_status.is_success() {
            println!("Got error response (status {resp_status}) from accounts service");
            return request::Outcome::Error((
                Status::Unauthorized,
                AuthFailure::Unauthorized(UnauthorizedResponse::new(config)),
            ));
        }

        let user_info: AccountsRsUserResponse = match resp.json().await {
            Ok(u) => u,
            Err(e) => {
                println!("Failed to deserialize user request to accounts service, err: {e:?}");
                return request::Outcome::Error((
                    Status::Unauthorized,
                    AuthFailure::Unauthorized(UnauthorizedResponse::new(config)),
                ));
            }
        };

        request::Outcome::Success(Auth {
            username: user_info.success.email,
        })
    }
}

#[derive(Responder)]
pub enum SpecComplianceResponse {
    #[response(status = 200)]
    Ok(()),
    #[response(status = 401)]
    Unauthorized(UnauthorizedResponse),
    #[response(status = 500)]
    InternalServerError(()),
}

#[get("/v2")]
pub fn get_spec_compliance(auth: Result<Auth, AuthFailure>) -> SpecComplianceResponse {
    match auth {
        Ok(_) => SpecComplianceResponse::Ok(()),
        Err(AuthFailure::Unauthorized(resp)) => SpecComplianceResponse::Unauthorized(resp),
        Err(AuthFailure::InternalServerError(err)) => {
            println!("Internal server error {err:?}");
            SpecComplianceResponse::InternalServerError(())
        }
    }
}
