pub mod adapter;
pub mod app;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    app::App::run()
}
