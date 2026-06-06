#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub struct PostDraft {
    pub title: String,
    pub excerpt: String,
    pub featured_image: String,
    pub categories: String,
    pub content: String,
}

#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub struct MigrationPreview {
    pub source_id: String,
    pub title: String,
    pub created_at: u64,
    pub categories: Vec<String>,
}

#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub struct MigrationPublishResult {
    pub source_id: String,
    pub event_id: String,
    pub title: String,
}

#[cfg(feature = "hydrate")]
use libnostrblog::{
    Event, EventBuilder, EventId, JsonUtil, Keys, Kind, PublicKey, Tag, TagStandard, Timestamp,
    ToBech32, UnsignedEvent, nips::nip10::Marker,
};

#[cfg(feature = "hydrate")]
use crate::app::client::{OWNER_PUBKEY, get_nostr_client};
#[cfg(feature = "hydrate")]
use crate::app::utils::title_slug;

#[cfg(feature = "hydrate")]
const LOCAL_COMMENT_SECRET_KEY: &str = "emerald.comment.secret_key";

#[cfg(feature = "hydrate")]
pub async fn publish_nostr_post(draft: PostDraft) -> Result<String, String> {
    let public_key = get_nostr_public_key().await?;
    let event = sign_unsigned_event(unsigned_event(
        public_key,
        Kind::LongFormTextNote,
        build_post_tags(&draft)?,
        draft.content,
    ))
    .await?;

    publish_signed_event(event).await
}

#[cfg(feature = "hydrate")]
pub async fn publish_comment(post_id: &str, content: &str) -> Result<String, String> {
    let body = content.trim();
    if body.is_empty() {
        return Err("Write a comment before publishing.".into());
    }

    let public_key = get_nostr_public_key().await?;
    let tags = vec![
        tag(["E", post_id, "", "root"])?,
        tag(["K", long_form_kind()])?,
        tag(["e", post_id, "", "reply"])?,
        tag(["k", long_form_kind()])?,
    ];
    let event = sign_unsigned_event(unsigned_event(
        public_key,
        Kind::Comment,
        tags,
        body.to_owned(),
    ))
    .await?;

    publish_signed_event(event).await
}

#[cfg(feature = "hydrate")]
pub async fn publish_local_comment(
    post_id: &str,
    display_name: &str,
    content: &str,
) -> Result<String, String> {
    let post_id =
        EventId::from_hex(post_id).map_err(|error| format!("Invalid post id: {error}"))?;
    let display_name = clean_display_name(display_name)
        .ok_or_else(|| "Choose a display name before publishing.".to_string())?;
    let content = content.trim();

    if content.is_empty() {
        return Err("Write a comment before publishing.".into());
    }

    let keys = local_comment_keys()?;
    let event = local_comment_builder(post_id, anonymous_comment_content(&display_name, content))
        .sign_with_keys(&keys)
        .map_err(|error| format!("Unable to sign comment: {error}"))?;

    publish_signed_event(event).await
}

#[cfg(feature = "hydrate")]
pub fn has_nostr_signer() -> bool {
    nostr_object().is_ok()
}

#[cfg(feature = "hydrate")]
pub async fn get_nostr_public_key() -> Result<PublicKey, String> {
    use js_sys::Promise;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let nostr = nostr_object()?;
    let get_public_key = nostr_function(&nostr, "getPublicKey")?;
    let public_key = get_public_key
        .call0(&nostr)
        .map_err(js_error)?
        .dyn_into::<Promise>()
        .map_err(|_| "window.nostr.getPublicKey did not return a Promise.".to_string())?;
    let public_key = JsFuture::from(public_key)
        .await
        .map_err(js_error)?
        .as_string()
        .ok_or_else(|| "window.nostr.getPublicKey did not return a string.".to_string())?;

    PublicKey::parse(public_key.trim())
        .map_err(|error| format!("Invalid NIP-07 public key: {error}"))
}

