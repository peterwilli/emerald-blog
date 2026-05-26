use leptos::prelude::*;
#[cfg(feature = "ssr")]
use libnostrblog::{
    Blog, BlogSettings, EventId, ToBech32, blog::comments::CommentsExt, objects::comment::Comment,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use std::collections::HashSet;

#[cfg(feature = "ssr")]
use crate::app::{
    client::{OWNER_PUBKEY, get_nostr_client},
    post_sync::POSTS_LIST,
    utils::to_date_string,
};

use super::{nostr, post_editor::PublishNotice};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdminComment {
    pub id: String,
    pub author: String,
    pub content: String,
    pub created_at: String,
    pub root: Option<String>,
    pub parent: Option<String>,
}

#[component]
pub fn CommentModeration(
    status: ReadSignal<String>,
    set_status: WriteSignal<String>,
) -> impl IntoView {
    let (comments, set_comments) = signal(Vec::<AdminComment>::new());
    let (loading, set_loading) = signal(true);
    let (approving, set_approving) = signal(false);
    let (refresh, set_refresh) = signal(0u32);

    Effect::new(move |_| {
        refresh.get();
        set_loading.set(true);
        leptos::task::spawn_local(async move {
            match list_pending_comments().await {
                Ok(items) => {
                    set_comments.set(items);
                    set_status.set(String::new());
                }
                Err(error) => set_status.set(format!("Could not load comments: {error}")),
            }
            set_loading.set(false);
        });
    });

    let approve = move |comment_id: String, post_id: Option<String>| {
        if approving.get_untracked() {
            return;
        }

        set_approving.set(true);
        set_status.set("Waiting for NIP-07 approval signature...".into());
        leptos::task::spawn_local(async move {
            match nostr::publish_comment_approval(&comment_id, post_id.as_deref()).await {
                Ok(event_id) => {
                    set_status.set(format!("Approved comment with event {event_id}."));
                    set_refresh.update(|value| *value += 1);
                }
                Err(error) => set_status.set(error),
            }
            set_approving.set(false);
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
            <PublishNotice status posting=approving />
        </div>

        <div class="admin-post-form admin-comments-panel">
            {move || {
                if loading.get() {
                    view! { <p class="admin-post-hint">"Loading pending comments..."</p> }.into_any()
                } else if comments.get().is_empty() {
                    view! { <p class="admin-post-hint">"No comments are waiting for approval."</p> }.into_any()
                } else {
                    view! {
                        <div class="admin-comment-list">
                            <For
                                each=move || comments.get()
                                key=|comment| comment.id.clone()
                                children=move |comment| {
                                    let approve_id = comment.id.clone();
                                    let approve_post_id = comment.root.clone().or(comment.parent.clone());
                                    view! {
                                        <article class="admin-comment-item">
                                            <div class="admin-comment-meta">
                                                <span>{comment.created_at}</span>
                                                <span>{comment.author}</span>
                                            </div>
                                            <p class="admin-comment-content">{comment.content}</p>
                                            <div class="admin-comment-refs">
                                                <span>"Comment " {comment.id.clone()}</span>
                                                {comment.root.clone().map(|root| view! { <span>"Post " {root}</span> }.into_any())}
                                                {comment.parent.clone().map(|parent| view! { <span>"Parent " {parent}</span> }.into_any())}
                                            </div>
                                            <button
                                                class="office2003-tool office2003-tool-primary admin-comment-approve"
                                                type="button"
                                                disabled=approving
                                                on:click=move |_| approve(approve_id.clone(), approve_post_id.clone())
                                            >
                                                <span class="office2003-icon office2003-icon-mail"></span>
                                                "Approve"
                                            </button>
                                        </article>
                                    }.into_any()
                                }
                            />
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

#[server]
pub async fn list_pending_comments() -> Result<Vec<AdminComment>, ServerFnError> {
    let client = get_nostr_client().await;
    let blog = Blog::new_with_settings(
        client.clone(),
        vec![OWNER_PUBKEY],
        BlogSettings {
            require_comment_approval: true,
        },
    );
    let owner_post_ids = POSTS_LIST
        .read()
        .iter()
        .map(|post| post.id)
        .collect::<HashSet<_>>();

    if owner_post_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut seen = HashSet::new();
    let mut comments = Vec::new();

    for post_id in owner_post_ids.iter().copied() {
        let post_comments = blog
            .list_unapproved_comments(post_id)
            .await
            .map_err(|error| ServerFnError::new(error.to_string()))?;

        comments.extend(
            post_comments
                .into_iter()
                .filter(|comment| seen.insert(comment.id))
                .filter(|comment| comment_targets_owner_post(comment, &owner_post_ids))
                .map(admin_comment_from_comment),
        );
    }

    comments.sort_by(|left, right| right.created_at.cmp(&left.created_at));

    Ok(comments)
}

#[cfg(feature = "ssr")]
fn comment_targets_owner_post(comment: &Comment<'_>, owner_post_ids: &HashSet<EventId>) -> bool {
    comment
        .root
        .is_some_and(|root| owner_post_ids.contains(&root))
        || comment
            .parent
            .is_some_and(|parent| owner_post_ids.contains(&parent))
}

#[cfg(feature = "ssr")]
fn admin_comment_from_comment(comment: Comment<'_>) -> AdminComment {
    AdminComment {
        id: comment.id.to_hex(),
        author: comment
            .author
            .to_bech32()
            .unwrap_or_else(|_| comment.author.to_hex()),
        content: comment.content.into_owned(),
        created_at: to_date_string(comment.created_at),
        root: comment.root.map(|id| id.to_hex()),
        parent: comment.parent.map(|id| id.to_hex()),
    }
}
