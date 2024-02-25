use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use sqlx::Pool;

use crate::{
    db::DB,
    services::get_images_service::{self, Image, Tag},
};

#[derive(Debug, Clone, Serialize)]
struct DisplayImage {
    name: String,
    tags: Vec<DisplayTag>,
}

impl From<Image> for DisplayImage {
    fn from(value: Image) -> Self {
        Self {
            name: value.name,
            tags: value.tags.into_iter().map(|v| v.into()).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct DisplayTag {
    name: String,
    update_date: String,
}

impl From<Tag> for DisplayTag {
    fn from(value: Tag) -> Self {
        Self {
            name: value.reference,
            update_date: value.created_at.format("%Y-%m-%d %H:%M").to_string(),
        }
    }
}

#[get("/")]
pub async fn get_main_view(db_pool: &State<Pool<DB>>) -> Template {
    let images = match get_images_service::get_all_images(db_pool).await {
        Ok(a) => a,
        Err(e) => return Template::render("error", context! {error: e.to_string()}),
    };

    let images: Vec<DisplayImage> = images.into_iter().map(|v| v.into()).collect();

    Template::render("main_view", context! {images: images})
}