#[cfg(feature = "hydrate")]
pub async fn publish_comment_approval(
    comment_id: &str,
    post_id: Option<&str>,
) -> Result<String, String> {
    let comment_id = EventId::parse(comment_id.trim())
        .map_err(|error| format!("Invalid comment event id: {error}"))?;
    let public_key = get_nostr_public_key().await?;
    let mut tags = vec![
        Tag::event(comment_id),
        tag(["L", "libnostrblog/moderation"])?,
        tag(["l", "approved", "libnostrblog/moderation"])?,
    ];

    if let Some(post_id) = post_id.map(str::trim).filter(|post_id| !post_id.is_empty()) {
        EventId::parse(post_id).map_err(|error| format!("Invalid post event id: {error}"))?;
        tags.push(tag(["E", post_id])?);
    }

    let event =
        sign_unsigned_event(unsigned_event(public_key, Kind::Custom(1985), tags, "")).await?;

    publish_signed_event(event).await
}

#[cfg(feature = "hydrate")]
pub async fn publish_post_deletion(
    post_id: &str,
    reason: &str,
    expected_author: &str,
) -> Result<String, String> {
    let post_id = post_id.trim();
    if post_id.is_empty() {
        return Err("Choose a post to delete.".into());
    }

    let post_id =
        EventId::parse(post_id).map_err(|error| format!("Invalid post event id: {error}"))?;
    let public_key = get_nostr_public_key().await?;
    let tags = vec![Tag::event(post_id), tag(["k", long_form_kind()])?];
    let event = sign_unsigned_event(unsigned_event(
        public_key,
        Kind::EventDeletion,
        tags,
        reason.trim().to_owned(),
    ))
    .await?;

    assert_signed_pubkey(&event, expected_author)?;
    publish_signed_event(event).await
}

#[cfg(feature = "hydrate")]
pub async fn preview_text_note_migration(raw: &str) -> Result<Vec<MigrationPreview>, String> {
    Ok(parse_migration_events(raw)?
        .into_iter()
        .map(|event| MigrationPreview {
            source_id: event.id.to_hex(),
            title: migration_title(&event),
            created_at: event.created_at.as_u64(),
            categories: migration_categories(&event),
        })
        .collect())
}

#[cfg(feature = "hydrate")]
pub async fn publish_text_note_migration(raw: &str) -> Result<Vec<MigrationPublishResult>, String> {
    let public_key = get_nostr_public_key().await?;
    let events = parse_migration_events(raw)?;
    let source_ids = events.iter().map(|event| event.id).collect::<Vec<_>>();
    let mut published = Vec::with_capacity(events.len());

    for event in events {
        if event.pubkey != public_key {
            return Err(format!(
                "Source event {} belongs to {} but the signer is {}.",
                event.id.to_hex(),
                event.pubkey.to_hex(),
                public_key.to_hex()
            ));
        }

        let title = migration_title(&event);
        let signed = sign_unsigned_event(unsigned_event(
            public_key,
            Kind::LongFormTextNote,
            migration_tags(&event)?,
            event.content.clone(),
        ))
        .await?;
        let event_id = publish_signed_event(signed).await?;

        published.push(MigrationPublishResult {
            source_id: event.id.to_hex(),
            event_id,
            title,
        });
    }

    publish_migrated_text_note_deletion(public_key, &source_ids).await?;

    Ok(published)
}

#[cfg(feature = "hydrate")]
pub async fn sign_unsigned_event(unsigned: UnsignedEvent) -> Result<Event, String> {
    use js_sys::{JSON, Promise};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let intended = unsigned.clone();
    let nostr = nostr_object()?;
    let sign_event = nostr_function(&nostr, "signEvent")?;
    let unsigned_json = unsigned
        .try_as_json()
        .map_err(|error| format!("Unable to serialize unsigned event: {error:?}"))?;
    let unsigned_value = JSON::parse(&unsigned_json).map_err(js_error)?;
    let signed = sign_event
        .call1(&nostr, &unsigned_value)
        .map_err(js_error)?
        .dyn_into::<Promise>()
        .map_err(|_| "window.nostr.signEvent did not return a Promise.".to_string())?;
    let signed = JsFuture::from(signed).await.map_err(js_error)?;
    let signed_json = JSON::stringify(&signed)
        .map_err(js_error)?
        .as_string()
        .ok_or_else(|| {
            "NIP-07 signer returned an event that could not be serialized.".to_string()
        })?;
    let event = Event::from_json(signed_json)
        .map_err(|error| format!("NIP-07 signer returned an invalid event: {error:?}"))?;

    validate_signed_event(&intended, &event)?;
    event
        .verify()
        .map_err(|error| format!("NIP-07 signer returned an unverifiable event: {error}"))?;

    Ok(event)
}

