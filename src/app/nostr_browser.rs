#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub struct PostDraft {
    pub title: String,
    pub excerpt: String,
    pub featured_image: String,
    pub categories: String,
    pub content: String,
}

#[cfg(feature = "hydrate")]
use libnostrblog::{
    Event, EventBuilder, EventId, JsonUtil, Keys, Kind, PublicKey, Tag, TagStandard, Timestamp,
    ToBech32, UnsignedEvent, nips::nip10::Marker,
};

#[cfg(feature = "hydrate")]
use crate::app::client::{OWNER_PUBKEY, get_nostr_client};

#[cfg(feature = "hydrate")]
const LOCAL_COMMENT_SECRET_KEY: &str = "emerald.comment.secret_key";

#[cfg(feature = "hydrate")]
pub async fn publish_nostr_post(draft: PostDraft) -> Result<String, String> {
    let public_key = get_nostr_public_key().await?;
    let event = sign_unsigned_event(unsigned_event(
        public_key,
        Kind::TextNote,
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
        tag(["K", "1"])?,
        tag(["e", post_id, "", "reply"])?,
        tag(["k", "1"])?,
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
    let tags = vec![Tag::event(post_id), tag(["k", "1"])?];
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
    let mut tags = vec![tag(["title", draft.title.trim()])?];
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
            kind: Kind::TextNote,
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
            kind: Kind::TextNote,
            uppercase: false,
        }),
        Tag::public_key(OWNER_PUBKEY),
    ])
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
