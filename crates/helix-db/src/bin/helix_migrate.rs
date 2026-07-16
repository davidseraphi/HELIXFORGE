//! Stand-alone migration binary.
//!
//! Connects to `DATABASE_URL`, runs `helix_db` embedded migrations, and exits.
//! Intended for Kubernetes pre-install/pre-upgrade Jobs where mounting raw SQL
//! files is fragile.

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL environment variable is required");

    let _pool = helix_db::connect_and_migrate(&database_url).await?;

    println!("helix_db migrations applied successfully");
    Ok(())
}
