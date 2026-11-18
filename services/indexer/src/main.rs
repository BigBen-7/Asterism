use std::time::Duration;

use asterism_indexer::{
    connect, migrate, rpc::RpcClient, sync, Error, SyncConfig, BACKFILL_CURSOR, DEFAULT_CURSOR,
};

const DEFAULT_RPC_URL: &str = "https://soroban-testnet.stellar.org";

fn env(name: &'static str) -> Result<String, Error> {
    std::env::var(name).map_err(|_| Error::MissingEnv(name))
}

fn env_number<T: std::str::FromStr>(name: &'static str) -> Result<Option<T>, Error> {
    match std::env::var(name) {
        Ok(value) => value
            .parse()
            .map(Some)
            .map_err(|_| Error::InvalidEnv { name, value }),
        Err(_) => Ok(None),
    }
}

/// `asterism-indexer` follows the registry forever; `asterism-indexer backfill <ledger>`
/// loads history from `<ledger>` once (its own cursor) and exits.
#[tokio::main]
async fn main() -> Result<(), Error> {
    let pool = connect(&env("DATABASE_URL")?).await?;
    migrate(&pool).await?;

    let rpc = RpcClient::new(std::env::var("STELLAR_RPC_URL").unwrap_or(DEFAULT_RPC_URL.into()));
    let config = SyncConfig {
        contract_id: env("REGISTRY_CONTRACT_ID")?,
        page_limit: 200,
    };

    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("backfill") {
        let value = args.get(2).cloned().unwrap_or_default();
        let from: i64 = value.parse().map_err(|_| Error::InvalidEnv {
            name: "backfill ledger",
            value,
        })?;
        let report = sync(&pool, &rpc, &config, BACKFILL_CURSOR, Some(from)).await?;
        println!(
            "backfill done: {} events fetched, {} new",
            report.fetched, report.inserted
        );
        return Ok(());
    }

    let start_ledger = env_number::<i64>("INDEXER_START_LEDGER")?;
    let poll = Duration::from_secs(env_number("INDEXER_POLL_SECS")?.unwrap_or(5));
    loop {
        match sync(&pool, &rpc, &config, DEFAULT_CURSOR, start_ledger).await {
            Ok(report) if report.fetched > 0 => {
                println!("synced {} events ({} new)", report.fetched, report.inserted)
            }
            Ok(_) => {}
            // A missing start point is a configuration error; retrying will not fix it.
            Err(e @ Error::NoStartPoint) => return Err(e),
            Err(e) => eprintln!("sync failed, will retry: {e}"),
        }
        tokio::time::sleep(poll).await;
    }
}
