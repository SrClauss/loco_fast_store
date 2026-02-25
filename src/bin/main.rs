use loco_fast_store::app::App;
use loco_fast_store::env;
use loco_rs::cli;
use migration::Migrator;

// load environment variables from `.env` at program start so callers don't need to export
// them manually. `dotenvy` is lightweight and will silently ignore a missing file.
#[tokio::main]
async fn main() -> loco_rs::Result<()> {
    // load .env one time for the entire process
    env::load();

    cli::main::<App, Migrator>().await
}
