use axum::body::Body;
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

/// The built frontend (`frontend/dist`, produced by `just build`) embedded
/// directly into the server binary for single-file deployment.
#[derive(RustEmbed)]
#[folder = "../../frontend/dist"]
struct Assets;

/// Serves an embedded asset by its exact path, or falls back to `index.html`
/// for any unmatched non-`/api` route (SPA client-side routing support).
pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if let Some(content) = Assets::get(path) {
        return serve(path, content.data.into_owned());
    }

    match Assets::get("index.html") {
        Some(content) => serve("index.html", content.data.into_owned()),
        None => (
            StatusCode::NOT_FOUND,
            "frontend assets not embedded — run `just build` first",
        )
            .into_response(),
    }
}

fn serve(path: &str, data: Vec<u8>) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Response::builder()
        .header(header::CONTENT_TYPE, mime.as_ref())
        .body(Body::from(data))
        .unwrap()
        .into_response()
}
