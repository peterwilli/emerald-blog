use crate::app::markdown::render_markdown;
use leptos::{html::Textarea, prelude::*};

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

    let append_markup = move |before: &'static str, after: &'static str| {
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
    };

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
        <div class="office2003-menubar">
            <button
                class="office2003-tool office2003-tool-primary"
                type="button"
                on:click=publish
                disabled=posting
            >
                <span class="office2003-icon office2003-icon-mail"></span>
                {move || if posting.get() { "Publishing" } else { "Publish" }}
            </button>
            <PublishNotice status posting />
            <span class="office2003-separator"></span>
            <button class="office2003-tool" type="button" on:click=move |_| append_markup("**bold text**", "")>
                <span class="office2003-icon office2003-icon-bold"></span>
                "Bold"
            </button>
            <button class="office2003-tool" type="button" on:click=move |_| append_markup("_italic text_", "")>
                <span class="office2003-icon office2003-icon-italic"></span>
                "Italic"
            </button>
            <button class="office2003-tool" type="button" on:click=move |_| append_markup("## Heading", "")>
                <span class="office2003-icon office2003-icon-heading"></span>
                "Heading"
            </button>
            <button class="office2003-tool" type="button" on:click=move |_| append_markup("- List item", "")>
                <span class="office2003-icon office2003-icon-list"></span>
                "List"
            </button>
            <button class="office2003-tool" type="button" on:click=move |_| append_markup("[link text](https://example.com)", "")>
                <span class="office2003-icon office2003-icon-link"></span>
                "Link"
            </button>
            <span class="office2003-separator"></span>
            <button
                class="office2003-tool office2003-tool-primary"
                type="button"
                on:click=move |_| set_source_view.update(|value| *value = !*value)
            >
                <span class="office2003-icon office2003-icon-view"></span>
                {move || if source_view.get() { "Display View" } else { "Source View" }}
            </button>
        </div>

        <div class="admin-post-form">
            <label>
                <span>"Title"</span>
                <input
                    type="text"
                    class="admin-post-input"
                    prop:value=title
                    on:input=move |event| set_title.set(event_target_value(&event))
                />
            </label>
            <label>
                <span>"Excerpt"</span>
                <input
                    type="text"
                    class="admin-post-input"
                    prop:value=excerpt
                    on:input=move |event| set_excerpt.set(event_target_value(&event))
                />
            </label>
            <label>
                <span>"Featured image URL"</span>
                <input
                    type="url"
                    class="admin-post-input"
                    prop:value=featured_image
                    on:input=move |event| set_featured_image.set(event_target_value(&event))
                />
            </label>
            <label>
                <span>"Categories"</span>
                <input
                    type="text"
                    class="admin-post-input"
                    placeholder="nostr, emerald, release"
                    prop:value=categories
                    on:input=move |event| set_categories.set(event_target_value(&event))
                />
            </label>

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
        </div>
    }
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
}
