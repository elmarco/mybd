use leptos::prelude::*;

/// Context signal to open/close the keyboard shortcuts dialog from anywhere.
#[derive(Clone, Copy)]
pub struct HelpDialogOpen(pub RwSignal<bool>);

#[component]
pub fn HelpDialog() -> impl IntoView {
    let dialog_open = expect_context::<HelpDialogOpen>().0;

    let shortcuts = [
        ("j / k", "Next / previous item"),
        ("Enter / o", "Open selected item"),
        ("Space", "Toggle album (when selected)"),
        ("1 / 2 / 3", "Switch tabs (on search page)"),
        ("/", "Focus search bar"),
        ("?", "Show this help"),
        ("Escape", "Close dialog / clear selection"),
        ("g then c", "Go to collection"),
        ("g then h", "Go to collection"),
        ("g then s", "Go to settings"),
        ("g then f", "Go to friends"),
        ("g then w", "Go to world map"),
    ];

    view! {
        <Show when=move || dialog_open.get()>
            <div
                class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
                on:click=move |_| dialog_open.set(false)
            >
                <div
                    class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6 max-w-sm w-full mx-4"
                    on:click=move |ev| ev.stop_propagation()
                >
                    <div class="flex items-center justify-between mb-4">
                        <h2 class="text-lg font-bold text-gray-900 dark:text-gray-100">"Keyboard shortcuts"</h2>
                        <button
                            class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                            on:click=move |_| dialog_open.set(false)
                        >
                            <span class="material-symbols-outlined">"close"</span>
                        </button>
                    </div>
                    <div class="space-y-2">
                        {shortcuts.into_iter().map(|(key, desc)| view! {
                            <div class="flex items-center justify-between py-1.5">
                                <span class="text-sm text-gray-700 dark:text-gray-300">{desc}</span>
                                <kbd class="ml-4 px-2 py-0.5 bg-gray-100 dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded text-xs font-mono text-gray-600 dark:text-gray-300 whitespace-nowrap">
                                    {key}
                                </kbd>
                            </div>
                        }).collect_view()}
                    </div>
                </div>
            </div>
        </Show>
    }
}
