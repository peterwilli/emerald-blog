use crate::app::markdown::render_markdown;
use leptos::{html::Textarea, prelude::*};

#[cfg(feature = "ssr")]
use crate::app::post_sync::POSTS_LIST;

use super::{PostDraft, publish_post};

#[component]
pub fn PostEditor(
    posting: ReadSignal<bool>,
    set_posting: WriteSignal<bool>,
    status: ReadSignal<String>,
    set_status: WriteSignal<String>,
) -> impl IntoView {
    let (title, set_title) = signal(String::new());
    let (excerpt, set_excerpt) = signal(String::new());
    let (featured_image, set_featured_image) = signal(String::new());
    let (categories, set_categories) = signal(String::new());
    let (content, set_content) = signal(String::new());
    let (source_view, set_source_view) = signal(true);
    let content_ref = NodeRef::<Textarea>::new();

    view! {
        <PostEditorToolbar
            title
            excerpt
            featured_image
            categories
            content
            set_content
            posting
            set_posting
            status
            set_status
            source_view
            set_source_view
            content_ref
        />
        <PostEditorForm
            title
            set_title
            excerpt
            set_excerpt
            featured_image
            set_featured_image
            categories
            set_categories
            content
            set_content
            source_view
            content_ref
        />
    }
}

#[component]
fn PostEditorToolbar(
    title: ReadSignal<String>,
    excerpt: ReadSignal<String>,
    featured_image: ReadSignal<String>,
    categories: ReadSignal<String>,
    content: ReadSignal<String>,
    set_content: WriteSignal<String>,
    posting: ReadSignal<bool>,
    set_posting: WriteSignal<bool>,
    status: ReadSignal<String>,
    set_status: WriteSignal<String>,
    source_view: ReadSignal<bool>,
    set_source_view: WriteSignal<bool>,
    content_ref: NodeRef<Textarea>,
) -> impl IntoView {
    view! {
        <div class="office2003-menubar">
            <PublishButton
                title
                excerpt
                featured_image
                categories
                content
                posting
                set_posting
                set_status
            />
            <PublishNotice status posting />
            <span class="office2003-separator"></span>
            <MarkupButton
                icon_class="office2003-icon office2003-icon-bold"
                label="Bold"
                before="**bold text**"
                after=""
                set_content
                content_ref
            />
            <MarkupButton
                icon_class="office2003-icon office2003-icon-italic"
                label="Italic"
                before="_italic text_"
                after=""
                set_content
                content_ref
            />
            <MarkupButton
                icon_class="office2003-icon office2003-icon-heading"
                label="Heading"
                before="## Heading"
                after=""
                set_content
                content_ref
            />
            <MarkupButton
                icon_class="office2003-icon office2003-icon-list"
                label="List"
                before="- List item"
                after=""
                set_content
                content_ref
            />
            <MarkupButton
                icon_class="office2003-icon office2003-icon-link"
                label="Link"
                before="[link text](https://example.com)"
                after=""
                set_content
                content_ref
            />
            <span class="office2003-separator"></span>
            <SourceViewButton source_view set_source_view />
        </div>
    }
}

#[component]
fn PublishButton(
    title: ReadSignal<String>,
    excerpt: ReadSignal<String>,
    featured_image: ReadSignal<String>,
    categories: ReadSignal<String>,
    content: ReadSignal<String>,
    posting: ReadSignal<bool>,
    set_posting: WriteSignal<bool>,
    set_status: WriteSignal<String>,
) -> impl IntoView {
    let publish = move |_| {
        let draft = PostDraft {
            title: title.get_untracked(),
            excerpt: excerpt.get_untracked(),
            featured_image: featured_image.get_untracked(),
            categories: categories.get_untracked(),
            content: content.get_untracked(),
        };

        if draft.title.trim().is_empty() || draft.content.trim().is_empty() {
            set_status.set("Title and markdown body are required.".into());
            return;
        }

        publish_post(
            draft,
            posting,
            set_posting,
            set_status,
            "Published signed event",
        );
    };

    view! {
        <button
            class="office2003-tool office2003-tool-primary"
            type="button"
            on:click=publish
            disabled=posting
        >
            <span class="office2003-icon office2003-icon-mail"></span>
            {move || if posting.get() { "Publishing" } else { "Publish" }}
        </button>
    }
}

