use crate::app::{
    AppState,
    components::media_display::MediaDisplay,
    markdown::render_markdown,
    nostr_browser,
    utils::{post_slug, title_slug, to_date_string},
};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;
#[cfg(any(feature = "ssr", feature = "hydrate"))]
use libnostrblog::{
    Blog, BlogSettings, EventId, Filter, Kind, PublicKey, blog::comments::CommentsExt,
    objects::comment::Comment,
};
use libnostrblog::{ToBech32, objects::post::Author};
use serde::{Deserialize, Serialize};
#[cfg(any(feature = "ssr", feature = "hydrate"))]
use std::{collections::HashMap, time::Duration};

use crate::app::client::OWNER_PUBKEY;
#[cfg(any(feature = "ssr", feature = "hydrate"))]
use crate::app::client::get_nostr_client;

#[cfg(feature = "hydrate")]
const COMMENT_REFRESH_INTERVAL_MS: u64 = 15_000;
const NJUMP_PROFILE_PREFIX: &str = "https://njump.me/";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApprovedComment {
    pub id: String,
    pub author: String,
    pub display_name: Option<String>,
    pub content: String,
    pub created_at: String,
    pub created_at_secs: u64,
}

#[component]
pub fn PostPage() -> impl IntoView {
    let params = use_params_map();
    let route_slug = params.read_untracked().get("slug").unwrap_or_default();
    let state = use_context::<AppState>().unwrap();
    let posts = state.posts.clone();
    let live_posts = state.live_posts.clone();

    view! {
        <div class="glass rounded-xl p-6 transition-all duration-300 post-card">
            <Suspense fallback=move || {
                view! { <p>"Loading..."</p> }
            }>
                {move || {
                    posts
                        .try_get()
                        .flatten()
                        .map(|seeded_posts| {
                            let current_posts = live_posts.try_get().unwrap_or_default();
                            let posts = if current_posts.is_empty() {
                                seeded_posts
                            } else {
                                current_posts
                            };

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
                                {post.featured_image.as_ref().map(|image| {
                                    view! {
                                        <Meta property="og:image" content=image.to_string() />
                                    }
                                })}
                                <Meta property="og:site_name" content="Emerald Show" />

                                <h1 class="text-2xl font-bold text-emerald mb-4">
                                    {post.title.clone()}
                                </h1>
                                <PostAuthor author=post.author.clone() created_at=to_date_string(post.created_at) />
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
                                                    <MediaDisplay
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
fn PostAuthor(author: Author<'static>, created_at: String) -> impl IntoView {
    let name = author_name(&author);
    let handle = author_handle(&author);
    let about = author_about(&author);
    let profile_href = owner_profile_href();
    let initial = name
        .chars()
        .find(|character| character.is_alphanumeric())
        .map(|character| character.to_uppercase().collect::<String>())
        .unwrap_or_else(|| "E".to_owned());

    view! {
        <div class="mb-6 flex items-center gap-3 border-y border-white/10 py-3">
            <a
                href=profile_href.clone()
                target="_blank"
                rel="noopener noreferrer"
                class="h-12 w-12 flex-none overflow-hidden rounded-full border border-emerald/50 bg-emerald/15 shadow-[0_0_16px_rgba(80,200,120,0.2)] transition-opacity hover:opacity-85"
                title="Open Emerald profile"
            >
                {author
                    .picture
                    .map(|picture| {
                        view! {
                            <img
                                src=picture.to_string()
                                alt=format!("{name} profile picture")
                                class="h-full w-full object-cover"
                            />
                        }
                            .into_any()
                    })
                    .unwrap_or_else(|| {
                        view! {
                            <div class="flex h-full w-full items-center justify-center text-lg font-bold text-white">
                                {initial}
                            </div>
                        }
                            .into_any()
                    })}
            </a>
            <div class="min-w-0">
                <div class="flex flex-wrap items-baseline gap-x-2 gap-y-1">
                    <span class="text-sm text-white/60">"Written by"</span>
                    <a
                        href=profile_href.clone()
                        target="_blank"
                        rel="noopener noreferrer"
                        class="truncate text-base font-semibold text-white transition-colors hover:text-emerald"
                    >
                        {name}
                    </a>
                </div>
                <div class="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-sm text-white/60">
                    <a
                        href=profile_href
                        target="_blank"
                        rel="noopener noreferrer"
                        class="max-w-full truncate transition-colors hover:text-emerald"
                    >
                        {handle}
                    </a>
                    <span aria-hidden="true">"/"</span>
                    <time>{created_at}</time>
                </div>
                {about
                    .map(|about| {
                        view! {
                            <p class="mt-2 text-sm leading-relaxed text-white/70">
                                {about}
                            </p>
                        }
                    })}
            </div>
        </div>
    }
}

fn owner_profile_href() -> String {
    format!(
        "{NJUMP_PROFILE_PREFIX}{}",
        OWNER_PUBKEY
            .to_bech32()
            .unwrap_or_else(|_| OWNER_PUBKEY.to_hex())
    )
}

fn author_name(author: &Author<'_>) -> String {
    author
        .display_name
        .as_deref()
        .or(author.username.as_deref())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| author_fallback_key(author))
}

fn author_handle(author: &Author<'_>) -> String {
    author
        .username
        .as_deref()
        .map(str::trim)
        .filter(|username| !username.is_empty())
        .map(|username| format!("@{username}"))
        .unwrap_or_else(|| author_fallback_key(author))
}

fn author_about(author: &Author<'_>) -> Option<String> {
    author
        .about
        .as_deref()
        .map(str::trim)
        .filter(|about| !about.is_empty())
        .map(ToOwned::to_owned)
}

fn author_fallback_key(author: &Author<'_>) -> String {
    author
        .pubkey
        .to_bech32()
        .unwrap_or_else(|_| author.pubkey.to_hex())
}

#[component]
fn ApprovedComments(post_id: String) -> impl IntoView {
    cfg_if::cfg_if! {
        if #[cfg(all(
            feature = "hydrate",
            feature = "client-side-nostr-fetch",
            not(feature = "islands")
        ))] {
            let comments_post_id = post_id.clone();
            let comments = LocalResource::new(move || {
                let comments_post_id = comments_post_id.clone();
                async move { fetch_approved_comments(comments_post_id).await }
            });
            let comments_result = RwSignal::new(None::<Result<Vec<ApprovedComment>, String>>);
            let live_comments = RwSignal::new(Vec::<ApprovedComment>::new());
            let refresh_started = RwSignal::new(false);
            let refresh_post_id = post_id.clone();

            Effect::new(move |_| {
                let Some(result) = comments.get() else {
                    return;
                };

                if let Ok(comments) = &result {
                    start_comment_refresh(
                        refresh_post_id.clone(),
                        live_comments,
                        refresh_started,
                        comments.iter().map(|comment| comment.id.clone()).collect(),
                    );
                }

                comments_result.set(Some(result));
            });

            view! {
                <section class="post-comments">
                    <h2>"Comments"</h2>
                    <CommentComposer post_id=post_id.clone() />
                    {move || {
                        comments_result
                            .get()
                            .map(|result| match result {
                                Ok(comments) => {
                                    view! {
                                        <CommentList live_comments=live_comments seeded_comments=comments />
                                    }.into_any()
                                }
                                Err(error) => {
                                    view! {
                                        <p class="post-comments-muted">
                                            "Could not load comments: " {error}
                                        </p>
                                    }.into_any()
                                }
                            })
                            .unwrap_or_else(|| {
                                view! { <p class="post-comments-muted">"Loading comments..."</p> }.into_any()
                            })
                    }}
                </section>
            }
            .into_any()
        } else {
            #[cfg(feature = "hydrate")]
            let comments = {
                let comments_post_id = post_id.clone();
                LocalResource::new(move || {
                    let comments_post_id = comments_post_id.clone();
                    async move { fetch_approved_comments(comments_post_id).await }
                })
            };
            #[cfg(not(feature = "hydrate"))]
            let comments = {
                let comments_post_id = post_id.clone();
                ArcOnceResource::new(async move { fetch_approved_comments(comments_post_id).await })
            };
            let live_comments = RwSignal::new(Vec::<ApprovedComment>::new());
            #[cfg(feature = "hydrate")]
            let refresh_started = RwSignal::new(false);

            view! {
                <section class="post-comments">
                    <h2>"Comments"</h2>
                    <CommentComposer post_id=post_id.clone() />
                    <Suspense fallback=move || view! { <p class="post-comments-muted">"Loading comments..."</p> }>
                        {move || {
                            comments
                                .get()
                                .map(|result| match result {
                                    Ok(comments) => {
                                        #[cfg(feature = "hydrate")]
                                        start_comment_refresh(
                                            post_id.clone(),
                                            live_comments,
                                            refresh_started,
                                            comments.iter().map(|comment| comment.id.clone()).collect(),
                                        );

                                        view! {
                                            <CommentList live_comments=live_comments seeded_comments=comments />
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
            .into_any()
        }
    }
}

#[component]
fn CommentList(
    live_comments: RwSignal<Vec<ApprovedComment>>,
    seeded_comments: Vec<ApprovedComment>,
) -> impl IntoView {
    let seeded_comments = RwSignal::new(seeded_comments);

    view! {
        <Show
            when=move || {
                !live_comments.read().is_empty() || !seeded_comments.read().is_empty()
            }
            fallback=move || view! { <p class="post-comments-muted">"No approved comments yet."</p> }
        >
            <div class="post-comment-list">
                <For
                    each=move || {
                        live_comments
                            .get()
                            .into_iter()
                            .chain(seeded_comments.get())
                            .collect::<Vec<_>>()
                    }
                    key=|comment| comment.id.clone()
                    children=move |comment| view! { <CommentItem comment=comment /> }
                />
            </div>
        </Show>
    }
}

#[component]
fn CommentItem(comment: ApprovedComment) -> impl IntoView {
    view! {
        <article class="post-comment">
            <div class="post-comment-meta">
                <span>{comment.created_at}</span>
                <span>{comment.display_name.unwrap_or(comment.author)}</span>
            </div>
            <p>{comment.content}</p>
        </article>
    }
}

#[cfg(feature = "hydrate")]
fn start_comment_refresh(
    post_id: String,
    live_comments: RwSignal<Vec<ApprovedComment>>,
    refresh_started: RwSignal<bool>,
    seeded_comment_ids: Vec<String>,
) {
    use std::time::Duration;

    if refresh_started.get_untracked() {
        return;
    }
    refresh_started.set(true);

    refresh_approved_comments(post_id.clone(), live_comments, seeded_comment_ids.clone());

    if let Ok(interval) = set_interval_with_handle(
        move || {
            refresh_approved_comments(post_id.clone(), live_comments, seeded_comment_ids.clone());
        },
        Duration::from_millis(COMMENT_REFRESH_INTERVAL_MS),
    ) {
        on_cleanup(move || interval.clear());
    }
}

#[cfg(feature = "hydrate")]
fn refresh_approved_comments(
    post_id: String,
    live_comments: RwSignal<Vec<ApprovedComment>>,
    seeded_comment_ids: Vec<String>,
) {
    leptos::task::spawn_local(async move {
        let Ok(comments) = fetch_approved_comments(post_id).await else {
            return;
        };

        live_comments.update(|live_comments| {
            for comment in comments {
                if seeded_comment_ids.contains(&comment.id)
                    || live_comments
                        .iter()
                        .any(|existing| existing.id == comment.id)
                {
                    continue;
                }

                live_comments.push(comment);
            }

            live_comments.sort_by(|left, right| {
                right
                    .created_at_secs
                    .cmp(&left.created_at_secs)
                    .then_with(|| right.id.cmp(&left.id))
            });
        });
    });
}

#[cfg_attr(feature = "islands", island)]
#[cfg_attr(not(feature = "islands"), component)]
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
                nostr_browser::publish_local_comment(&post_id, &name, &comment).await
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

#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub async fn fetch_approved_comments(post_id: String) -> Result<Vec<ApprovedComment>, String> {
    let post_id = EventId::from_hex(&post_id).map_err(|error| error.to_string())?;
    let client = get_nostr_client().await;
    let blog = Blog::new_with_settings(
        client.clone(),
        vec![OWNER_PUBKEY],
        BlogSettings {
            require_comment_approval: true,
        },
    );

    let comments = blog
        .fetch_approved_comments(post_id)
        .await
        .map_err(|error| error.to_string())?;
    let profiles = fetch_comment_profiles(&client, &comments).await;

    let mut comments = comments
        .into_iter()
        .map(|comment| approved_comment_from_comment(comment, &profiles))
        .collect::<Vec<_>>();
    comments.sort_by(|left, right| {
        right
            .created_at_secs
            .cmp(&left.created_at_secs)
            .then_with(|| right.id.cmp(&left.id))
    });

    Ok(comments)
}

#[cfg(not(any(feature = "ssr", feature = "hydrate")))]
pub async fn fetch_approved_comments(_post_id: String) -> Result<Vec<ApprovedComment>, String> {
    Err("Comment fetching is only available in the SSR or hydrated app.".into())
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
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
        created_at_secs: comment.created_at.as_u64(),
    }
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
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

#[cfg(any(feature = "ssr", feature = "hydrate"))]
#[derive(Debug, Deserialize)]
struct CommentProfile {
    name: Option<String>,
    display_name: Option<String>,
    #[serde(alias = "displayName")]
    display_name_camel: Option<String>,
    nip05: Option<String>,
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
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

#[cfg(any(feature = "ssr", feature = "hydrate"))]
struct ParsedComment {
    display_name: Option<String>,
    content: String,
}

#[cfg(any(feature = "ssr", feature = "hydrate"))]
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
