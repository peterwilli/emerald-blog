use crate::app::{
    AppState,
    utils::{is_dream_art_post, post_slug},
};
use leptos::prelude::*;
use libnostrblog::objects::post::Post;
use serde::{Deserialize, Serialize};

const SLIDE_INTERVAL_MS: i32 = 6_500;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArtSlide {
    title: String,
    excerpt: String,
    image: String,
    href: String,
}

impl ArtSlide {
    fn from_post(post: &Post) -> Option<Self> {
        let image = post.featured_image.as_ref()?.to_string();

        if image.trim().is_empty() {
            return None;
        }

        Some(Self {
            title: post.title.to_string(),
            excerpt: post.excerpt.to_string(),
            image,
            href: format!("/post/{}", post_slug(post)),
        })
    }
}

#[component]
pub fn ArtSlider() -> impl IntoView {
    let state = use_context::<AppState>().unwrap();
    let posts = state.posts.clone();

    view! {
        <Suspense fallback=move || {
            view! { <div class="art-slider-skeleton glass rounded-xl mb-6"></div> }
        }>
            {move || {
                posts
                    .get()
                    .map(|posts| {
                        let slides = posts
                            .iter()
                            .filter(|post| is_dream_art_post(post))
                            .filter_map(ArtSlide::from_post)
                            .collect::<Vec<_>>();

                        view! { <ArtSliderView slides=slides /> }
                    })
            }}
        </Suspense>
    }
}

#[component]
fn ArtSliderView(slides: Vec<ArtSlide>) -> impl IntoView {
    if slides.is_empty() {
        return ().into_any();
    }

    let (active_index, set_active_index) = signal(0usize);
    let (timer_key, set_timer_key) = signal(0usize);
    let stage_slides = slides.clone();
    let thumb_slides = slides.clone();

    #[cfg(feature = "hydrate")]
    {
        use std::time::Duration;

        let slide_count = slides.len();
        if let Ok(interval) = set_interval_with_handle(
            move || {
                if slide_count > 1 {
                    set_active_index.update(|idx| *idx = (*idx + 1) % slide_count);
                    set_timer_key.update(|key| *key += 1);
                }
            },
            Duration::from_millis(SLIDE_INTERVAL_MS as u64),
        ) {
            on_cleanup(move || interval.clear());
        }
    }

    view! {
        <section class="art-slider glass rounded-xl overflow-hidden mb-6">
            <div class="art-slider-stage">
                <For
                    each=move || stage_slides.clone().into_iter().enumerate()
                    key=|(_, slide)| slide.href.clone()
                    children=move |(idx, slide): (usize, ArtSlide)| {
                        let active_slide = move || active_index.get() == idx;

                        view! {
                            <a
                                id=format!("DreamArt{}", idx)
                                href=slide.href
                                class=move || {
                                    if active_slide() {
                                        "art-slider-slide art-slider-slide-active"
                                    } else {
                                        "art-slider-slide"
                                    }
                                }
                                aria-hidden=move || (!active_slide()).to_string()
                            >
                                <img src=slide.image class="art-slider-image" />
                                <div class="art-slider-vignette"></div>
                                <div class="art-slider-copy">
                                    <div class="art-slider-kicker">"Dream Art"</div>
                                    <h2>{slide.title}</h2>
                                    <p>{slide.excerpt}</p>
                                </div>
                            </a>
                        }
                    }
                />
                <div class="art-slider-progress" aria-hidden="true">
                    <For
                        each=move || vec![timer_key.get()]
                        key=|key| *key
                        children=move |_| {
                            view! {
                                <div
                                    class="art-slider-progress-fill"
                                    style=format!("animation-duration: {}ms;", SLIDE_INTERVAL_MS)
                                ></div>
                            }
                        }
                    />
                </div>
            </div>
            <div class="art-slider-filmstrip">
                <For
                    each=move || thumb_slides.clone().into_iter().enumerate()
                    key=|(_, slide)| slide.href.clone()
                    children=move |(idx, slide): (usize, ArtSlide)| {
                        let title = slide.title.clone();
                        let select_slide = move |ev: leptos::ev::MouseEvent| {
                            ev.prevent_default();
                            set_active_index.set(idx);
                            set_timer_key.update(|key| *key += 1);
                        };

                        view! {
                            <a
                                href=format!("#DreamArt{}", idx)
                                class=move || {
                                    if active_index.get() == idx {
                                        "art-slider-thumb art-slider-thumb-active"
                                    } else {
                                        "art-slider-thumb"
                                    }
                                }
                                aria-label=format!("Show {}", title)
                                aria-current=move || {
                                    if active_index.get() == idx {
                                        "true"
                                    } else {
                                        "false"
                                    }
                                }
                                on:click=select_slide
                            >
                                <img src=slide.image alt="" aria-hidden="true" />
                                <span>{idx + 1}</span>
                            </a>
                        }
                    }
                />
            </div>
        </section>
    }
    .into_any()
}
