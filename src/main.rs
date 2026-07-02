use axum::Router;
use axum::routing::{get, put};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{env, fs};
use tokio::net::TcpListener;
#[cfg(debug_assertions)]
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::config::Config;
use crate::db_index::DbIndex;
use crate::jobs::build_index::BuildIndexJob;
use crate::storage::Storage;

mod attachment;
mod config;
mod db_index;
mod error;
mod item;
mod jobs;
mod routes;
mod storage;
mod validate;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub storage: Arc<Storage>,
    pub db: DbIndex,
}

async fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load()?;
    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| config.bind_address.clone());

    let data_dir = PathBuf::from("data");
    fs::create_dir_all(&data_dir)?;

    let app_state = AppState {
        config: Arc::new(config),
        storage: Arc::new(Storage::new(data_dir.clone())),
        db: Arc::new(Mutex::new(db_index::open()?)),
    };

    BuildIndexJob::new(&app_state.storage, &app_state.db).run()?;

    let app = Router::new()
        .route(
            "/api/items",
            get(routes::list_items::handler).post(routes::create_item::handler),
        )
        .route(
            "/api/items/{path}/attachments",
            get(routes::list_attachments::handler).post(routes::add_attachment::handler),
        )
        .route(
            "/api/items/{path}/status",
            put(routes::update_status::handler),
        )
        .route("/api/items/{path}/tags", put(routes::update_tags::handler))
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