#[cfg(feature = "hydrate")]
fn unsigned_event(
    public_key: PublicKey,
    kind: Kind,
    tags: Vec<Tag>,
    content: impl Into<String>,
) -> UnsignedEvent {
    UnsignedEvent::new(public_key, Timestamp::now(), kind, tags, content)
}

#[cfg(feature = "hydrate")]
async fn publish_signed_event(event: Event) -> Result<String, String> {
    let output = get_nostr_client()
        .await
        .send_event(&event)
        .await
        .map_err(|error| format!("Relay publish failed: {error}"))?;

    Ok(output
        .id()
        .to_bech32()
        .unwrap_or_else(|_| output.id().to_hex()))
}

#[cfg(feature = "hydrate")]
async fn publish_migrated_text_note_deletion(
    public_key: PublicKey,
    source_ids: &[EventId],
) -> Result<String, String> {
    let mut tags = source_ids
        .iter()
        .copied()
        .map(Tag::event)
        .collect::<Vec<_>>();
    tags.push(tag(["k", text_note_kind()])?);
    let event = sign_unsigned_event(unsigned_event(
        public_key,
        Kind::EventDeletion,
        tags,
        "Migrated to long-form posts.",
    ))
    .await?;

    publish_signed_event(event).await
}

#[cfg(feature = "hydrate")]
fn validate_signed_event(unsigned: &UnsignedEvent, event: &Event) -> Result<(), String> {
    if event.pubkey != unsigned.pubkey {
        return Err(format!(
            "NIP-07 signer returned pubkey {} instead of {}.",
            event.pubkey.to_hex(),
            unsigned.pubkey.to_hex()
        ));
    }
    if event.created_at != unsigned.created_at {
        return Err("NIP-07 signer changed the event timestamp.".into());
    }
    if event.kind != unsigned.kind {
        return Err("NIP-07 signer changed the event kind.".into());
    }
    if event.tags != unsigned.tags {
        return Err("NIP-07 signer changed the event tags.".into());
    }
    if event.content != unsigned.content {
        return Err("NIP-07 signer changed the event content.".into());
    }

    Ok(())
}

#[cfg(feature = "hydrate")]
fn assert_signed_pubkey(event: &Event, expected_pubkey: &str) -> Result<(), String> {
    let expected = expected_pubkey.trim();
    if expected.is_empty() {
        return Ok(());
    }

    let expected = PublicKey::parse(expected)
        .map_err(|error| format!("Invalid expected author public key: {error}"))?;
    if event.pubkey != expected {
        return Err(format!(
            "NIP-07 signer pubkey {} cannot delete posts by {}.",
            event.pubkey.to_hex(),
            expected.to_hex()
        ));
    }

    Ok(())
}

#[cfg(feature = "hydrate")]
fn build_post_tags(draft: &PostDraft) -> Result<Vec<Tag>, String> {
    let published_at = Timestamp::now().as_u64().to_string();
    let identifier = format!("{}-{published_at}", title_slug(&draft.title));
    let mut tags = vec![
        tag(["title", draft.title.trim()])?,
        tag(["d", identifier.as_str()])?,
        tag(["published_at", published_at.as_str()])?,
    ];
    let excerpt = draft.excerpt.trim();
    let featured_image = draft.featured_image.trim();

    if !excerpt.is_empty() {
        tags.push(tag(["summary", excerpt])?);
    }
    if !featured_image.is_empty() {
        tags.push(tag(["image", featured_image])?);
    }

    for category in draft
        .categories
        .split(',')
        .map(str::trim)
        .filter(|c| !c.is_empty())
    {
        tags.push(tag(["t", category])?);
    }

    Ok(tags)
}

