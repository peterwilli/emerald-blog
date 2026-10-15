use pulldown_cmark::{Event, Options, Parser, Tag, html};

pub fn contains_image(markdown: &str, image_url: &str) -> bool {
    Parser::new(markdown).any(|event| {
        matches!(event, Event::Start(Tag::Image { dest_url, .. }) if dest_url.trim() == image_url.trim())
    })
}

pub fn render_markdown(markdown: &str) -> String {
    let options = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_SMART_PUNCTUATION;
    let parser = Parser::new_ext(markdown, options).map(|event| match event {
        Event::Html(html) | Event::InlineHtml(html) => Event::Text(html),
        event => event,
    });
    let mut html = String::new();
    html::push_html(&mut html, parser);
    html
}

#[cfg(test)]
mod tests {
    use super::contains_image;

    #[test]
    fn detects_inline_and_reference_images() {
        assert!(contains_image("![Sky](/sky.png)", "/sky.png"));
        assert!(contains_image(
            "![Sky][photo]\n\n[photo]: /sky.png",
            "/sky.png"
        ));
        assert!(contains_image(
            "![Sky](/sky.png?a=1&b=2)",
            "/sky.png?a=1&b=2"
        ));
        assert!(!contains_image("[Open photo](/sky.png)", "/sky.png"));
        assert!(!contains_image("`![Sky](/sky.png)`", "/sky.png"));
        assert!(!contains_image("![Sky](/other.png)", "/sky.png"));
    }
}
