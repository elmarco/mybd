use crate::app::LentCountResource;
use crate::server::social::{get_lent_albums, return_album};
use leptos::either::EitherOf3;
use leptos::prelude::*;

#[component]
pub fn LentPage() -> impl IntoView {
    let loans = Resource::new(|| (), |_| get_lent_albums());

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-6">"Lent Albums"</h1>

            <Suspense fallback=|| view! { <p class="text-gray-500">"Loading..."</p> }>
                {move || Suspend::new(async move {
                    match loans.await {
                        Ok(list) if list.is_empty() => {
                            EitherOf3::A(view! {
                                <p class="text-gray-500">"No albums currently lent out."</p>
                            })
                        }
                        Ok(list) => {
                            EitherOf3::B(view! {
                                <div class="space-y-2">
                                    {list.into_iter().map(|loan| {
                                        let loan_id = loan.id;
                                        let returning = RwSignal::new(false);
                                        let returned = RwSignal::new(false);
                                        let album_href = format!("/album/{}", loan.album_slug);
                                        let borrower_href = format!("/profile/{}", urlencoding::encode(&loan.borrower_username));
                                        let title = loan.album_title.unwrap_or_else(|| loan.series_title.clone());
                                        let series_title = loan.series_title;
                                        let cover_url = loan.cover_url;
                                        let borrower_name = loan.borrower_display_name;

                                        view! {
                                            <Show when=move || !returned.get()>
                                                <div class="flex items-center gap-4 p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm">
                                                    <a href=album_href.clone() class="flex-shrink-0">
                                                        {cover_url.as_ref().map(|url| view! {
                                                            <img src=url.clone() alt="" class="w-12 h-16 object-cover rounded"/>
                                                        })}
                                                    </a>
                                                    <div class="flex-1 min-w-0">
                                                        <a href=album_href.clone() class="font-medium text-gray-900 dark:text-gray-100 truncate block hover:underline">
                                                            {title.clone()}
                                                        </a>
                                                        <div class="text-sm text-gray-500 dark:text-gray-400">
                                                            {series_title.clone()}
                                                        </div>
                                                        <div class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">
                                                            "Lent to "
                                                            <a href=borrower_href.clone() class="text-indigo-accent hover:underline">
                                                                {borrower_name.clone()}
                                                            </a>
                                                        </div>
                                                    </div>
                                                    <button
                                                        class="px-3 py-1.5 text-sm text-green-700 dark:text-green-400 bg-green-50 dark:bg-green-900/30 hover:bg-green-100 dark:hover:bg-green-900/50 rounded-lg transition-colors cursor-pointer"
                                                        prop:disabled=move || returning.get()
                                                        on:click=move |ev| {
                                                            ev.prevent_default();
                                                            ev.stop_propagation();
                                                            returning.set(true);
                                                            leptos::task::spawn_local(async move {
                                                                if return_album(loan_id).await.is_ok() {
                                                                    returned.set(true);
                                                                    if let Some(res) = use_context::<LentCountResource>() {
                                                                        res.0.refetch();
                                                                    }
                                                                }
                                                                returning.set(false);
                                                            });
                                                        }
                                                    >
                                                        "Return"
                                                    </button>
                                                </div>
                                            </Show>
                                        }
                                    }).collect_view()}
                                </div>
                            })
                        }
                        Err(e) => EitherOf3::C(view! {
                            <div class="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
                                {format!("Error: {e}")}
                            </div>
                        }),
                    }
                })}
            </Suspense>
        </div>
    }
}
