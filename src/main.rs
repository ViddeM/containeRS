#![forbid(unsafe_code)]

use std::str::FromStr;

use config::Config;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions,
};

#[macro_use]
extern crate rocket;

pub mod api;
pub mod config;
pub mod db;
pub mod debug_headers;
pub mod models;
pub mod registry_error;
pub mod services;
pub mod types;

#[launch]
async fn rocket() -> _ {
    let config = Config::new().expect("Failed to load config");

    // Setup DB
    let mut pg_options =
        PgConnectOptions::from_str(&config.database_url).expect("Invalid database url provided");

    if !config.log_db_statements {
        pg_options.disable_statement_logging();
    }

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(pg_options)
        .await
        .expect("Failed to connect to DB");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    rocket::build()
        .mount(
            "/",
            routes![
                api::blobs::get_blob,
                api::get_spec_compliance,
                api::blobs::post_create_session,
                api::blobs::patch_upload_blob,
                api::blobs::put_upload_blob,
                api::manifests::put_manifest,
                api::manifests::get_manifest,
            ],
        )
        .manage(db_pool)
        .manage(config)
}