#[cfg(feature = "hydrate")]
fn local_comment_builder(post_id: EventId, content: String) -> EventBuilder {
    EventBuilder::new(Kind::Comment, content).tags([
        Tag::from_standardized_without_cell(TagStandard::Event {
            event_id: post_id,
            relay_url: None,
            marker: Some(Marker::Root),
            public_key: Some(OWNER_PUBKEY),
            uppercase: true,
        }),
        Tag::from_standardized_without_cell(TagStandard::Kind {
            kind: Kind::LongFormTextNote,
            uppercase: true,
        }),
        Tag::from_standardized_without_cell(TagStandard::PublicKey {
            public_key: OWNER_PUBKEY,
            relay_url: None,
            alias: None,
            uppercase: true,
        }),
        Tag::from_standardized_without_cell(TagStandard::Event {
            event_id: post_id,
            relay_url: None,
            marker: Some(Marker::Reply),
            public_key: Some(OWNER_PUBKEY),
            uppercase: false,
        }),
        Tag::from_standardized_without_cell(TagStandard::Kind {
            kind: Kind::LongFormTextNote,
            uppercase: false,
        }),
        Tag::public_key(OWNER_PUBKEY),
    ])
}

#[cfg(feature = "hydrate")]
fn parse_migration_events(raw: &str) -> Result<Vec<Event>, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("Paste raw kind 1 events before parsing.".into());
    }

    let mut events = Vec::new();
    let values = serde_json::Deserializer::from_str(raw).into_iter::<serde_json::Value>();

    for value in values {
        let value = value.map_err(|error| format!("Invalid raw Nostr JSON: {error}"))?;
        match value {
            serde_json::Value::Array(items) => {
                for item in items {
                    push_migration_event(item, &mut events)?;
                }
            }
            item => push_migration_event(item, &mut events)?,
        }
    }

    if events.is_empty() {
        return Err("No kind 1 events were found in the pasted raw output.".into());
    }

    Ok(events)
}

#[cfg(feature = "hydrate")]
fn push_migration_event(value: serde_json::Value, events: &mut Vec<Event>) -> Result<(), String> {
    let event = Event::from_json(value.to_string())
        .map_err(|error| format!("Invalid Nostr event JSON: {error:?}"))?;
    if event.kind == Kind::TextNote {
        events.push(event);
    }
    Ok(())
}

#[cfg(feature = "hydrate")]
fn migration_tags(event: &Event) -> Result<Vec<Tag>, String> {
    let mut tags = event.tags.iter().cloned().collect::<Vec<_>>();
    if !has_tag(&tags, "d") {
        tags.push(tag(["d", event.id.to_hex().as_str()])?);
    }
    if !has_tag(&tags, "published_at") {
        tags.push(tag([
            "published_at",
            event.created_at.as_u64().to_string().as_str(),
        ])?);
    }
    Ok(tags)
}

#[cfg(feature = "hydrate")]
fn has_tag(tags: &[Tag], key: &str) -> bool {
    tags.iter().any(|tag| {
        let [tag_key, ..] = tag.as_slice() else {
            return false;
        };
        tag_key == key
    })
}

#[cfg(feature = "hydrate")]
fn migration_title(event: &Event) -> String {
    tag_values(event, "title")
        .next()
        .filter(|title| !title.trim().is_empty())
        .unwrap_or("Untitled")
        .to_owned()
}

#[cfg(feature = "hydrate")]
fn migration_categories(event: &Event) -> Vec<String> {
    tag_values(event, "t").map(ToOwned::to_owned).collect()
}

#[cfg(feature = "hydrate")]
fn tag_values<'a>(event: &'a Event, key: &'a str) -> impl Iterator<Item = &'a str> {
    event.tags.iter().filter_map(move |tag| {
        let [tag_key, value, ..] = tag.as_slice() else {
            return None;
        };
        (tag_key == key).then_some(value.as_str())
    })
}

#[cfg(feature = "hydrate")]
fn long_form_kind() -> &'static str {
    "30023"
}

#[cfg(feature = "hydrate")]
fn text_note_kind() -> &'static str {
    "1"
}

#[cfg(feature = "hydrate")]
fn local_comment_keys() -> Result<Keys, String> {
    let storage = web_sys::window()
        .ok_or_else(|| "Browser window is not available.".to_string())?
        .local_storage()
        .map_err(|_| "Unable to read localStorage.".to_string())?
        .ok_or_else(|| "localStorage is not available.".to_string())?;

    if let Some(secret_key) = storage
        .get_item(LOCAL_COMMENT_SECRET_KEY)
        .map_err(|_| "Unable to read local comment key.".to_string())?
    {
        return Keys::parse(secret_key.trim())
            .map_err(|error| format!("Stored local comment key is invalid: {error}"));
    }

    let keys = Keys::generate();
    storage
        .set_item(LOCAL_COMMENT_SECRET_KEY, &keys.secret_key().to_secret_hex())
        .map_err(|_| "Unable to store local comment key.".to_string())?;

    Ok(keys)
}

