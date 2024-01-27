use rocket::{
    http::{Header, Status},
    request::{self, FromRequest},
    Request,
};

pub mod blobs;
pub mod manifests;

const CONTENT_TYPE_HEADER_NAME: &str = "Content-Type";
const CONTENT_RANGE_HEADER_NAME: &str = "Content-Range";
const CONTENT_LENGTH_HEADER_NAME: &str = "Content-Length";
const LOCATION_HEADER_NAME: &str = "Location";
const RANGE_HEADER_NAME: &str = "Range";
const DOCKER_CONTENT_DIGEST_HEADER_NAME: &str = "Docker-Content-Digest";

struct Auth {}

#[derive(Responder, Debug)]
#[response(status = 401)]
struct UnauthorizedResponse {
    inner: (),
    www_authenticate: Header<'static>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth {
    type Error = UnauthorizedResponse;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        println!("HEADERS: {:?}", req.headers());
        let auth_header = req.headers().get_one("authorization");
        if auth_header.is_none() {
            let resp = UnauthorizedResponse {
                inner: (),
                www_authenticate: Header::new("www-authenticate", "lol, no idea"),
            };

            return request::Outcome::Error((Status::Unauthorized, resp));
        }

        request::Outcome::Success(Auth {})
    }
}

#[get("/v2")]
pub fn get_spec_compliance(auth: Auth) -> () {
    ()
}
