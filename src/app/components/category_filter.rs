use leptos::prelude::*;
use libnostrblog::objects::post::Post;

use crate::app::utils::{category_href, is_art_category};

#[component]
pub fn CategoryFilter(
    posts: Vec<Post<'static>>,
    base_path: &'static str,
    #[prop(default = None)] active_category: Option<String>,
) -> impl IntoView {
    let categories = collect_categories(&posts);

    if categories.is_empty() {
        return ().into_any();
    }

    view! {
        <nav class="category-filter glass" aria-label="Filter by category">
            <a
                href=base_path
                class=if active_category.is_none() {
                    "category-filter-chip category-filter-chip-active"
                } else {
                    "category-filter-chip"
                }
            >
                "All"
            </a>
            <For
                each=move || categories.clone()
                key=|category| category.to_lowercase()
                children=move |category| {
                    let is_active = active_category
                        .as_ref()
                        .is_some_and(|active| active.eq_ignore_ascii_case(&category));
                    let href = category_href(base_path, &category);
                    let label = category.clone();
                    view! {
                        <a
                            href=href
                            class=if is_active {
                                "category-filter-chip category-filter-chip-active"
                            } else {
                                "category-filter-chip"
                            }
                        >
                            {label}
                        </a>
                    }
                }
            />
        </nav>
    }
    .into_any()
}

fn collect_categories(posts: &[Post<'static>]) -> Vec<String> {
    let mut categories = Vec::<String>::new();

    for post in posts {
        for category in &post.categories {
            let category = category.trim();
            if category.is_empty()
                || is_art_category(category)
                || categories
                    .iter()
                    .any(|existing| existing.eq_ignore_ascii_case(category))
            {
                continue;
            }

            categories.push(category.to_owned());
        }
    }

    categories.sort_by_key(|category| category.to_lowercase());
    categories
}
