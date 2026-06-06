use leptos::prelude::*;

use crate::app::utils::is_video_media_url;

#[component]
pub fn MediaDisplay(
    src: String,
    #[prop(default = "")] class: &'static str,
    alt: String,
) -> AnyView {
    let src = src.trim().to_owned();

    if src.is_empty() {
        return view! { <div class="media-display-empty">"No content URL"</div> }.into_any();
    }

    if is_video_media_url(&src) {
        let class = if class.is_empty() {
            "media-display-video".to_owned()
        } else {
            format!("{class} media-display-video")
        };

        view! {
            <video
                src=src
                class=class
                autoplay
                loop
                muted
                playsinline
                preload="metadata"
                aria-label=alt
            ></video>
        }
        .into_any()
    } else {
        view! { <img src=src class=class alt=alt /> }.into_any()
    }
}
