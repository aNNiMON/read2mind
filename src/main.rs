use axum::Router;
use axum::routing::{delete, get, post, put};
use reqwest::Client;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{env, fs};
use tokio::net::TcpListener;
#[cfg(debug_assertions)]
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

use crate::config::Config;
use crate::db_index::DbIndex;
use crate::jobs::build_index::BuildIndexJob;
use crate::storage::Storage;

mod config;
mod db_index;
mod error;
mod jobs;
mod model;
mod routes;
mod storage;
mod validate;

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub config: Arc<Config>,
    pub db: DbIndex,
    pub storage: Arc<Storage>,
}

async fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=info,tower_http=info,axum::rejection=info",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::load()?;
    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| config.bind_address.clone());

    let client = Client::builder().build()?;

    let data_dir = PathBuf::from("data");
    fs::create_dir_all(&data_dir)?;

    let app_state = AppState {
        client,
        config: Arc::new(config),
        db: Arc::new(Mutex::new(db_index::open()?)),
        storage: Arc::new(Storage::new(data_dir.clone())),
    };

    BuildIndexJob::new(&app_state.storage, &app_state.db).run()?;

    let app = Router::new()
        .route(
            "/api/items",
            get(routes::list_items::handler).post(routes::create_item::handler),
        )
        .route("/api/items/{path}", delete(routes::delete_item::handler))
        .route(
            "/api/items/{path}/attachments",
            get(routes::list_attachments::handler).post(routes::add_attachment::handler),
        )
        .route(
            "/api/items/{path}/attachments/{name}",
            delete(routes::delete_attachment::handler),
        )
        .route(
            "/api/items/{path}/status",
            put(routes::update_status::handler),
        )
        .route("/api/items/{path}/ai", post(routes::ai_generate::handler))
        .route("/api/items/{path}/tags", put(routes::update_tags::handler))
        .route("/api/tags", get(routes::list_tags::handler))
        .route("/api/health", get(routes::health::handler))
        .route("/api/config/ai", get(routes::config_ai::handler))
        .nest_service("/data", ServeDir::new(&data_dir));

    #[cfg(debug_assertions)]
    let app = app.layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    );

    let app = app.layer(TraceLayer::new_for_http()).with_state(app_state);

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
