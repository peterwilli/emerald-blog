use leptos::prelude::*;
use leptos_meta::*;

use crate::app::components::{intro::Intro, post_list::PostList};

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Title text="Emerald Show Blog" />
        <Navbar />
        <Intro />
        <PostList />
    }
}
