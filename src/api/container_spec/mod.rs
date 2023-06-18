pub mod blobs;
pub mod manifests;

const CONTENT_TYPE_HEADER_NAME: &str = "Content-Type";
const CONTENT_RANGE_HEADER_NAME: &str = "Content-Range";
const CONTENT_LENGTH_HEADER_NAME: &str = "Content-Length";
const LOCATION_HEADER_NAME: &str = "Location";
const RANGE_HEADER_NAME: &str = "Range";
const DOCKER_CONTENT_DIGEST_HEADER_NAME: &str = "Docker-Content-Digest";

#[get("/v2")]
pub fn get_spec_compliance() -> () {
    ()
}
