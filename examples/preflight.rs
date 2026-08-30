//! Check a post against every account it targets, without publishing it.
//!
//! ```sh
//! export FOPOST_API_KEY=fp_...
//! cargo run --example preflight -- <post-id>
//! ```
//!
//! `issues` are hard blockers. `signals` are advice — they never stop a publish.

use fopost::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let post_id = std::env::args()
        .nth(1)
        .ok_or("Usage: preflight <post-id>")?;
    let client = Client::from_env()?;

    let result = client.posts().preflight(&post_id).await?;
    println!("ready: {}", result.ready);

    for account in &result.accounts {
        let platform = account.platform.as_deref().unwrap_or("?");
        let username = account.username.as_deref().unwrap_or("?");
        println!("\n{platform} @{username} — ready: {}", account.ready);

        if let Some(score) = account.score {
            println!("  score: {score:.0}/100");
        }
        for issue in &account.issues {
            println!("  ✗ {issue}");
        }
        for signal in &account.signals {
            println!("  · [{}] {}", signal.level, signal.message);
        }
    }

    Ok(())
}
