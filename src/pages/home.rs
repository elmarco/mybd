use crate::components::login_dialog::LoginDialogOpen;
use crate::models::UserPublic;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn HomePage() -> impl IntoView {
    let login_dialog = expect_context::<LoginDialogOpen>().0;
    let user = expect_context::<Resource<Result<Option<UserPublic>, ServerFnError>>>();
    let navigate = use_navigate();

    // Redirect to /collection if user is logged in
    Effect::new(move || {
        if let Some(Ok(Some(_))) = user.get() {
            navigate("/collection", Default::default());
        }
    });

    view! {
        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <div class="max-w-4xl mx-auto px-4 py-20 text-center">
                <img src="/mybd.svg" alt="mybd" class="w-1/3 mx-auto mb-4"/>
                <p class="text-xl text-gray-600 dark:text-gray-300 mb-8">
                    "Track your comics, manga, and graphic novels in one place."
                </p>
                <div class="space-y-4 max-w-md mx-auto">
                    <a
                        href="/register"
                        class="block w-full py-3 px-6 rounded-lg text-white bg-indigo-accent hover:bg-indigo-accent-dark font-medium text-lg"
                    >
                        "Get Started"
                    </a>
                    <button
                        class="block w-full py-3 px-6 rounded-lg text-indigo-accent border-2 border-indigo-accent hover:bg-gray-100 dark:hover:bg-gray-800 font-medium text-lg cursor-pointer"
                        on:click=move |_| login_dialog.set(true)
                    >
                        "Sign In"
                    </button>
                </div>

                <div class="mt-20 grid grid-cols-1 md:grid-cols-3 gap-8 text-left">
                    <div class="bg-white dark:bg-gray-800 p-6 rounded-xl shadow-sm">
                        <div class="text-2xl mb-3 text-comic-blue font-bold">"BDs, Manga, ..."</div>
                        <p class="text-gray-600 dark:text-gray-300">"Catalog your collection"</p>
                    </div>
                    <div class="bg-white dark:bg-gray-800 p-6 rounded-xl shadow-sm">
                        <div class="text-2xl mb-3 text-manga-purple font-bold">"Social"</div>
                        <p class="text-gray-600 dark:text-gray-300">"Browse what your friends own, and manage your lent items"</p>
                    </div>
                    <div class="bg-white dark:bg-gray-800 p-6 rounded-xl shadow-sm">
                        <div class="text-2xl mb-3 text-gn-green font-bold">"Discover"</div>
                        <p class="text-gray-600 dark:text-gray-300">"Find new books to read based on your interests and recommendations"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}
