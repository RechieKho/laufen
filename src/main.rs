pub mod adapter;
pub mod app;

fn main() -> anyhow::Result<()> {
    app::App::run()
}
