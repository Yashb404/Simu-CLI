//! CastImportButton — a simple file-picker that uploads a `.cast` file to the backend

use gloo_net::http::Request;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::RequestCredentials;

use crate::api::api_base;
use shared::dto::demo_dto::ImportCastResponse;

const MAX_CAST_UPLOAD_BYTES: u64 = 5 * 1024 * 1024;
// ── Upload state machine ──────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum UploadState {
    Idle,
    Uploading,
    Success(String),
    Error(String),
}

impl UploadState {
    fn display_text(&self) -> String {
        match self {
            Self::Idle => "Click to select .cast file".to_string(),
            Self::Uploading => "Uploading...".to_string(),
            Self::Success(_) => "Imported!".to_string(),
            Self::Error(msg) => msg.clone(),
        }
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn CastImportButton(
    demo_id: String,
    on_success: Callback<ImportCastResponse>,
) -> impl IntoView {
    let state = RwSignal::new(UploadState::Idle);
    let is_dragging = RwSignal::new(false);
    let input_id = "cast-file-input";

    let start_upload = Callback::new({
        let demo_id = demo_id.clone();
        move |file: web_sys::File| {
            if let Err(msg) = validate_cast_file(&file) {
                state.set(UploadState::Error(msg));
                return;
            }

            let demo_id = demo_id.clone();
            let state_clone = state;
            let on_success_clone = on_success;

            spawn_local(async move {
                upload_file(&file, &demo_id, state_clone, on_success_clone).await;
            });
        }
    });

    let on_file_input = {
        move |ev: leptos::ev::Event| {
            let target = event_target::<web_sys::HtmlInputElement>(&ev);
            if let Some(files) = target.files() {
                if let Some(file) = files.get(0) {
                    start_upload.run(file);
                }
            }
        }
    };

    let reset_status = move |_| {
        if matches!(state.get(), UploadState::Success(_) | UploadState::Error(_)) {
            state.set(UploadState::Idle);
        }
    };

    view! {
        <div
            class=move || {
                if is_dragging.get() {
                    "rounded-[28px] border border-dashed border-primary bg-primary/10 p-5 shadow-[0_24px_90px_-48px_rgba(74,225,118,0.5)] transition-all duration-200 ease-out"
                } else {
                    "rounded-[28px] border border-dashed border-outline bg-surface-container-low/90 p-5 shadow-[0_24px_90px_-52px_rgba(0,0,0,0.95)] transition-all duration-200 ease-out hover:border-primary/70 hover:bg-surface-container"
                }
            }
            on:dragover=move |ev: web_sys::DragEvent| {
                ev.prevent_default();
                is_dragging.set(true);
            }
            on:dragleave=move |ev: web_sys::DragEvent| {
                ev.prevent_default();
                is_dragging.set(false);
            }
            on:drop=move |ev: web_sys::DragEvent| {
                ev.prevent_default();
                is_dragging.set(false);
                if let Some(file) = ev
                    .data_transfer()
                    .and_then(|transfer| transfer.files())
                    .and_then(|files| files.get(0))
                {
                    start_upload.run(file);
                }
            }
        >
            <input
                id=input_id
                type="file"
                accept=".cast"
                class="hidden"
                on:change=on_file_input
            />
            <label for=input_id class="flex cursor-pointer items-center gap-4">
                <span class="grid h-12 w-12 shrink-0 place-items-center rounded-2xl border border-outline-variant bg-surface-container-high font-mono text-sm font-bold text-primary transition-transform duration-200 group-hover:scale-105">
                    "CAST"
                </span>
                <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2 font-headline text-base font-semibold text-on-surface">
                        {move || {
                            match state.get() {
                                UploadState::Uploading => {
                                    view! { <span class="animate-pulse">"Uploading cast file..."</span> }
                                        .into_any()
                                }
                                UploadState::Success(_) => {
                                    view! { <span>"Import complete"</span> }.into_any()
                                }
                                _ => {
                                    view! { <span>"Drop a .cast file or browse"</span> }.into_any()
                                }
                            }
                        }}
                    </div>
                    <p class="mt-1 text-sm text-on-surface-variant">
                        {move || state.get().display_text()}
                        <span class="ml-2 text-xs text-on-surface-variant/70">"Max 5 MB"</span>
                    </p>
                </div>
            </label>

            {move || {
                let st = state.get();
                match &st {
                    UploadState::Success(msg) | UploadState::Error(msg) => {
                        view! {
                            <div
                                class={
                                    match st {
                                        UploadState::Success(_) => {
                                            "mt-4 flex items-center justify-between gap-3 rounded-2xl border border-primary/30 bg-primary/10 px-4 py-3 text-sm text-primary"
                                        }
                                        _ => "mt-4 flex items-center justify-between gap-3 rounded-2xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-200",
                                    }
                                }
                            >
                                <span>{msg.clone()}</span>
                                <button class="rounded-full border border-current/30 px-2 py-1 text-[10px] font-bold uppercase tracking-[0.16em] transition-colors duration-200 hover:bg-white/10" on:click=reset_status>
                                    "X"
                                </button>
                            </div>
                        }
                        .into_any()
                    }
                    _ => view! { <></> }.into_any(),
                }
            }}
        </div>
    }
}

// ── File upload logic ──────────────────────────────────────────────────────────

async fn upload_file(
    file: &web_sys::File,
    demo_id: &str,
    state: RwSignal<UploadState>,
    on_success: Callback<ImportCastResponse>,
) {
    state.set(UploadState::Uploading);

    if let Err(msg) = validate_cast_file(file) {
        state.set(UploadState::Error(msg));
        return;
    }

    match read_file_as_string(file).await {
        Ok(text) => match post_cast_file(demo_id, &file.name(), &text).await {
            Ok(response) => {
                on_success.run(response.clone());
                state.set(UploadState::Success(response.message));
            }
            Err(e) => {
                state.set(UploadState::Error(format!("Upload failed: {}", e)));
            }
        },
        Err(e) => {
            state.set(UploadState::Error(format!("Read failed: {}", e)));
        }
    }
}

fn validate_cast_file(file: &web_sys::File) -> Result<(), String> {
    let file_name = file.name();
    let is_cast = file_name.to_ascii_lowercase().ends_with(".cast");
    if !is_cast {
        return Err("Only .cast files are accepted".to_string());
    }

    if file.size() > MAX_CAST_UPLOAD_BYTES as f64 {
        return Err(format!(
            "File too large. Max allowed is {} MB",
            MAX_CAST_UPLOAD_BYTES / (1024 * 1024)
        ));
    }

    Ok(())
}
/// Read a `web_sys::File` as a UTF-8 string.
async fn read_file_as_string(file: &web_sys::File) -> Result<String, String> {
    use wasm_bindgen_futures::JsFuture;

    // Use the native arrayBuffer() method on File (which is a Blob subclass)
    let array_buffer_promise = file.array_buffer();
    let array_buffer = JsFuture::from(array_buffer_promise)
        .await
        .map_err(|_| "Failed to read array buffer")?;

    let array = js_sys::Uint8Array::new(&array_buffer);
    let bytes = array.to_vec();

    String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8: {}", e))
}

/// POST the cast file text to the backend.
async fn post_cast_file(
    demo_id: &str,
    file_name: &str,
    cast_text: &str,
) -> Result<ImportCastResponse, String> {
    let url = format!(
        "{}/api/demos/{}/import-cast?strip_trailing_prompt=true",
        api_base(),
        demo_id
    );

    // Create a simple multipart form body manually
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let mut body = String::new();

    // Add file field
    body.push_str(&format!("--{}\r\n", boundary));
    body.push_str(&format!(
        "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n",
        file_name
    ));
    body.push_str("Content-Type: text/plain\r\n\r\n");
    body.push_str(cast_text);
    body.push_str(&format!("\r\n--{}--\r\n", boundary));

    let request = Request::post(&url)
        .credentials(RequestCredentials::Include)
        .header(
            "Content-Type",
            &format!("multipart/form-data; boundary={}", boundary),
        )
        .body(body)
        .map_err(|e| format!("Request build failed: {}", e))?;

    let response = request
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("Server error: {}", response.status()));
    }

    response
        .json::<ImportCastResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}
