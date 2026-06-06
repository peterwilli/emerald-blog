use leptos::prelude::*;

use crate::app::nostr_browser::{self as nostr, MigrationPreview};

use super::post_editor::PublishNotice;

#[component]
pub fn MigrationTool(
    posting: ReadSignal<bool>,
    set_posting: WriteSignal<bool>,
    status: ReadSignal<String>,
    set_status: WriteSignal<String>,
) -> impl IntoView {
    let (raw_events, set_raw_events) = signal(String::new());
    let (previews, set_previews) = signal(Vec::<MigrationPreview>::new());
    let (parsing, set_parsing) = signal(false);

    let parse = move |_| {
        if parsing.get_untracked() {
            return;
        }

        let raw = raw_events.get_untracked();
        set_parsing.set(true);
        set_status.set("Parsing raw Nostr events...".into());
        leptos::task::spawn_local(async move {
            match nostr::preview_text_note_migration(&raw).await {
                Ok(items) => {
                    let count = items.len();
                    set_previews.set(items);
                    set_status.set(format!("Found {count} kind 1 event(s) ready to migrate."));
                }
                Err(error) => {
                    set_previews.set(Vec::new());
                    set_status.set(error);
                }
            }
            set_parsing.set(false);
        });
    };

    let publish = move |_| {
        if posting.get_untracked() {
            return;
        }

        let raw = raw_events.get_untracked();
        set_posting.set(true);
        set_status.set("Waiting for NIP-07 signatures...".into());
        leptos::task::spawn_local(async move {
            match nostr::publish_text_note_migration(&raw).await {
                Ok(results) => {
                    let count = results.len();
                    let last = results
                        .last()
                        .map(|result| result.event_id.as_str())
                        .unwrap_or("");
                    set_status.set(format!(
                        "Migrated {count} post(s) as long-form events and deleted the source kind 1 events. Last event: {last}."
                    ));
                }
                Err(error) => set_status.set(error),
            }
            set_posting.set(false);
        });
    };

    view! {
        <div class="office2003-menubar">
            <button
                class="office2003-tool office2003-tool-primary"
                type="button"
                on:click=parse
                disabled=parsing
            >
                <span class="office2003-icon office2003-icon-refresh"></span>
                {move || if parsing.get() { "Parsing" } else { "Parse Raw" }}
            </button>
            <button
                class="office2003-tool office2003-tool-primary"
                type="button"
                on:click=publish
                disabled=posting
            >
                <span class="office2003-icon office2003-icon-mail"></span>
                {move || if posting.get() { "Migrating" } else { "Repost Long-Form" }}
            </button>
            <PublishNotice status posting />
        </div>

        <div class="admin-post-form admin-migration-panel">
            <label>
                <span>"Raw Nostr CLI output"</span>
                <textarea
                    class="admin-post-textarea admin-migration-raw"
                    prop:value=raw_events
                    on:input=move |event| set_raw_events.set(event_target_value(&event))
                ></textarea>
            </label>

            <div class="admin-migration-summary">
                <b>{move || previews.get().len()}</b>
                <span>"kind 1 events parsed for migration"</span>
            </div>

            {move || {
                let previews = previews.get();
                if previews.is_empty() {
                    view! {
                        <p class="admin-post-hint">
                            "Paste newline-delimited raw event JSON from the Nostr CLI, then parse it before reposting."
                        </p>
                    }
                    .into_any()
                } else {
                    view! {
                        <div class="admin-comment-list">
                            <For
                                each=move || previews.clone()
                                key=|preview| preview.source_id.clone()
                                children=move |preview| {
                                    view! { <MigrationPreviewItem preview /> }.into_any()
                                }
                            />
                        </div>
                    }
                    .into_any()
                }
            }}
        </div>
    }
}

#[component]
fn MigrationPreviewItem(preview: MigrationPreview) -> impl IntoView {
    let categories = if preview.categories.is_empty() {
        "No categories".to_owned()
    } else {
        preview.categories.join(", ")
    };

    view! {
        <article class="admin-comment-item">
            <div class="admin-comment-meta">
                <span>{format!("Source {}", preview.source_id)}</span>
                <span>{format!("Original timestamp {}", preview.created_at)}</span>
            </div>
            <p class="admin-comment-content">{preview.title}</p>
            <div class="admin-comment-refs">
                <span>"Kind 1 -> 30023"</span>
                <span>{categories}</span>
            </div>
        </article>
    }
}
