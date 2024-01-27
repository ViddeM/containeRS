use rocket::State;
use rocket_dyn_templates::{context, Template};
use sqlx::Pool;

use crate::{db::DB, services::get_images_service};

#[get("/")]
pub async fn get_main_view(db_pool: &State<Pool<DB>>) -> Template {
    let images = match get_images_service::get_all_images(db_pool).await {
        Ok(a) => a,
        Err(e) => return Template::render("error", context! {error: e.to_string()}),
    };

    Template::render("main_view", context! {images: images})
}
