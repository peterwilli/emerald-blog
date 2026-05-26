use libnostrblog::{Timestamp, objects::post::Post};

pub fn is_dream_art_post(post: &Post<'_>) -> bool {
    post.categories
        .iter()
        .any(|category| category.trim().eq_ignore_ascii_case("dream art"))
}

pub fn to_date_string(timestamp: Timestamp) -> String {
    let seconds = timestamp.as_u64();
    let days = seconds / 86_400;
    let seconds_of_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days as i64);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
}

pub fn post_slug(post: &Post<'_>) -> String {
    format!("{}-{}", title_slug(&post.title), post.created_at.as_u64())
}

pub fn title_slug(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    let mut needs_separator = false;

    for character in title.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            if needs_separator && !slug.is_empty() {
                slug.push('-');
            }
            slug.push(character);
            needs_separator = false;
        } else if !slug.is_empty() {
            needs_separator = true;
        }
    }

    if slug.is_empty() {
        "post".to_owned()
    } else {
        slug
    }
}

fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let days = days_since_epoch + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + if month <= 2 { 1 } else { 0 };

    (year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use libnostrblog::Timestamp;

    #[test]
    fn formats_unix_timestamps_as_utc() {
        assert_eq!(to_date_string(Timestamp::from(0)), "1970-01-01 00:00:00");
        assert_eq!(
            to_date_string(Timestamp::from(1_704_067_200)),
            "2024-01-01 00:00:00"
        );
    }
}
