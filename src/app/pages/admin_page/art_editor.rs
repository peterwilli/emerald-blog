use leptos::prelude::*;

use crate::app::{
    components::media_display::MediaDisplay,
    utils::{ART_CATEGORY, is_art_category},
};

use super::{
    PostDraft,
    post_editor::{CategoryInput, PublishNotice},
    publish_post,
};

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
    let (categories, set_categories) = signal(String::new());
    let (content, set_content) = signal(String::new());

    let publish = move |_| {
        let draft = PostDraft {
            title: title.get_untracked(),
            excerpt: excerpt.get_untracked(),
            featured_image: featured_image.get_untracked(),
            categories: art_categories(&categories.get_untracked()),
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
            "Published art event",
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
                <span>"Excerpt"</span>
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
                <span>"Categories"</span>
                <CategoryInput categories set_categories />
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
                {move || {
                    view! {
                        <MediaDisplay
                            src=featured_image.get()
                            alt=title.get()
                        />
                    }
                }}
                <div class="admin-art-preview-copy">
                    <b>{title}</b>
                    <span>{excerpt}</span>
                </div>
            </div>
            <p class="admin-post-hint">"Art posts are always tagged as art, plus any categories added here."</p>
        </div>
    }
}

fn art_categories(user_categories: &str) -> String {
    let mut categories = vec![ART_CATEGORY.to_owned()];

    for category in user_categories.split(',').map(str::trim) {
        if category.is_empty()
            || is_art_category(category)
            || categories
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(category))
        {
            continue;
        }

        categories.push(category.to_owned());
    }

    categories.join(", ")
}
