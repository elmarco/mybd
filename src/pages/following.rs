use crate::app::{FollowingCountDisplay, FollowingCountResource};
use crate::components::Avatar;
use crate::server::social::{get_following, unfollow_user};
use leptos::either::EitherOf3;
use leptos::prelude::*;

#[component]
pub fn FollowingPage() -> impl IntoView {
    let following = Resource::new(|| (), |_| get_following());

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-6">"Friends"</h1>

            <Suspense fallback=|| view! { <p class="text-gray-500">"Loading..."</p> }>
                {move || Suspend::new(async move {
                    match following.await {
                        Ok(list) if list.is_empty() => {
                            EitherOf3::A(view! {
                                <p class="text-gray-500">"No friends yet. Search for users to add them."</p>
                            })
                        }
                        Ok(list) => {
                            EitherOf3::B(view! {
                                <div class="space-y-2">
                                    {list.into_iter().map(|f| {
                                        let target_id = f.id;
                                        let href = format!("/profile/{}", urlencoding::encode(&f.username));
                                        let removing = RwSignal::new(false);
                                        let removed = RwSignal::new(false);

                                        view! {
                                            <Show when=move || !removed.get()>
                                                <div class="flex items-center gap-4 p-4 bg-white dark:bg-gray-800 rounded-xl shadow-sm">
                                                    <a href=href.clone() class="flex items-center gap-3 flex-1 min-w-0">
                                                        <Avatar url=f.avatar_url.clone() name=f.display_name.clone() size="w-10 h-10" text_size="text-sm"/>
                                                        <div class="min-w-0">
                                                            <div class="font-medium text-gray-900 dark:text-gray-100 truncate">{f.display_name.clone()}</div>
                                                            <div class="text-sm text-gray-500 dark:text-gray-400 truncate">"@"{f.username.clone()}</div>
                                                        </div>
                                                    </a>
                                                    <button
                                                        class="px-3 py-1.5 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/30 rounded-lg transition-colors cursor-pointer"
                                                        prop:disabled=move || removing.get()
                                                        on:click=move |ev| {
                                                            ev.prevent_default();
                                                            ev.stop_propagation();
                                                            removing.set(true);
                                                            leptos::task::spawn_local(async move {
                                                                if unfollow_user(target_id).await.is_ok() {
                                                                    removed.set(true);
                                                                    if let Some(d) = use_context::<FollowingCountDisplay>() {
                                                                        d.0.update(|n| *n = n.map(|c| c - 1));
                                                                    }
                                                                    if let Some(res) = use_context::<FollowingCountResource>() {
                                                                        res.0.refetch();
                                                                    }
                                                                }
                                                                removing.set(false);
                                                            });
                                                        }
                                                    >
                                                        <span class="material-symbols-outlined" style="font-size: 18px;">"person_remove"</span>
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
