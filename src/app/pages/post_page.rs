use crate::app::{
    AppState,
    markdown::render_markdown,
    nostr_browser,
    utils::{post_slug, title_slug, to_date_string},
};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;
#[cfg(feature = "ssr")]
use libnostrblog::{
    Blog, BlogSettings, EventBuilder, EventId, Filter, Kind, PublicKey, Tag, TagStandard, ToBech32,
    blog::comments::CommentsExt, nips::nip10::Marker, objects::comment::Comment,
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use std::{collections::HashMap, time::Duration};

#[cfg(feature = "ssr")]
use crate::app::client::{OWNER_PUBKEY, get_anonymous_comment_client, get_nostr_client};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApprovedComment {
    pub id: String,
    pub author: String,
    pub display_name: Option<String>,
    pub content: String,
    pub created_at: String,
}

#[component]
pub fn PostPage() -> impl IntoView {
    let params = use_params_map();
    let route_slug = params.read_untracked().get("slug").unwrap_or_default();
    let state = use_context::<AppState>().unwrap();
    let posts = state.posts;

    view! {
        <div class="glass rounded-xl p-6 transition-all duration-300 post-card">
            <Suspense fallback=move || {
                view! { <p>"Loading..."</p> }
            }>
                {move || {
                    posts
                        .get()
                        .map(|posts| {
                            let posts = posts.read();
                            posts
                                .iter()
                                .find(|&post| {
                                    post_slug(post) == route_slug || title_slug(&post.title) == route_slug
                                })
                                .cloned()
                                .map(|post| {
                                    let route_slug = route_slug.clone();

                            view! {
                                <Title text=format!("{} - Emerald Show", post.title) />
                                <Meta property="og:title" content=post.title.to_string() />
                                <Meta property="og:description" content=post.excerpt.to_string() />
                                <Meta property="og:type" content="article" />
                                <Meta
                                    property="og:url"
                                    content=format!("https://emerald.show/post/{}", route_slug)
                                />
                                <Meta
                                    property="og:image"
                                    content="https://yourblog.com/image.jpg"
                                />
                                <Meta property="og:site_name" content="Emerald Show" />

                                <h1 class="text-2xl font-bold text-emerald mb-4">
                                    {post.title.clone()}
                                </h1>
                                <p class="text-emerald/70 text-sm mb-3">
                                    {to_date_string(post.created_at)}
                                </p>
                                <div class="mt-4 flex flex-col gap-5 md:flex-row md:items-start">
                                    <div class="prose prose-emerald max-w-none min-w-0 flex-1">
                                        <div
                                            class="markdown-body"
                                            inner_html=render_markdown(&post.content)
                                        ></div>
                                    </div>
                                    {post
                                        .featured_image
                                        .map(|image| {
                                            let image_url = image.to_string();
                                            let image_href = image_url.clone();
                                            view! {
                                                <a
                                                    href=image_href
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    class="block w-full md:w-72 lg:w-80 flex-none rounded-xl overflow-hidden border border-white/10 transition-opacity hover:opacity-90"
                                                    title="Open larger image"
                                                >
                                                    <img
                                                        src=image_url
                                                        class="w-full max-h-64 object-cover"
                                                        alt=post.title.to_string()
                                                    />
                                                </a>
                                            }
                                        })}
                                </div>
                                <ApprovedComments post_id=post.id.to_hex() />
                            }
                                })
                        })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn ApprovedComments(post_id: String) -> impl IntoView {
    let comments_post_id = post_id.clone();
    let comments =
        OnceResource::new(async move { fetch_approved_comments(comments_post_id).await });

    view! {
        <section class="post-comments">
            <h2>"Comments"</h2>
            <CommentComposer post_id=post_id />
            <Suspense fallback=move || view! { <p class="post-comments-muted">"Loading comments..."</p> }>
                {move || {
                    comments
                        .get()
                        .map(|result| match result {
                            Ok(comments) if comments.is_empty() => {
                                view! { <p class="post-comments-muted">"No approved comments yet."</p> }.into_any()
                            }
                            Ok(comments) => {
                                view! {
                                    <div class="post-comment-list">
                                        <For
                                            each=move || comments.clone()
                                            key=|comment| comment.id.clone()
                                            children=move |comment| {
                                                view! {
                                                    <article class="post-comment">
                                                        <div class="post-comment-meta">
                                                            <span>{comment.created_at}</span>
                                                            <span>
                                                                {comment.display_name.unwrap_or(comment.author)}
                                                            </span>
                                                        </div>
                                                        <p>{comment.content}</p>
                                                    </article>
                                                }
                                            }
                                        />
                                    </div>
                                }.into_any()
                            }
                            Err(error) => {
                                view! {
                                    <p class="post-comments-muted">
                                        "Could not load comments: " {error.to_string()}
                                    </p>
                                }.into_any()
                            }
                        })
                }}
            </Suspense>
        </section>
    }
}

#[island]
fn CommentComposer(post_id: String) -> impl IntoView {
    let (content, set_content) = signal(String::new());
    let (display_name, set_display_name) = signal(String::new());
    let (has_nostr, set_has_nostr) = signal(false);
    let (posting, set_posting) = signal(false);
    let (status, set_status) = signal(String::new());

    Effect::new(move |_| {
        set_has_nostr.set(nostr_browser::has_nostr_signer());
    });

    let publish = move |_| {
        if posting.get_untracked() {
            return;
        }

        let comment = content.get_untracked();
        if comment.trim().is_empty() {
            set_status.set("Write a comment before publishing.".into());
            return;
        }

        let name = display_name.get_untracked();
        let submit_with_nostr = has_nostr.get_untracked();
        if !submit_with_nostr && name.trim().is_empty() {
            set_status.set("Choose a display name before publishing.".into());
            return;
        }

        let post_id = post_id.clone();
        set_posting.set(true);
        set_status.set(if submit_with_nostr {
            "Waiting for NIP-07 signature...".into()
        } else {
            "Sending anonymous comment...".into()
        });
        leptos::task::spawn_local(async move {
            let result = if submit_with_nostr {
                nostr_browser::publish_comment(&post_id, &comment).await
            } else {
                publish_anonymous_comment(post_id, name, comment)
                    .await
                    .map_err(|error| error.to_string())
            };

            match result {
                Ok(event_id) => {
                    set_content.set(String::new());
                    set_status.set(format!("Comment sent for moderation: {event_id}."));
                }
                Err(error) => set_status.set(error),
            }
            set_posting.set(false);
        });
    };

    view! {
        <div class="post-comment-composer">
            <Show when=move || !has_nostr.get()>
                <input
                    class="post-comment-name"
                    type="text"
                    maxlength="64"
                    placeholder="Display name"
                    prop:value=display_name
                    on:input=move |event| set_display_name.set(event_target_value(&event))
                />
            </Show>
            <textarea
                class="post-comment-input"
                rows="4"
                placeholder="Write a comment"
                prop:value=content
                on:input=move |event| set_content.set(event_target_value(&event))
            ></textarea>
            <div class="post-comment-actions">
                <button
                    class="post-comment-submit"
                    type="button"
                    disabled=move || {
                        posting.get()
                            || content.get().trim().is_empty()
                            || (!has_nostr.get() && display_name.get().trim().is_empty())
                    }
                    on:click=publish
                >
                    {move || {
                        if posting.get() && has_nostr.get() {
                            "Signing..."
                        } else if posting.get() {
                            "Sending..."
                        } else {
                            "Comment"
                        }
                    }}
                </button>
                <Show when=move || !status.get().is_empty()>
                    <p class="post-comment-status">{status}</p>
                </Show>
            </div>
        </div>
    }
}

#[server]
pub async fn publish_anonymous_comment(
    post_id: String,
    display_name: String,
    content: String,
) -> Result<String, ServerFnError> {
    let post_id = EventId::from_hex(&post_id)?;
    let display_name = clean_display_name(&display_name)
        .ok_or_else(|| ServerFnError::new("Display name is required."))?;
    let content = content.trim();

    if content.is_empty() {
        return Err(ServerFnError::new("Comment is required."));
    }

    let client = get_anonymous_comment_client().await;
    let event_id = client
        .send_event_builder(comment_builder(
            post_id,
            anonymous_comment_content(&display_name, content),
        ))
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?;

    Ok(event_id.to_hex())
}

#[server]
pub async fn fetch_approved_comments(
    post_id: String,
) -> Result<Vec<ApprovedComment>, ServerFnError> {
    let post_id = EventId::from_hex(&post_id)?;
    let client = get_nostr_client().await;
    let blog = Blog::new_with_settings(
        client.clone(),
        vec![OWNER_PUBKEY],
        BlogSettings {
            require_comment_approval: true,
        },
    );

    let comments = blog
        .fetch_comments(post_id)
        .await
        .map_err(|error| ServerFnError::new(error.to_string()))?;
    let profiles = fetch_comment_profiles(&client, &comments).await;

    Ok(comments
        .into_iter()
        .map(|comment| approved_comment_from_comment(comment, &profiles))
        .collect())
}

#[cfg(feature = "ssr")]
fn approved_comment_from_comment(
    comment: Comment<'_>,
    profiles: &HashMap<PublicKey, CommentProfile>,
) -> ApprovedComment {
    let parsed = parse_comment_frontmatter(&comment.content);
    let profile_name = profiles
        .get(&comment.author)
        .and_then(CommentProfile::preferred_name);

    ApprovedComment {
        id: comment.id.to_hex(),
        author: comment
            .author
            .to_bech32()
            .unwrap_or_else(|_| comment.author.to_hex()),
        display_name: parsed.display_name.or(profile_name),
        content: parsed.content,
        created_at: to_date_string(comment.created_at),
    }
}

#[cfg(feature = "ssr")]
async fn fetch_comment_profiles(
    client: &libnostrblog::Client,
    comments: &[Comment<'_>],
) -> HashMap<PublicKey, CommentProfile> {
    let mut authors = comments
        .iter()
        .map(|comment| comment.author)
        .collect::<Vec<_>>();
    authors.sort();
    authors.dedup();

    if authors.is_empty() {
        return HashMap::new();
    }

    let Ok(events) = client
        .fetch_events(
            Filter::new().authors(authors).kind(Kind::Metadata),
            Duration::from_secs(10),
        )
        .await
    else {
        return HashMap::new();
    };
    let mut profiles = HashMap::new();

    for event in events.iter() {
        let Ok(profile) = serde_json::from_str::<CommentProfile>(&event.content) else {
            continue;
        };
        profiles.insert(event.pubkey, profile);
    }

    profiles
}

#[cfg(feature = "ssr")]
#[derive(Debug, Deserialize)]
struct CommentProfile {
    name: Option<String>,
    display_name: Option<String>,
    #[serde(alias = "displayName")]
    display_name_camel: Option<String>,
    nip05: Option<String>,
}

#[cfg(feature = "ssr")]
impl CommentProfile {
    fn preferred_name(&self) -> Option<String> {
        self.display_name
            .as_deref()
            .or(self.display_name_camel.as_deref())
            .or(self.name.as_deref())
            .or(self.nip05.as_deref())
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(ToOwned::to_owned)
    }
}

#[cfg(feature = "ssr")]
fn comment_builder(post_id: EventId, content: String) -> EventBuilder {
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

#[cfg(feature = "ssr")]
fn anonymous_comment_content(display_name: &str, content: &str) -> String {
    format!(
        "---\ndisplay_name: \"{}\"\n---\n\n{}",
        display_name.replace('\\', "\\\\").replace('"', "\\\""),
        content.trim()
    )
}

#[cfg(feature = "ssr")]
fn clean_display_name(display_name: &str) -> Option<String> {
    let cleaned = display_name.trim().replace(['\r', '\n'], " ");
    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

    (!cleaned.is_empty()).then(|| cleaned.chars().take(64).collect())
}

#[cfg(feature = "ssr")]
struct ParsedComment {
    display_name: Option<String>,
    content: String,
}

#[cfg(feature = "ssr")]
fn parse_comment_frontmatter(content: &str) -> ParsedComment {
    let Some(rest) = content.strip_prefix("---\n") else {
        return ParsedComment {
            display_name: None,
            content: content.to_owned(),
        };
    };
    let Some((frontmatter, body)) = rest.split_once("\n---\n") else {
        return ParsedComment {
            display_name: None,
            content: content.to_owned(),
        };
    };

    let display_name = frontmatter.lines().find_map(|line| {
        let value = line.strip_prefix("display_name:")?.trim();
        Some(
            value
                .trim_matches('"')
                .replace("\\\"", "\"")
                .replace("\\\\", "\\"),
        )
    });

    ParsedComment {
        display_name,
        content: body.trim_start().to_owned(),
    }
}
