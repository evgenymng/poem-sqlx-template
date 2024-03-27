mod api;
mod database;
mod model;
mod util;

use std::sync::Arc;

use database::{AuthorRepo, BookRepo};

use poem::{listener::TcpListener, Route, Server};
use poem_openapi::OpenApiService;
use sqlx::PgPool;
use util::Context;

use crate::api::route::{AuthorRoute, BookRoute, IndexRoute};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init_timed();
    dotenv::dotenv()?;

    let db_url = std::env::var("DATABASE_URL")?;
    log::info!("Establishing database connection...");
    let pool = PgPool::connect(&db_url).await?;
    log::info!("Connected to the database");
    let ctx = Arc::new(Context {
        book_repo: BookRepo { pool: pool.clone() },
        author_repo: AuthorRepo { pool },
    });

    let routes = (
        IndexRoute,
        AuthorRoute { ctx: ctx.clone() },
        BookRoute { ctx },
    );

    let api_service = OpenApiService::new(routes, "EMS", "1.0.0")
        .server("http://localhost:3000/api");
    let ui = api_service.swagger_ui();

    // BUG(vinc3nzo): OpenAPI is aiming for the wrong server (different port)
    let addr = std::env::var("LISTEN_ADDRESS")?;
    Server::new(TcpListener::bind(addr)).run(Route::new().nest("/api", api_service).nest("/docs", ui)).await?;
    Ok(())
}
