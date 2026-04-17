use crate::models::Series;
use leptos::prelude::*;

#[component]
pub fn SeriesCard(
    series: Series,
    /// (owned_count, total_albums) — shown as badge when Some.
    ownership: Option<(i64, i64)>,
    /// When set, appends ?owner=username to the link (for viewing another user's collection).
    #[prop(optional)]
    owner: Option<String>,
    #[prop(optional, default = 0)] for_sale_count: i64,
    #[prop(optional)] is_terminated: bool,
) -> impl IntoView {
    let base = if !series.slug.is_empty() {
        format!("/series/{}", series.slug)
    } else if let Some(bid) = &series.bubble_id {
        format!("/series/bubble/{}", urlencoding::encode(bid))
    } else {
        "#".to_string()
    };
    let href = match owner {
        Some(u) if base != "#" => format!("{base}?owner={}", urlencoding::encode(&u)),
        _ => base,
    };

    view! {
        <a href=href data-nav-item="" class="block bg-white dark:bg-gray-800 rounded-xl shadow-sm overflow-hidden hover:-translate-y-1 hover:bg-gray-100 dark:hover:bg-gray-700 transition-all">
            <div class="aspect-[2/3] bg-gradient-to-br from-gray-600 to-gray-800 flex items-center justify-center relative">
                {series.cover_url.clone().map(|url| view! {
                    <img src=url alt=series.title.clone() class="w-full h-full object-cover"/>
                })}
                {ownership.map(|(owned, total)| {
                    let (badge_text, badge_class) = if owned >= total && total > 0 && is_terminated {
                        ("Complete".to_string(), "bg-green-600")
                    } else if owned >= total && total > 0 {
                        (format!("{owned}/{total}\u{2026}"), "bg-indigo-accent")
                    } else {
                        (format!("{owned}/{total}"), "bg-indigo-accent")
                    };
                    view! {
                        <span class=format!("absolute top-2 right-2 {badge_class} text-white text-xs font-bold px-2 py-0.5 rounded-full")>
                            {badge_text}
                        </span>
                    }
                })}
                {(for_sale_count > 0).then(|| view! {
                    <span class="absolute bottom-2 right-2 bg-red-500 text-white text-xs font-bold px-2 py-0.5 rounded-full">
                        {for_sale_count}" for sale"
                    </span>
                })}
            </div>
            <div class="p-3">
                <div class="text-sm font-semibold text-gray-900 dark:text-gray-100 truncate">{series.title.clone()}</div>
                <div class="text-xs text-gray-500 dark:text-gray-400 mt-0.5 truncate">{series.author.clone()}</div>
                {series.year.map(|y| view! {
                    <div class="text-xs text-gray-400 dark:text-gray-500 mt-0.5">{y}</div>
                })}
            </div>
        </a>
    }
}
