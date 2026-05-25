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
pub async fn publish_nostr_post(draft: PostDraft) -> Result<String, String> {
    let draft_object = js_sys::Object::new();
    set_property(&draft_object, "title", &draft.title)?;
    set_property(&draft_object, "excerpt", &draft.excerpt)?;
    set_property(&draft_object, "featured_image", &draft.featured_image)?;
    set_property(&draft_object, "categories", &draft.categories)?;
    set_property(&draft_object, "content", &draft.content)?;

    let args = js_sys::Array::new();
    args.push(&draft_object);
    call_core_promise("publishNostrPost", args).await
}

#[cfg(feature = "hydrate")]
pub async fn publish_comment(post_id: &str, content: &str) -> Result<String, String> {
    let args = js_sys::Array::new();
    args.push(&post_id.into());
    args.push(&content.into());
    call_core_promise("publishComment", args).await
}

#[cfg(feature = "hydrate")]
pub fn has_nostr_signer() -> bool {
    call_core_bool("hasNostrSigner").unwrap_or(false)
}

#[cfg(feature = "hydrate")]
pub async fn publish_comment_approval(comment_id: &str) -> Result<String, String> {
    let args = js_sys::Array::new();
    args.push(&comment_id.into());
    call_core_promise("publishCommentApproval", args).await
}

#[cfg(feature = "hydrate")]
pub async fn publish_post_deletion(
    post_id: &str,
    reason: &str,
    expected_author: &str,
) -> Result<String, String> {
    let args = js_sys::Array::new();
    args.push(&post_id.into());
    args.push(&reason.into());
    args.push(&expected_author.into());
    call_core_promise("publishPostDeletion", args).await
}

#[cfg(feature = "hydrate")]
async fn call_core_promise(name: &str, args: js_sys::Array) -> Result<String, String> {
    use js_sys::Promise;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let (core, function) = core_function(name)?;
    let result = function
        .apply(&core, &args)
        .map_err(js_error)?
        .dyn_into::<Promise>()
        .map_err(|_| format!("window.EmeraldBlogCore.{name} did not return a Promise."))?;

    JsFuture::from(result)
        .await
        .map_err(js_error)?
        .as_string()
        .ok_or_else(|| format!("window.EmeraldBlogCore.{name} did not return a string."))
}

#[cfg(feature = "hydrate")]
fn call_core_bool(name: &str) -> Result<bool, String> {
    let (core, function) = core_function(name)?;
    function
        .call0(&core)
        .map_err(js_error)?
        .as_bool()
        .ok_or_else(|| format!("window.EmeraldBlogCore.{name} did not return a boolean."))
}

#[cfg(feature = "hydrate")]
fn core_function(name: &str) -> Result<(wasm_bindgen::JsValue, js_sys::Function), String> {
    use wasm_bindgen::{JsCast, JsValue};

    let window = web_sys::window().ok_or_else(|| "Browser window is not available.".to_string())?;
    let core = js_sys::Reflect::get(&window, &JsValue::from_str("EmeraldBlogCore"))
        .map_err(|_| "Unable to read window.EmeraldBlogCore.".to_string())?;
    if core.is_undefined() || core.is_null() {
        return Err("window.EmeraldBlogCore is not loaded.".into());
    }

    let function = js_sys::Reflect::get(&core, &JsValue::from_str(name))
        .map_err(|_| format!("window.EmeraldBlogCore.{name} is not available."))?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| format!("window.EmeraldBlogCore.{name} is not callable."))?;

    Ok((core, function))
}

#[cfg(feature = "hydrate")]
fn set_property(object: &js_sys::Object, name: &str, value: &str) -> Result<(), String> {
    use wasm_bindgen::JsValue;

    js_sys::Reflect::set(object, &JsValue::from_str(name), &JsValue::from_str(value))
        .map(|_| ())
        .map_err(js_error)
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
pub fn has_nostr_signer() -> bool {
    false
}

#[cfg(not(feature = "hydrate"))]
pub async fn publish_comment_approval(_comment_id: &str) -> Result<String, String> {
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
