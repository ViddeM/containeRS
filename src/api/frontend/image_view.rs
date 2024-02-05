use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use sqlx::Pool;

use crate::{
    db::DB,
    services::get_images_service::{self, Tag},
};

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

#[get("/r/<image>")]
pub async fn get_image_view(db_pool: &State<Pool<DB>>, image: &str) -> Template {
    let image = match get_images_service::get_image(db_pool, image).await {
        Ok(a) => a,
        Err(e) => return Template::render("error", context! { error: e.to_string()}),
    };

    let tags = image
        .tags
        .into_iter()
        .map(|t| t.into())
        .collect::<Vec<DisplayTag>>();

    Template::render("image_view", context! {name: image.name, tags: tags })
}
