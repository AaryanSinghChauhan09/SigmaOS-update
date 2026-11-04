use sigmaos_update::{Config, UpdateComponent, Component};
fn main() {
    env_logger::init();
    let mut c = UpdateComponent::new(Config::default());
    if let Err(e) = c.initialize() { eprintln!("Failed: {}", e); std::process::exit(1); }
    println!("SigmaOS update running");
    let _ = c.shutdown();
}
