#[tokio::main]
async fn main() {
    if let Err(err) = rpc_api::server::run_from_env().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
