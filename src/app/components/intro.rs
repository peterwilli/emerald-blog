use leptos::prelude::*;

#[component]
pub fn Intro() -> impl IntoView {
    view! {
        <section class="mb-16 glass rounded-2xl p-6 md:p-10 shadow-2xl backdrop-blur-sm border border-emerald/20">
            <h1 class="text-4xl md:text-5xl font-bold text-center text-emerald mb-4 font-fantasy">
                "Emerald Show"
            </h1>
            <p class="text-emerald/80 text-lg md:text-xl text-center max-w-3xl mx-auto mb-8">
                "Reality is just a shared dream - let's make it magical!"
            </p>
            <div class="prose prose-emerald max-w-none text-emerald/90">
                <p class="text-lg mb-6">
                    "Welcome to the Emerald Show, where the boundaries between reality and dreams blur into a kaleidoscope of wonder. I'm Peter Willemsen, an artist, coder, data scientist, and lucid dreamer who finds inspiration in the mystical world of dragons and the mesmerizing beauty of emeralds."
                </p>
                <p class="text-lg mb-6">
                    "This blog is my canvas for sharing the magic I discover in both the digital and dream realms. From exploring the mathematical patterns that underlie our universe to the fantastical landscapes of my subconscious, I invite you to join me on this journey of creativity, technology, and imagination."
                </p>
                <p class="text-lg">
                    "Whether I'm crafting code, painting digital art, or exploring the depths of data, I always keep one eye open for the sparkle of the extraordinary in the ordinary. So come along, let's dive into the Emerald Show and discover the magic that surrounds us all."
                </p>
            </div>
            <div class="flex justify-center space-x-6 mt-10">
                <a
                    href="https://www.twitch.tv/theemeraldshow"
                    target="_blank"
                    class="social-icon transition-all duration-300 transform hover:rotate-6"
                >
                    <img
                        src="./twitch.svg"
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
                        src="./discord.svg"
                        alt="Discord"
                        class="w-10 h-10 md:w-12 md:h-12 filter hover:brightness-125"
                    />
                </a>
            </div>
        </section>
    }
}
