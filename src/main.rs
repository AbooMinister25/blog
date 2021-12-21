use anyhow::{Context, Result};

#[rocket::main]
async fn main() -> Result<()> {
    blog::app()
        .await
        .launch()
        .await
        .context("Error while launching application")?;
    Ok(())
}
