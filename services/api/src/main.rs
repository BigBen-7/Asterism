use std::sync::Arc;

use asterism_api::{
    chain::PgChainReader,
    routes::{router, AppState},
    Error,
};
use sqlx::postgres::PgPoolOptions;

fn env(name: &'static str) -> Result<String, Error> {
    std::env::var(name).map_err(|_| Error::MissingEnv(name))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env("DATABASE_URL")?)
        .await?;
    let registry_id = env("REGISTRY_CONTRACT_ID")?;
    let state = Arc::new(AppState {
        reader: PgChainReader::new(pool, registry_id.clone()),
        registry_id,
        credentials_id: env("CREDENTIALS_CONTRACT_ID")?,
        verify_base_url: std::env::var("VERIFY_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string()),
    });

    let bind = std::env::var("API_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    println!("asterism-api listening on {bind}");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
