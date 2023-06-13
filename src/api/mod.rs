pub mod blobs;
pub mod manifests;

#[get("/v2")]
pub fn get_spec_compliance() -> () {
    ()
}
