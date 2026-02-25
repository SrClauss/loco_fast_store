use once_cell::sync::OnceCell;

// simple helper to load `.env` exactly once. call `crate::env::load()` whenever
// you access environment variables so config from `.env` is available without
// having to export manually.
static ENV_LOADED: OnceCell<()> = OnceCell::new();

pub fn load() {
    ENV_LOADED.get_or_init(|| {
        let _ = dotenvy::dotenv();
    });
}
