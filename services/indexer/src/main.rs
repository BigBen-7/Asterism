use asterism_indexer::{connect, migrate, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| Error::MissingEnv("DATABASE_URL"))?;
    let pool = connect(&database_url).await?;
    migrate(&pool).await?;
    println!("asterism-indexer: database ready (event ingestion is not implemented yet)");
    Ok(())
}
