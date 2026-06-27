use axum::{
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "fontend_dist/"] // fontend release
#[allow_missing = true]
struct Asset;

pub async fn handle(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.is_empty() {
        path = "index.html".to_string();
    }

    match Asset::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();

            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(axum::body::Body::from(content.data))
                .unwrap()
        }
        None => {
            if let Some(index) = Asset::get("index.html") {
                Response::builder()
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(axum::body::Body::from(index.data))
                    .unwrap()
            } else {
                (StatusCode::NOT_FOUND, "404 Not Found").into_response()
            }
        }
    }
}
