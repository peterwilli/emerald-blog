use leptos::prelude::*;
#[cfg(feature = "ssr")]
use libnostrblog::ToBech32;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::app::{post_sync::POSTS_LIST, utils::to_date_string};

use crate::app::nostr_browser as nostr;

use super::post_editor::PublishNotice;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdminDeletePost {
    pub id: String,
    pub author: String,
    pub author_hex: String,
    pub title: String,
    pub created_at: String,
}

#[component]
pub fn DeletePosts(status: ReadSignal<String>, set_status: WriteSignal<String>) -> impl IntoView {
    let (posts, set_posts) = signal(Vec::<AdminDeletePost>::new());
    let (loading, set_loading) = signal(true);
    let (deleting, set_deleting) = signal(false);
    let (reason, set_reason) = signal(String::new());
    let (refresh, set_refresh) = signal(0u32);

    Effect::new(move |_| {
        refresh.get();
        set_loading.set(true);
        leptos::task::spawn_local(async move {
            match list_deletable_posts().await {
                Ok(items) => {
                    set_posts.set(items);
                    set_status.set(String::new());
                }
                Err(error) => set_status.set(format!("Could not load posts: {error}")),
            }
            set_loading.set(false);
        });
    });

    let delete_post = move |post_id: String, title: String, author_hex: String| {
        if deleting.get_untracked() {
            return;
        }

        if !confirm_delete(&title) {
            return;
        }

        let reason_text = reason.get_untracked();
        set_deleting.set(true);
        set_status.set("Waiting for NIP-07 deletion signature...".into());
        leptos::task::spawn_local(async move {
            match nostr::publish_post_deletion(&post_id, &reason_text, &author_hex).await {
                Ok(event_id) => {
                    set_posts.update(|items| items.retain(|post| post.id != post_id));
                    set_status.set(format!("Published deletion request {event_id}."));
                }
                Err(error) => set_status.set(error),
            }
            set_deleting.set(false);
        });
    };

    view! {
        <div class="office2003-menubar">
            <button
                class="office2003-tool office2003-tool-primary"
                type="button"
                on:click=move |_| set_refresh.update(|value| *value += 1)
                disabled=loading
            >
                <span class="office2003-icon office2003-icon-refresh"></span>
                {move || if loading.get() { "Loading" } else { "Refresh" }}
            </button>
            <PublishNotice status posting=deleting />
        </div>

        <div class="admin-post-form admin-comments-panel">
            <label>
                <span>"Deletion reason"</span>
                <input
                    type="text"
                    class="admin-post-input"
                    placeholder="Optional NIP-09 reason"
                    prop:value=reason
                    on:input=move |event| set_reason.set(event_target_value(&event))
                />
            </label>

            <Show
                when=move || !loading.get()
                fallback=move || view! { <p class="admin-post-hint">"Loading posts..."</p> }
            >
                <Show
                    when=move || !posts.get().is_empty()
                    fallback=move || view! { <p class="admin-post-hint">"No posts are loaded."</p> }
                >
                    <div class="admin-comment-list">
                        <For
                            each=move || posts.get()
                            key=|post| post.id.clone()
                            children=move |post| {
                                let post_id = post.id.clone();
                                let title = post.title.clone();
                                let author_hex = post.author_hex.clone();
                                view! {
                                    <article class="admin-comment-item">
                                        <div class="admin-comment-meta">
                                            <span>{post.created_at}</span>
                                            <span>{post.author}</span>
                                        </div>
                                        <p class="admin-comment-content">{post.title}</p>
                                        <div class="admin-comment-refs">
                                            <span>"Post " {post_id.clone()}</span>
                                            <span>"Kind 1"</span>
                                        </div>
                                        <button
                                            class="office2003-tool office2003-tool-primary admin-comment-approve"
                                            type="button"
                                            disabled=deleting
                                            on:click=move |_| delete_post(post_id.clone(), title.clone(), author_hex.clone())
                                        >
                                            <span class="office2003-icon office2003-icon-delete"></span>
                                            "Delete"
                                        </button>
                                    </article>
                                }
                            }
                        />
                    </div>
                </Show>
            </Show>
        </div>
    }
}

#[server]
pub async fn list_deletable_posts() -> Result<Vec<AdminDeletePost>, ServerFnError> {
    let mut posts = POSTS_LIST
        .read()
        .iter()
        .map(|post| AdminDeletePost {
            id: post.id.to_hex(),
            author_hex: post.author.pubkey.to_hex(),
            author: post
                .author
                .pubkey
                .to_bech32()
                .unwrap_or_else(|_| post.author.pubkey.to_hex()),
            title: post.title.to_string(),
            created_at: to_date_string(post.created_at),
        })
        .collect::<Vec<_>>();

    posts.sort_by(|left, right| right.created_at.cmp(&left.created_at));

    Ok(posts)
}

fn confirm_delete(title: &str) -> bool {
    #[cfg(feature = "hydrate")]
    {
        let message = format!("Publish a NIP-09 deletion request for \"{title}\"?");
        return web_sys::window()
            .and_then(|window| window.confirm_with_message(&message).ok())
            .unwrap_or(false);
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = title;
        true
    }
}
