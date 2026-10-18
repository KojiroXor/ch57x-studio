//! CH57x Studio main binary entrypoint

mod coordinator;
mod tray;

fn main() {
    tracing_subscriber::fmt::init();
    println!("CH57x Studio initialized.");
}
