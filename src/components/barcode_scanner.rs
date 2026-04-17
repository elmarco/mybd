use leptos::prelude::*;

/// A barcode scanner modal that uses Quagga2 (loaded from CDN) to scan EAN-13 barcodes.
/// Calls `on_scan` with the decoded EAN string when a barcode is detected.
#[component]
pub fn BarcodeScanner(
    #[prop(into)] on_scan: Callback<String>,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let scanner_active = RwSignal::new(false);
    let error_msg: RwSignal<Option<String>> = RwSignal::new(None);

    Effect::new(move || {
        start_scanner(scanner_active, error_msg, on_scan, on_close);
    });

    on_cleanup(move || {
        stop_scanner();
    });

    view! {
        <style>
            "#barcode-scanner video, #barcode-scanner canvas {
                width: 100% !important;
                height: 100% !important;
                object-fit: cover !important;
                display: block;
            }
            #barcode-scanner > canvas.drawingBuffer {
                position: absolute !important;
                top: 0 !important;
                left: 0 !important;
            }"
        </style>
        <div class="fixed inset-0 z-50 bg-black/80 flex items-center justify-center p-4">
            <div class="bg-white rounded-2xl overflow-hidden max-w-md w-full shadow-2xl">
                <div class="flex items-center justify-between px-4 py-3 bg-gray-50 border-b">
                    <h3 class="font-semibold text-gray-900">"Scan barcode"</h3>
                    <button
                        class="text-gray-400 hover:text-gray-600 text-2xl leading-none"
                        on:click=move |_| {
                            stop_scanner();
                            on_close.run(());
                        }
                    >
                        "×"
                    </button>
                </div>
                <div class="relative overflow-hidden">
                    <div id="barcode-scanner" class="w-full h-64 bg-black relative"></div>
                    <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                        <div class="w-3/4 h-16 border-2 border-red-400/70 rounded-lg"></div>
                    </div>
                    {move || {
                        if !scanner_active.get() && error_msg.get().is_none() {
                            Some(view! {
                                <div class="absolute inset-0 flex items-center justify-center bg-black/50">
                                    <p class="text-white text-sm">"Starting camera..."</p>
                                </div>
                            })
                        } else {
                            None
                        }
                    }}
                </div>
                {move || error_msg.get().map(|msg| view! {
                    <div class="px-4 py-3 bg-red-50 text-red-700 text-sm">{msg}</div>
                })}
                <div class="px-4 py-3 text-center text-xs text-gray-500">
                    "Point your camera at an EAN-13 barcode"
                </div>
            </div>
        </div>
    }
}

fn start_scanner(
    _scanner_active: RwSignal<bool>,
    _error_msg: RwSignal<Option<String>>,
    _on_scan: Callback<String>,
    _on_close: Callback<()>,
) {
    #[cfg(feature = "hydrate")]
    {
        let scanner_active = _scanner_active;
        let error_msg = _error_msg;
        let on_scan = _on_scan;
        let on_close = _on_close;

        use wasm_bindgen::JsValue;
        use wasm_bindgen::prelude::*;

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();

        let quagga_val = js_sys::Reflect::get(&window, &JsValue::from_str("Quagga"))
            .unwrap_or(JsValue::UNDEFINED);

        if !quagga_val.is_undefined() {
            init_quagga(scanner_active, error_msg, on_scan, on_close);
        } else {
            let script = document.create_element("script").unwrap();
            script
                .set_attribute(
                    "src",
                    "https://cdn.jsdelivr.net/npm/@ericblade/quagga2/dist/quagga.min.js",
                )
                .unwrap();

            let on_load = Closure::once(move || {
                init_quagga(scanner_active, error_msg, on_scan, on_close);
            });
            script
                .add_event_listener_with_callback("load", on_load.as_ref().unchecked_ref())
                .unwrap();
            on_load.forget();

            let on_error = Closure::once(move || {
                error_msg.set(Some("Failed to load barcode scanner library".to_string()));
            });
            script
                .add_event_listener_with_callback("error", on_error.as_ref().unchecked_ref())
                .unwrap();
            on_error.forget();

            document.head().unwrap().append_child(&script).unwrap();
        }
    }
}

