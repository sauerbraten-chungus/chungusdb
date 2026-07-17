use std::env;

use axum::{
    Router, middleware as axum_middleware,
    routing::{get, post},
};
use log::info;
use middleware::jwt_auth;
use tonic::transport::Server;

use crate::chungusdb_grpc_service::{
    ChungusDbImpl, chungusdb::chungus_db_server::ChungusDbServer,
};
use crate::handlers::*;

mod chungusdb_grpc_service;
mod db;
mod handlers;
mod logger;
mod middleware;
mod models;

#[derive(Clone)]
struct AppState {
    db: db::Db,
    secret: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    logger::init().unwrap();

    let app_state = AppState {
        db: db::Db::new().await?,
        secret: env::var("SECRET_CHUNGUS").unwrap_or_else(|_| "".to_string()),
    };

    // Spawn HTTP server (Axum) on port 3000
    let http_state = app_state.clone();
    let http_server = tokio::spawn(async move {
        let protected_routes = Router::new()
            // .route("/players/batch", post(post_batch_player_data))
            .layer(axum_middleware::from_fn_with_state(
                http_state.clone(),
                jwt_auth,
            ));

        let app = Router::new()
            .route("/", get(|| async { "Hello World!" }))
            // .route("/player", get(get_all_player_data))
            // .route("/player/{id}", get(get_player_data))
            .merge(protected_routes)
            .with_state(http_state);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        info!("HTTP server listening on 0.0.0.0:3000");

        axum::serve(listener, app).await.unwrap();
    });

    // Spawn gRPC server (Tonic) on port 50052
    let grpc_state = app_state.clone();
    let grpc_server = tokio::spawn(async move {
        let addr = "0.0.0.0:50052".parse().unwrap();
        let service = ChungusDbImpl::new(grpc_state.db);

        info!("gRPC server listening on 0.0.0.0:50052");

        Server::builder()
            .add_service(ChungusDbServer::new(service))
            .serve(addr)
            .await
            .unwrap();
    });

    // Wait for both servers (they run indefinitely)
    tokio::try_join!(http_server, grpc_server)?;

    Ok(())
}
