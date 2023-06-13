use rocket::{
    request::{self, FromRequest},
    Request,
};

pub struct DebugHeaders;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DebugHeaders {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        req.headers().iter().for_each(|h| {
            println!("HEADER: {h:?}");
        });

        request::Outcome::Success(Self {})
    }
}
