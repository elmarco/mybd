use leptos::prelude::*;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center py-24">
            <div class="text-center">
                <a href="/">
                    <img src="/mybd.svg" alt="mybd" class="w-48 h-48 mx-auto mb-8"/>
                </a>
                <h1 class="text-6xl font-bold text-gray-300 dark:text-gray-600 mb-4">"404"</h1>
                <p class="text-xl text-gray-600 dark:text-gray-400 mb-6">"Page not found"</p>
                <a
                    href="/"
                    class="inline-block px-6 py-3 rounded-lg text-white bg-indigo-accent hover:bg-indigo-accent-dark font-medium"
                >
                    "Go Home"
                </a>
            </div>
        </div>
    }
}
