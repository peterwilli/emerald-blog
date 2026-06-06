use libnostrblog::{Timestamp, objects::post::Post};

pub const ART_CATEGORY: &str = "art";
pub const LEGACY_ART_CATEGORY: &str = "dream art";

pub fn is_art_post(post: &Post<'_>) -> bool {
    post.categories
        .iter()
        .any(|category| is_art_category(category))
}

pub fn is_art_category(category: &str) -> bool {
    let category = category.trim();
    category.eq_ignore_ascii_case(ART_CATEGORY)
        || category.eq_ignore_ascii_case(LEGACY_ART_CATEGORY)
}

pub fn post_has_category(post: &Post<'_>, selected_category: &str) -> bool {
    post.categories.iter().any(|category| {
        category
            .trim()
            .eq_ignore_ascii_case(selected_category.trim())
    })
}

pub fn is_video_media_url(url: &str) -> bool {
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .trim()
        .to_ascii_lowercase();

    matches!(
        path.rsplit('.').next(),
        Some("webm" | "mp4" | "mov" | "ogg" | "ogv")
    )
}

pub fn blossom_thumb_url(url: &str) -> String {
    let trimmed = url.trim();
    let base = trimmed.split(['?', '#']).next().unwrap_or(trimmed);

    if base.contains("/thumb/") {
        return base.to_owned();
    }

    let Some((prefix, filename)) = base.rsplit_once('/') else {
        return format!("/thumb/{base}");
    };

    let origin = if let Some((scheme_host, _)) = prefix.split_once("://") {
        let scheme_end = scheme_host.len() + 3;
        let after_scheme = &prefix[scheme_end..];
        match after_scheme.find('/') {
            Some(path_start) => &prefix[..scheme_end + path_start],
            None => prefix,
        }
    } else if prefix.is_empty() {
        ""
    } else {
        prefix
    };

    format!("{origin}/thumb/{filename}")
}

pub fn category_href(base_path: &str, category: &str) -> String {
    format!(
        "{}/{}",
        base_path.trim_end_matches('/'),
        encode_path_segment(category)
    )
}

pub fn encode_path_segment(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

pub fn decode_path_segment(value: &str) -> String {
    let mut decoded = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
            {
                decoded.push(high * 16 + low);
                index += 3;
                continue;
            }
        }

        decoded.push(bytes[index]);
        index += 1;
    }

    String::from_utf8(decoded).unwrap_or_else(|_| value.to_owned())
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
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

    #[test]
    fn encodes_path_segments() {
        assert_eq!(encode_path_segment("dream art"), "dream%20art");
        assert_eq!(encode_path_segment("a/b?c"), "a%2Fb%3Fc");
    }

    #[test]
    fn decodes_path_segments() {
        assert_eq!(decode_path_segment("dream%20art"), "dream art");
        assert_eq!(decode_path_segment("a%2Fb%3Fc"), "a/b?c");
    }

    #[test]
    fn detects_video_media_urls() {
        assert!(is_video_media_url("https://example.com/art.webm"));
        assert!(is_video_media_url("https://example.com/art.MP4?download=1"));
        assert!(!is_video_media_url("https://example.com/art.webp"));
        assert!(!is_video_media_url("https://example.com/webm-art"));
    }

    #[test]
    fn builds_blossom_thumb_urls() {
        assert_eq!(
            blossom_thumb_url("https://media.emerald.show/41ab5.webm"),
            "https://media.emerald.show/thumb/41ab5.webm"
        );
        assert_eq!(
            blossom_thumb_url("https://media.emerald.show/files/41ab5.webm?download=1"),
            "https://media.emerald.show/thumb/41ab5.webm"
        );
        assert_eq!(
            blossom_thumb_url("https://media.emerald.show/thumb/41ab5.webm"),
            "https://media.emerald.show/thumb/41ab5.webm"
        );
    }
}
