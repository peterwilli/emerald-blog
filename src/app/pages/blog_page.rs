use leptos::prelude::*;
use leptos_meta::*;

use crate::app::components::post_list::PostList;

#[component]
pub fn BlogPage() -> impl IntoView {
    view! {
        <Title text="Emerald Show Blog" />
        <PostList />
    }
}
