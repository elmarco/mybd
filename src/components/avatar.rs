use leptos::prelude::*;

#[component]
pub fn Avatar(
    /// The avatar URL (e.g. gravatar or custom).
    url: Option<String>,
    /// The display name, used for alt text and initials fallback.
    name: String,
    /// Tailwind size classes for the container, e.g. "w-8 h-8".
    #[prop(default = "w-8 h-8")]
    size: &'static str,
    /// Tailwind text size class for the initials fallback, e.g. "text-sm".
    #[prop(default = "text-sm")]
    text_size: &'static str,
) -> impl IntoView {
    let initial = name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();

    view! {
        <div class={format!("{size} rounded-full bg-indigo-accent flex items-center justify-center text-white {text_size} font-semibold overflow-hidden flex-shrink-0")}>
            {match url {
                Some(src) if !src.is_empty() => view! {
                    <img
                        src=src
                        alt=name
                        class={format!("{size} rounded-full object-cover")}
                    />
                }.into_any(),
                _ => initial.into_any(),
            }}
        </div>
    }
}
