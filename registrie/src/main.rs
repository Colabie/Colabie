mod db;
mod errors;
mod utils;

use db::DB;
use errors::*;
pub use utils::Schemou;

use schemou::*;

use axum::{
    extract::State,
    http::{header, Method},
    routing::post,
    Router,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use tower_http::{cors, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const DB_PATH: &str = "db";

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = DB::get_or_create(DB_PATH);

    let cors = cors::CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(cors::Any)
        .allow_headers([header::CONTENT_TYPE]);

    let router = Router::new()
        .route("/register", post(register))
        .with_state(db)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                // By default `TraceLayer` will log 5xx responses but we're doing our specific
                // logging of errors so disable that
                .on_failure(()),
        );

    let address = "0.0.0.0:8081";
    let listner = tokio::net::TcpListener::bind(address).await.unwrap();
    tracing::info!("listening on: http://{}\n", address);
    axum::serve(listner, router).await.unwrap();
}

async fn register(
    State(db): State<DB>,
    Schemou(register_req): Schemou<RegisterReq>,
) -> RegistrieResult<Schemou<RegisterRes>> {
    // TODO: Validation of user requests and fields
    // labels: help wanted
    // Issue URL: https://github.com/Colabie/Colabie/issues/10
    let pubkey = BASE64_STANDARD.encode(&register_req.pubkey);
    let commit_id = db
        .new_record((*register_req.username).clone(), pubkey)
        .await
        .as_bytes()
        .into();

    Ok(Schemou(RegisterRes { commit_id }))
}
