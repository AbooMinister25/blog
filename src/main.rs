use clap::Parser;
use color_eyre::Result;

#[derive(Parser)]
struct Args {}

fn main() -> Result<()> {
    // Install panic and error report handlers.
    color_eyre::install()?;

    let _ = Args::parse();

    Ok(())
}
