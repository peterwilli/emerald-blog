use leptos::prelude::*;

#[component]
pub fn Intro() -> impl IntoView {
    view! {
        <section class="mb-16 glass rounded-2xl p-6 md:p-10 shadow-2xl backdrop-blur-sm border border-emerald/20">
            <p class="text-emerald/80 text-lg md:text-xl text-center max-w-3xl mx-auto mb-8">
                "Reality is just a shared dream, let's make it magical!"
            </p>
            <div class="prose prose-emerald max-w-none text-emerald/90">
                <p class="text-lg mb-6">
                    "Welcome to the Emerald Show, where the boundaries between reality and dreams blur into a kaleidoscope of wonder. As an artist, coder, data scientist, and lucid dreamer who finds inspiration in that which divides reality from fantasy, we will dive into a world of wonder."
                </p>
            </div>
            <div class="flex justify-center space-x-6 mt-10">
                <a
                    href="https://www.twitch.tv/theemeraldshow"
                    target="_blank"
                    class="social-icon transition-all duration-300 transform hover:rotate-6"
                >
                    <img
                        src="/twitch-icon.svg"
                        alt="Twitch"
                        class="w-10 h-10 md:w-12 md:h-12 filter hover:brightness-125"
                    />
                </a>
                <a
                    href="https://discord.gg/dCjH8zZXuM"
                    target="_blank"
                    class="social-icon transition-all duration-300 transform hover:rotate-6"
                >
                    <img
                        src="/discord-icon.svg"
                        alt="Discord"
                        class="w-10 h-10 md:w-12 md:h-12 filter hover:brightness-125"
                    />
                </a>
            </div>
        </section>
    }
}
