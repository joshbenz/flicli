use color_eyre::Result as EyreResult;
use flicli::ssh_client::Client;
use tracing::{info, instrument};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> EyreResult<()> {
    let username = "";
    let password = "!";

    let mut client = Client::connect(username, password, "127.0.0.1:22")
        .await
        .unwrap();

    let out = client.send_command(b"ls -l\n".to_vec()).await.unwrap();
    println!("{}", out);
    Ok(())
}

pub fn setup_logging() -> EyreResult<()> {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1");
    }
    color_eyre::install()?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "trace");
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Ok(())
}
