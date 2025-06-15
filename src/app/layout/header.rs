use leptos::prelude::*;

#[component]
pub fn HeaderItem<'a>(link: &'a str, title: &'a str) -> impl IntoView {
    view! {
        <a
            href=link
            class="relative group px-3 py-2 text-emerald transition-all duration-300 hover:text-gold after:content-[''] after:absolute after:bottom-0 after:left-0 after:w-full after:h-0.5 after:bg-gradient-to-r after:from-emerald after:to-gold after:scale-x-0 after:origin-bottom-right after:transition-transform after:duration-300 after:hover:origin-bottom-left after:hover:scale-x-100"
        >
            {title}
        </a>
    }
}

#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="relative z-10">
            <nav class="container mx-auto px-4 py-6">
                // Desktop/Tablet Navigation
                <div class="hidden md:flex items-center justify-between">
                    <div class="flex space-x-4">
                        <HeaderItem title="Home" link="/" />
                        <a
                            href="#"
                            class="relative group px-3 py-2 text-emerald transition-all duration-300 hover:text-gold after:content-[''] after:absolute after:bottom-0 after:left-0 after:w-full after:h-0.5 after:bg-gradient-to-r after:from-emerald after:to-gold after:scale-x-0 after:origin-bottom-right after:transition-transform after:duration-300 after:hover:origin-bottom-left after:hover:scale-x-100"
                        >
                            About
                        </a>
                    </div>

                    // Centered Logo
                    <div class="flex items-center justify-center flex-grow">
                        <div class="logo-glow animate-float">
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                xml:space="preserve"
                                width="200"
                                height="218"
                                viewBox="0 0 68.922 75.766"
                                class="w-24 h-24 md:w-32 md:h-32"
                            >
                                <path
                                    d="M157.082 115.88c-7.29-1.382 1.746 10.303.343 1.334zm-2.644 9.655c7.529 6.372.221 17.656-8.812 13.814-10.75 3.671 11.433 7.342.78 13.42-5.087-1.71-10.96 3.432-16.017 1.636-8.204.533-3.763 15.847-14.104 14.406-6.92-2.653-5.933 5.195-.255 4.667 9.194-6.85-1.174 5.456-3.737 8.963-2.843 3.341-10.514 5.827-2.098 4.3 5.806-1.32 12.26-16.05 15.18-4.366 5.408 4.276 14.008 2.694 20.659 1.686 16.092-3.242 27.186-20.507 23.739-36.536-1.518-9.12-7.123-17.594-15.335-21.99m5.121 14.567c1.616-.413-.72 1.486 0 0m8.228 4.702c-1.612 4.38-10.691 1.96-9.948 7-1.373-6.689 5.672-8.3 10.197-5.843l.074-.425zm-39.25 1.175c-5.706-.652 2.142 5.037 0 0m1.763 8.565-.21.11zm27.622 2.098c5.32 8.098-3.967 16.05-10.832 18.83-11.03 1.345 5.472-4.332 7.686-7.351 3.263-2.76 4.964-7.432 3.146-11.479m-22.542 2.1c5.464 4.677-3.312 12.022-.78 16.973-3.124 3.56-10.45 4.482-4.178-.68 3.37-4.49 7.023-10.51 4.958-16.294m-7.473 2.435-.042.042zm-.085.028-.155.087zm20.402.882c-1.302 2.602-3.654.316 0 0m-7.01 6.548-.042.042zm2.183.168-.042.042zm-.21.084-.042.042zm-2.603.546.042.042z"
                                    style="fill:#50C878;stroke:#020202;stroke-width:0.5"
                                    transform="translate(-101.376 -111.263)"
                                />
                            </svg>
                        </div>
                    </div>

                    <div class="flex space-x-4">
                        <a
                            href="#"
                            class="relative group px-3 py-2 text-emerald transition-all duration-300 hover:text-gold after:content-[''] after:absolute after:bottom-0 after:left-0 after:w-full after:h-0.5 after:bg-gradient-to-r after:from-emerald after:to-gold after:scale-x-0 after:origin-bottom-right after:transition-transform after:duration-300 after:hover:origin-bottom-left after:hover:scale-x-100"
                        >
                            Blog
                        </a>
                        <a
                            href="#"
                            class="relative group px-3 py-2 text-emerald transition-all duration-300 hover:text-gold after:content-[''] after:absolute after:bottom-0 after:left-0 after:w-full after:h-0.5 after:bg-gradient-to-r after:from-emerald after:to-gold after:scale-x-0 after:origin-bottom-right after:transition-transform after:duration-300 after:hover:origin-bottom-left after:hover:scale-x-100"
                        >
                            Contact
                        </a>
                    </div>
                </div>

                // Mobile Navigation
                <div class="md:hidden flex flex-col">
                    <div class="flex justify-between items-center">
                        <div class="logo-glow animate-float">
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                xml:space="preserve"
                                width="200"
                                height="218"
                                viewBox="0 0 68.922 75.766"
                                class="w-20 h-20"
                            >
                                <path
                                    d="M157.082 115.88c-7.29-1.382 1.746 10.303.343 1.334zm-2.644 9.655c7.529 6.372.221 17.656-8.812 13.814-10.75 3.671 11.433 7.342.78 13.42-5.087-1.71-10.96 3.432-16.017 1.636-8.204.533-3.763 15.847-14.104 14.406-6.92-2.653-5.933 5.195-.255 4.667 9.194-6.85-1.174 5.456-3.737 8.963-2.843 3.341-10.514 5.827-2.098 4.3 5.806-1.32 12.26-16.05 15.18-4.366 5.408 4.276 14.008 2.694 20.659 1.686 16.092-3.242 27.186-20.507 23.739-36.536-1.518-9.12-7.123-17.594-15.335-21.99m5.121 14.567c1.616-.413-.72 1.486 0 0m8.228 4.702c-1.612 4.38-10.691 1.96-9.948 7-1.373-6.689 5.672-8.3 10.197-5.843l.074-.425zm-39.25 1.175c-5.706-.652 2.142 5.037 0 0m1.763 8.565-.21.11zm27.622 2.098c5.32 8.098-3.967 16.05-10.832 18.83-11.03 1.345 5.472-4.332 7.686-7.351 3.263-2.76 4.964-7.432 3.146-11.479m-22.542 2.1c5.464 4.677-3.312 12.022-.78 16.973-3.124 3.56-10.45 4.482-4.178-.68 3.37-4.49 7.023-10.51 4.958-16.294m-7.473 2.435-.042.042zm-.085.028-.155.087zm20.402.882c-1.302 2.602-3.654.316 0 0m-7.01 6.548-.042.042zm2.183.168-.042.042zm-.21.084-.042.042zm-2.603.546.042.042z"
                                    style="fill:#50C878;stroke:#020202;stroke-width:0.5"
                                    transform="translate(-101.376 -111.263)"
                                />
                            </svg>
                        </div>

                        <button id="menuButton" class="text-emerald focus:outline-none">
                            // Hamburger Icon
                            <svg
                                class="w-8 h-8 transition-transform duration-300 transform"
                                id="hamburgerIcon"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                                xmlns="http://www.w3.org/2000/svg"
                            >
                                <path
                                    strokeLinecap="round"
                                    strokeLinejoin="round"
                                    strokeWidth="2"
                                    d="M4 6h16M4 12h16M4 18h16"
                                ></path>
                            </svg>
                            // Close Icon
                            <svg
                                class="w-8 h-8 hidden transition-transform duration-300 transform"
                                id="closeIcon"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                                xmlns="http://www.w3.org/2000/svg"
                            >
                                <path
                                    strokeLinecap="round"
                                    strokeLinejoin="round"
                                    strokeWidth="2"
                                    d="M6 18L18 6M6 6l12 12"
                                ></path>
                            </svg>
                        </button>
                    </div>

                    // Mobile Menu
                    <div
                        id="mobileMenu"
                        class="hidden mt-4 pb-4 space-y-2 overflow-hidden transition-all duration-300 ease-in-out max-h-0"
                    >
                        <a
                            href="#"
                            class="block px-4 py-2 text-emerald bg-emerald-dark bg-opacity-30 rounded-lg transition-all duration-300 hover:bg-opacity-50 hover:text-gold"
                        >
                            Home
                        </a>
                        <a
                            href="#"
                            class="block px-4 py-2 text-emerald bg-emerald-dark bg-opacity-30 rounded-lg transition-all duration-300 hover:bg-opacity-50 hover:text-gold"
                        >
                            About
                        </a>
                        <a
                            href="#"
                            class="block px-4 py-2 text-emerald bg-emerald-dark bg-opacity-30 rounded-lg transition-all duration-300 hover:bg-opacity-50 hover:text-gold"
                        >
                            Blog
                        </a>
                        <a
                            href="#"
                            class="block px-4 py-2 text-emerald bg-emerald-dark bg-opacity-30 rounded-lg transition-all duration-300 hover:bg-opacity-50 hover:text-gold"
                        >
                            Contact
                        </a>
                    </div>
                </div>
            </nav>
        </header>
    }
}
