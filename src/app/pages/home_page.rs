use leptos::prelude::*;
use leptos_meta::*;

use crate::app::components::{art_slider::ArtSlider, intro::Intro, post_list::PostList};

#[component]
pub fn HomePage() -> impl IntoView {
    let description = "Emerald's blog about art, code, data science, lucid dreaming, and the mysterious boundary between reality and fantasy.";

    view! {
        <Title text="Emerald Show Blog" />
        <Meta name="description" content=description />
        <Meta property="og:title" content="Emerald Show Blog" />
        <Meta property="og:description" content=description />
        <Meta property="og:type" content="website" />
        <Meta property="og:url" content="https://emerald.show/" />
        <Meta property="og:image" content="https://emerald.show/android-chrome-512x512.png" />
        <Meta property="og:site_name" content="Emerald Show" />
        <div class="flex flex-col md:flex-row gap-8">
            <div class="flex-1">
                <ArtSlider />
                <Intro />
            </div>
            <div class="flex-1">
                <PostList />
            </div>
        </div>
    }
}
