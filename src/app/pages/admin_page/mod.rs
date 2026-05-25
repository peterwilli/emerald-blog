mod art_editor;
mod delete_posts;
mod moderation;
mod post_editor;

use crate::app::nostr_browser as nostr;
use leptos::prelude::*;
use leptos_meta::Title;

use self::{
    art_editor::ArtEditor, delete_posts::DeletePosts, moderation::CommentModeration,
    post_editor::PostEditor,
};
use nostr::PostDraft;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AdminTab {
    Post,
    Art,
    Comments,
    Delete,
}

#[component]
pub fn AdminPostPage() -> impl IntoView {
    let (tab, set_tab) = signal(AdminTab::Post);
    let (posting, set_posting) = signal(false);
    let (status, set_status) = signal(String::new());

    view! {
        <Title text="Emerald Admin" />
        <div class="glass rounded-xl overflow-hidden admin-post-shell">
            <div class="office2003-titlebar">
                <div class="office2003-brand">"Emerald Admin"</div>
                <div class="office2003-tabs">
                    <AdminTabButton current=tab set_current=set_tab label="Post" target=AdminTab::Post />
                    <AdminTabButton current=tab set_current=set_tab label="Art" target=AdminTab::Art />
                    <AdminTabButton current=tab set_current=set_tab label="Comments" target=AdminTab::Comments />
                    <AdminTabButton current=tab set_current=set_tab label="Delete" target=AdminTab::Delete />
                </div>
            </div>

            <Show
                when=move || tab.get() == AdminTab::Delete
                fallback=move || {
                    view! {
                        <Show
                            when=move || tab.get() == AdminTab::Comments
                            fallback=move || {
                                view! {
                                    <Show
                                        when=move || tab.get() == AdminTab::Art
                                        fallback=move || {
                                            view! {
                                                <PostEditor posting set_posting status set_status />
                                            }
                                        }
                                    >
                                        <ArtEditor posting set_posting status set_status />
                                    </Show>
                                }
                            }
                        >
                            <CommentModeration status set_status />
                        </Show>
                    }
                }
            >
                <DeletePosts status set_status />
            </Show>
        </div>
    }
}

#[component]
fn AdminTabButton(
    current: ReadSignal<AdminTab>,
    set_current: WriteSignal<AdminTab>,
    label: &'static str,
    target: AdminTab,
) -> impl IntoView {
    view! {
        <button
            class=move || {
                if current.get() == target {
                    "office2003-tab office2003-tab-active"
                } else {
                    "office2003-tab"
                }
            }
            type="button"
            on:click=move |_| set_current.set(target)
        >
            {label}
        </button>
    }
}

fn publish_post(
    draft: PostDraft,
    posting: ReadSignal<bool>,
    set_posting: WriteSignal<bool>,
    set_status: WriteSignal<String>,
    success_message: &'static str,
) {
    if posting.get_untracked() {
        return;
    }

    set_posting.set(true);
    set_status.set("Waiting for NIP-07 signature...".into());
    leptos::task::spawn_local(async move {
        match nostr::publish_nostr_post(draft).await {
            Ok(event_id) => {
                set_status.set(format!("{success_message} {event_id}."));
            }
            Err(error) => set_status.set(error),
        }
        set_posting.set(false);
    });
}
