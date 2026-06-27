use axum::Router;
use axum::routing::get;
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fs};
use tokio::net::TcpListener;
#[cfg(debug_assertions)]
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::storage::Storage;

mod error;
mod item;
mod jobs;
mod routes;
mod storage;
mod validate;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<Storage>,
}

async fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    let bind_address = env::var("BIND_ADDRESS").expect("BIND_ADDRESS not set");

    let data_dir = PathBuf::from("data");
    fs::create_dir_all(&data_dir)?;

    let app_state = AppState {
        storage: Arc::new(Storage::new(data_dir.clone())),
    };

    let app = Router::new()
        .route(
            "/api/items",
            get(routes::list_items::handler).post(routes::create_item::handler),
        )
        .nest_service("/data", ServeDir::new(&data_dir));

    #[cfg(debug_assertions)]
    let app = app.layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    );

    let app = app.with_state(app_state);

    println!("Starting server on {}", bind_address);
    let listener = TcpListener::bind(&bind_address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run_app().await {
        eprintln!("Error starting app: {}", e);
        std::process::exit(1);
    }
}
