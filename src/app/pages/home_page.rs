use leptos::prelude::*;
use leptos_meta::*;

use crate::app::components::{art_slider::ArtSlider, intro::Intro, post_list::PostList};

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Title text="Emerald Show Blog" />
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