#[cfg(feature = "hydrate")]
fn js_obj(entries: &[(&str, wasm_bindgen::JsValue)]) -> wasm_bindgen::JsValue {
    use wasm_bindgen::JsValue;
    let obj = js_sys::Object::new();
    for (key, val) in entries {
        js_sys::Reflect::set(&obj, &JsValue::from_str(key), val).unwrap();
    }
    obj.into()
}

#[cfg(feature = "hydrate")]
fn init_quagga(
    scanner_active: RwSignal<bool>,
    error_msg: RwSignal<Option<String>>,
    on_scan: Callback<String>,
    on_close: Callback<()>,
) {
    use wasm_bindgen::JsValue;
    use wasm_bindgen::prelude::*;

    let window = web_sys::window().unwrap();
    let quagga: JsValue = js_sys::Reflect::get(&window, &JsValue::from_str("Quagga")).unwrap();

    // Build config object programmatically (no eval)
    let target: JsValue = window
        .document()
        .unwrap()
        .query_selector("#barcode-scanner")
        .unwrap()
        .map(JsValue::from)
        .unwrap_or(JsValue::NULL);

    let constraints = js_obj(&[("facingMode", JsValue::from_str("environment"))]);

    let input_stream = js_obj(&[
        ("name", JsValue::from_str("Live")),
        ("type", JsValue::from_str("LiveStream")),
        ("target", target),
        ("constraints", constraints),
    ]);

    let readers = js_sys::Array::new();
    readers.push(&JsValue::from_str("ean_reader"));

    let decoder = js_obj(&[("readers", readers.into())]);

    let config = js_obj(&[
        ("inputStream", input_stream),
        ("decoder", decoder),
        ("locate", JsValue::TRUE),
        ("frequency", JsValue::from_f64(10.0)),
    ]);

    let init_fn =
        js_sys::Function::from(js_sys::Reflect::get(&quagga, &JsValue::from_str("init")).unwrap());

    let quagga_clone = quagga.clone();
    let callback = Closure::once(move |err: JsValue| {
        if !err.is_null() && !err.is_undefined() {
            error_msg.set(Some("Camera access denied or unavailable".to_string()));
            return;
        }

        // Quagga.start()
        let start_fn = js_sys::Function::from(
            js_sys::Reflect::get(&quagga_clone, &JsValue::from_str("start")).unwrap(),
        );
        start_fn.call0(&quagga_clone).unwrap();
        scanner_active.set(true);

        // Quagga.onDetected(callback)
        let quagga_for_detected = quagga_clone.clone();
        let detected_callback = Closure::<dyn Fn(JsValue)>::new(move |result: JsValue| {
            let code_result: JsValue =
                match js_sys::Reflect::get(&result, &JsValue::from_str("codeResult")) {
                    Ok(v) => v,
                    Err(_) => return,
                };
            let code: JsValue = match js_sys::Reflect::get(&code_result, &JsValue::from_str("code"))
            {
                Ok(v) => v,
                Err(_) => return,
            };
            if let Some(ean) = code.as_string() {
                // Stop scanning and return result
                let stop_fn = js_sys::Function::from(
                    js_sys::Reflect::get(&quagga_for_detected, &JsValue::from_str("stop")).unwrap(),
                );
                let _ = stop_fn.call0(&quagga_for_detected);
                on_scan.run(ean);
                on_close.run(());
            }
        });

        let on_detected_fn = js_sys::Function::from(
            js_sys::Reflect::get(&quagga_clone, &JsValue::from_str("onDetected")).unwrap(),
        );
        on_detected_fn
            .call1(&quagga_clone, detected_callback.as_ref())
            .unwrap();
        detected_callback.forget();
    });

    init_fn.call2(&quagga, &config, callback.as_ref()).unwrap();
    callback.forget();
}

fn stop_scanner() {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsValue;

        let window = web_sys::window().unwrap();
        let quagga: JsValue = js_sys::Reflect::get(&window, &JsValue::from_str("Quagga"))
            .unwrap_or(JsValue::UNDEFINED);

        if !quagga.is_undefined() {
            let stop_fn: JsValue = js_sys::Reflect::get(&quagga, &JsValue::from_str("stop"))
                .unwrap_or(JsValue::UNDEFINED);
            if stop_fn.is_function() {
                let stop_fn = js_sys::Function::from(stop_fn);
                let _ = stop_fn.call0(&quagga);
            }
        }
    }
}