#[component]
fn MarkupButton(
    icon_class: &'static str,
    label: &'static str,
    before: &'static str,
    after: &'static str,
    set_content: WriteSignal<String>,
    content_ref: NodeRef<Textarea>,
) -> impl IntoView {
    view! {
        <button
            class="office2003-tool"
            type="button"
            on:click=move |_| append_markup(before, after, set_content, content_ref)
        >
            <span class=icon_class></span>
            {label}
        </button>
    }
}

#[component]
fn SourceViewButton(
    source_view: ReadSignal<bool>,
    set_source_view: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <button
            class="office2003-tool office2003-tool-primary"
            type="button"
            on:click=move |_| set_source_view.update(|value| *value = !*value)
        >
            <span class="office2003-icon office2003-icon-view"></span>
            {move || if source_view.get() { "Display View" } else { "Source View" }}
        </button>
    }
}

fn append_markup(
    before: &'static str,
    after: &'static str,
    set_content: WriteSignal<String>,
    content_ref: NodeRef<Textarea>,
) {
    set_content.update(move |content| {
        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }
        content.push_str(before);
        content.push_str(after);
    });
    if let Some(textarea) = content_ref.get() {
        _ = textarea.focus();
    }
}

#[component]
fn PostEditorForm(
    title: ReadSignal<String>,
    set_title: WriteSignal<String>,
    excerpt: ReadSignal<String>,
    set_excerpt: WriteSignal<String>,
    featured_image: ReadSignal<String>,
    set_featured_image: WriteSignal<String>,
    categories: ReadSignal<String>,
    set_categories: WriteSignal<String>,
    content: ReadSignal<String>,
    set_content: WriteSignal<String>,
    source_view: ReadSignal<bool>,
    content_ref: NodeRef<Textarea>,
) -> impl IntoView {
    view! {
        <div class="admin-post-form">
            <PostTextField label="Title" input_type="text" value=title set_value=set_title />
            <PostTextField label="Excerpt" input_type="text" value=excerpt set_value=set_excerpt />
            <PostTextField
                label="Featured image URL"
                input_type="url"
                value=featured_image
                set_value=set_featured_image
            />
            <div class="admin-post-field">
                <span>"Categories"</span>
                <CategoryInput categories set_categories />
            </div>
            <PostBodyEditor content set_content source_view content_ref />
        </div>
    }
}

#[component]
fn PostTextField(
    label: &'static str,
    input_type: &'static str,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <label>
            <span>{label}</span>
            <input
                type=input_type
                class="admin-post-input"
                prop:value=value
                on:input=move |event| set_value.set(event_target_value(&event))
            />
        </label>
    }
}

#[component]
fn PostBodyEditor(
    content: ReadSignal<String>,
    set_content: WriteSignal<String>,
    source_view: ReadSignal<bool>,
    content_ref: NodeRef<Textarea>,
) -> impl IntoView {
    view! {
        <div class="admin-post-editor">
            <Show
                when=move || source_view.get()
                fallback=move || {
                    view! {
                        <div
                            class="admin-post-preview markdown-body"
                            inner_html=move || render_markdown(&content.get())
                        ></div>
                    }
                }
            >
                <textarea
                    node_ref=content_ref
                    class="admin-post-textarea"
                    prop:value=content
                    on:input=move |event| set_content.set(event_target_value(&event))
                ></textarea>
            </Show>
        </div>
    }
}

