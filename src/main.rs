use anyhow::Result;
use clap::Parser;
use update::update;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    auto: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = update::Config { auto_start: cli.auto, ..Default::default() };
    let mut component = update::new(config);
    
    component.initialize()?;
    component.run()?;
    
    println!("SigmaOS-update operational");
    Ok(())
}