#[cfg(feature = "hydrate")]
fn anonymous_comment_content(display_name: &str, content: &str) -> String {
    format!(
        "---\ndisplay_name: \"{}\"\n---\n\n{}",
        display_name.replace('\\', "\\\\").replace('"', "\\\""),
        content.trim()
    )
}

#[cfg(feature = "hydrate")]
fn clean_display_name(display_name: &str) -> Option<String> {
    let cleaned = display_name.trim().replace(['\r', '\n'], " ");
    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

    (!cleaned.is_empty()).then(|| cleaned.chars().take(64).collect())
}

#[cfg(feature = "hydrate")]
fn tag<const N: usize>(values: [&str; N]) -> Result<Tag, String> {
    Tag::parse(values).map_err(|error| format!("Invalid Nostr tag: {error}"))
}

#[cfg(feature = "hydrate")]
fn nostr_object() -> Result<wasm_bindgen::JsValue, String> {
    use wasm_bindgen::JsValue;

    let window = web_sys::window().ok_or_else(|| "Browser window is not available.".to_string())?;
    let nostr = js_sys::Reflect::get(&window, &JsValue::from_str("nostr"))
        .map_err(|_| "Unable to read window.nostr.".to_string())?;
    if nostr.is_undefined() || nostr.is_null() {
        return Err("NIP-07 extension not found. Install or unlock a window.nostr signer.".into());
    }

    Ok(nostr)
}

#[cfg(feature = "hydrate")]
fn nostr_function(nostr: &wasm_bindgen::JsValue, name: &str) -> Result<js_sys::Function, String> {
    use wasm_bindgen::{JsCast, JsValue};

    js_sys::Reflect::get(nostr, &JsValue::from_str(name))
        .map_err(|_| format!("window.nostr.{name} is not available."))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| format!("window.nostr.{name} is not callable."))
}

#[cfg(feature = "hydrate")]
fn js_error(error: wasm_bindgen::JsValue) -> String {
    use wasm_bindgen::JsValue;

    if let Some(message) = error.as_string() {
        return message;
    }

    js_sys::Reflect::get(&error, &JsValue::from_str("message"))
        .ok()
        .and_then(|message| message.as_string())
        .filter(|message| !message.is_empty())
        .unwrap_or_else(|| "NIP-07 operation failed.".into())
}

#[cfg(not(feature = "hydrate"))]
pub async fn publish_nostr_post(_draft: PostDraft) -> Result<String, String> {
    Err("Publishing is only available in the hydrated browser app.".into())
}

#[cfg(not(feature = "hydrate"))]
pub async fn publish_comment(_post_id: &str, _content: &str) -> Result<String, String> {
    Err("Commenting is only available in the hydrated browser app.".into())
}

#[cfg(not(feature = "hydrate"))]
pub async fn publish_local_comment(
    _post_id: &str,
    _display_name: &str,
    _content: &str,
) -> Result<String, String> {
    Err("Commenting is only available in the hydrated browser app.".into())
}

#[cfg(not(feature = "hydrate"))]
pub fn has_nostr_signer() -> bool {
    false
}

#[cfg(not(feature = "hydrate"))]
pub async fn publish_comment_approval(
    _comment_id: &str,
    _post_id: Option<&str>,
) -> Result<String, String> {
    Err("Comment approval is only available in the hydrated browser app.".into())
}

#[cfg(not(feature = "hydrate"))]
pub async fn publish_post_deletion(
    _post_id: &str,
    _reason: &str,
    _expected_author: &str,
) -> Result<String, String> {
    Err("Post deletion is only available in the hydrated browser app.".into())
}

#[cfg(not(feature = "hydrate"))]
pub async fn preview_text_note_migration(_raw: &str) -> Result<Vec<MigrationPreview>, String> {
    Err("Migration is only available in the hydrated browser app.".into())
}

#[cfg(not(feature = "hydrate"))]
pub async fn publish_text_note_migration(
    _raw: &str,
) -> Result<Vec<MigrationPublishResult>, String> {
    Err("Migration is only available in the hydrated browser app.".into())
}
