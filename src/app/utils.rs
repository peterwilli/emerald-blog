use chrono::DateTime;
use libnostrblog::{Timestamp, objects::post::Post};
use slug::slugify;

pub fn is_dream_art_post(post: &Post<'_>) -> bool {
    post.categories
        .iter()
        .any(|category| category.trim().eq_ignore_ascii_case("dream art"))
}

pub fn to_date_string(timestamp: Timestamp) -> String {
    if let Some(datetime) = DateTime::from_timestamp(timestamp.as_u64() as i64, 0) {
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "Invalid date".to_owned()
    }
}

pub fn post_slug(post: &Post<'_>) -> String {
    format!("{}-{}", title_slug(&post.title), post.created_at.as_u64())
}

pub fn title_slug(title: &str) -> String {
    slugify(title)
}
