use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tower_http::services::ServeDir;
use tracing::{info, warn};

#[derive(Debug)]
struct HttpServeState {
    path: PathBuf,
}

pub async fn process_http_serve(path: PathBuf, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Serving {:?} on {}", path, addr);

    let state = HttpServeState { path: path.clone() };
    // axum router
    let router = Router::new()
        .nest_service("/tower", ServeDir::new(path))
        .route("/*path", get(file_handler))
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

async fn file_handler(
    State(state): State<Arc<HttpServeState>>,
    Path(path): Path<String>,
) -> (StatusCode, Response) {
    let p = std::path::Path::new(&state.path).join(path);
    info!("Reading file {:?}", p);
    if !p.exists() {
        (
            StatusCode::NOT_FOUND,
            format!("File {} note found", p.display()).into_response(),
        )
    } else {
        if p.is_dir() {
            let mut dir = tokio::fs::read_dir(&p).await.unwrap();
            let mut res = String::from("<html><head><title>Directory Listing</title></head><body>");
            res.push_str(format!("<h1>Directory Listing: {}</h1><ul>", p.display()).as_str());

            while let Some(entry) = dir.next_entry().await.unwrap() {
                let file_name = entry.file_name().into_string().unwrap();
                if !file_name.starts_with('.') {
                    res.push_str(
                        format!(
                            "<li><a href=\"/{}/{}\">{}</a></li>",
                            p.to_str().unwrap(),
                            file_name,
                            file_name
                        )
                        .as_str(),
                    );
                }
            }

            res.push_str("</ul></body></html>");
            return (StatusCode::OK, Html(res).into_response());
        }

        match tokio::fs::read_to_string(p).await {
            Ok(content) => {
                info!("Read {} bytes", content.len());
                (StatusCode::OK, content.into_response())
            }
            Err(e) => {
                warn!("Error reading file: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    e.to_string().into_response(),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_handler() {
        let state = Arc::new(HttpServeState {
            path: PathBuf::from("."),
        });
        let (status, resp) = file_handler(State(state), Path("Cargo.toml".to_string())).await;
        assert_eq!(status, StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let content = String::from_utf8(body.to_vec()).unwrap();

        assert!(content.starts_with("[package]"));
    }
}
