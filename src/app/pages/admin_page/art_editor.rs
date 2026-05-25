use leptos::prelude::*;

use super::{PostDraft, post_editor::PublishNotice, publish_post};

#[component]
pub fn ArtEditor(
    posting: ReadSignal<bool>,
    set_posting: WriteSignal<bool>,
    status: ReadSignal<String>,
    set_status: WriteSignal<String>,
) -> impl IntoView {
    let (title, set_title) = signal(String::new());
    let (excerpt, set_excerpt) = signal(String::new());
    let (featured_image, set_featured_image) = signal(String::new());
    let (content, set_content) = signal(String::new());

    let publish = move |_| {
        let draft = PostDraft {
            title: title.get_untracked(),
            excerpt: excerpt.get_untracked(),
            featured_image: featured_image.get_untracked(),
            categories: "dream art".into(),
            content: content.get_untracked(),
        };

        if draft.title.trim().is_empty() || draft.featured_image.trim().is_empty() {
            set_status.set("Art title and image URL are required.".into());
            return;
        }

        publish_post(
            draft,
            posting,
            set_posting,
            set_status,
            "Published dream art event",
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
                {move || if posting.get() { "Publishing" } else { "Publish Art" }}
            </button>
            <PublishNotice status posting />
        </div>

        <div class="admin-post-form">
            <label>
                <span>"Art title"</span>
                <input
                    type="text"
                    class="admin-post-input"
                    prop:value=title
                    on:input=move |event| set_title.set(event_target_value(&event))
                />
            </label>
            <label>
                <span>"Slider caption"</span>
                <input
                    type="text"
                    class="admin-post-input"
                    prop:value=excerpt
                    on:input=move |event| set_excerpt.set(event_target_value(&event))
                />
            </label>
            <label>
                <span>"Art image URL"</span>
                <input
                    type="url"
                    class="admin-post-input"
                    prop:value=featured_image
                    on:input=move |event| set_featured_image.set(event_target_value(&event))
                />
            </label>
            <label>
                <span>"Post body"</span>
                <textarea
                    class="admin-post-textarea admin-post-textarea-compact"
                    prop:value=content
                    on:input=move |event| set_content.set(event_target_value(&event))
                ></textarea>
            </label>
            <div class="admin-art-preview glass rounded-xl overflow-hidden">
                <img src=move || featured_image.get() alt="" />
                <div>
                    <b>{title}</b>
                    <span>{excerpt}</span>
                </div>
            </div>
            <p class="admin-post-hint">"Art posts are tagged as dream art for the homepage slider."</p>
        </div>
    }
}