#[component]
pub fn CategoryInput(
    categories: ReadSignal<String>,
    set_categories: WriteSignal<String>,
) -> impl IntoView {
    let (category_entry, set_category_entry) = signal(String::new());
    let (recent_categories, set_recent_categories) = signal(Vec::<String>::new());

    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            if let Ok(categories) = list_recent_categories().await {
                set_recent_categories.set(categories);
            }
        });
    });

    view! {
        <div class="admin-category-field">
            <div class="admin-category-tags" role="group" aria-label="Selected categories">
                <For
                    each=move || split_categories(&categories.get())
                    key=|category| category.to_lowercase()
                    children=move |category| view! { <CategoryChip category set_categories /> }
                />
                <input
                    type="text"
                    class="admin-category-entry"
                    aria-label="Add category"
                    placeholder="Add category"
                    prop:value=category_entry
                    on:input=move |event| set_category_entry.set(event_target_value(&event))
                    on:keydown=move |event| {
                        if event.key() == "Enter" || event.key() == "," {
                            event.prevent_default();
                            if add_category_to_signal(category_entry.get_untracked(), set_categories) {
                                set_category_entry.set(String::new());
                            }
                        }
                    }
                    on:blur=move |_| {
                        if add_category_to_signal(category_entry.get_untracked(), set_categories) {
                            set_category_entry.set(String::new());
                        }
                    }
                />
            </div>
            <RecentCategorySuggestions recent_categories set_categories />
        </div>
    }
    .into_any()
}

#[component]
fn CategoryChip(category: String, set_categories: WriteSignal<String>) -> impl IntoView {
    let remove_label = format!("Remove {category}");

    view! {
        <span class="admin-category-chip">
            <span>{category.clone()}</span>
            <button
                type="button"
                aria-label=remove_label
                on:click=move |_| remove_category_from_signal(&category, set_categories)
            >
                "x"
            </button>
        </span>
    }
    .into_any()
}

#[component]
fn RecentCategorySuggestions(
    recent_categories: ReadSignal<Vec<String>>,
    set_categories: WriteSignal<String>,
) -> impl IntoView {
    view! {
        {move || {
            let recent_categories = recent_categories.get();
            if recent_categories.is_empty() {
                ().into_any()
            } else {
                view! {
                    <div class="admin-category-recent" aria-label="Recently used categories">
                        <span>"Recently used:"</span>
                        <For
                            each=move || recent_categories.clone()
                            key=|category| category.to_lowercase()
                            children=move |category| {
                                view! { <RecentCategorySuggestion category set_categories /> }
                            }
                        />
                    </div>
                }.into_any()
            }
        }}
    }
    .into_any()
}

#[component]
fn RecentCategorySuggestion(
    category: String,
    set_categories: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <a
            href="#"
            class="admin-category-suggestion"
            on:click=move |event| {
                event.prevent_default();
                add_category_to_signal(category.clone(), set_categories);
            }
        >
            {category.clone()}
        </a>
    }
    .into_any()
}

fn add_category_to_signal(raw_category: String, set_categories: WriteSignal<String>) -> bool {
    let category = raw_category.trim().trim_matches(',').trim().to_owned();
    if category.is_empty() {
        return false;
    }

    set_categories.update(|categories| {
        let mut current = split_categories(categories);
        if !current
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(&category))
        {
            current.push(category);
        }
        *categories = current.join(", ");
    });
    true
}

fn remove_category_from_signal(category: &str, set_categories: WriteSignal<String>) {
    set_categories.update(|categories| {
        *categories = split_categories(categories)
            .into_iter()
            .filter(|existing| !existing.eq_ignore_ascii_case(category))
            .collect::<Vec<_>>()
            .join(", ");
    });
}

fn split_categories(categories: &str) -> Vec<String> {
    categories
        .split(',')
        .map(str::trim)
        .filter(|category| !category.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[server]
pub async fn list_recent_categories() -> Result<Vec<String>, ServerFnError> {
    let mut categories = Vec::new();

    for post in POSTS_LIST.read().iter() {
        for category in &post.categories {
            let category = category.trim();
            if category.is_empty()
                || crate::app::utils::is_art_category(category)
                || categories
                    .iter()
                    .any(|existing: &String| existing.eq_ignore_ascii_case(category))
            {
                continue;
            }

            categories.push(category.to_owned());
        }
    }

    Ok(categories)
}

#[component]
pub fn PublishNotice(status: ReadSignal<String>, posting: ReadSignal<bool>) -> impl IntoView {
    view! {
        {move || {
            if status.get().is_empty() {
                ().into_any()
            } else {
                view! {
                    <div class="admin-publish-notice" role="status" aria-live="polite">
                        <span class=move || {
                            if posting.get() {
                                "admin-publish-spinner"
                            } else {
                                "hidden"
                            }
                        }></span>
                        <span>{status}</span>
                    </div>
                }.into_any()
            }
        }}
    }
    .into_any()
}
