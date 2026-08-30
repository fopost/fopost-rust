//! Create a post, and optionally publish it.
//!
//! ```sh
//! export FOPOST_API_KEY=fp_...
//! cargo run --example create_post -- "Hello from the Rust SDK"
//! cargo run --example create_post -- "Going out now" --publish
//! ```
//!
//! Without `--publish` it stops at a draft, so it is safe to run against a real
//! workspace. `FOPOST_BASE_URL` points it at another deployment.

use fopost::models::{CreatePost, PublishOptions};
use fopost::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let publish = args.iter().any(|a| a == "--publish");
    let text = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| "Hello from the FoPost Rust SDK".to_string());

    let mut builder = Client::builder().api_key(
        std::env::var("FOPOST_API_KEY")
            .map_err(|_| "Set FOPOST_API_KEY first — create a key under Settings → API Keys.")?,
    );
    if let Ok(base_url) = std::env::var("FOPOST_BASE_URL") {
        builder = builder.base_url(base_url);
    }
    let client = builder.build()?;

    let workspaces = client.workspaces().list().await?;
    let Some(workspace) = workspaces.first() else {
        return Err("No workspaces on this key.".into());
    };
    println!("Workspace: {} ({})", workspace.name, workspace.id);

    let accounts = client.accounts().list(Some(&workspace.id)).await?;
    if accounts.is_empty() {
        return Err("No connected accounts in this workspace.".into());
    }
    for account in &accounts {
        println!(
            "  · {}: @{}",
            account.platform,
            account.username.as_deref().unwrap_or("?")
        );
    }

    let body = CreatePost::text(&workspace.id, text)
        .accounts(accounts.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
    let post = client.posts().create(&body).await?;
    println!("Created post {} ({})", post.id, post.status);

    if publish {
        match client
            .posts()
            .publish(&post.id, &PublishOptions::new())
            .await
        {
            Ok(outcome) => {
                for delivery in outcome.deliveries() {
                    println!(
                        "  · {}: {}",
                        delivery.account_id.as_deref().unwrap_or("?"),
                        delivery.status.as_ref().map(|s| s.as_str()).unwrap_or("?")
                    );
                }
            }
            Err(Error::Api(err)) if err.is_payment_required() => {
                println!("Publishing needs an active subscription: {}", err.message);
            }
            Err(err) => return Err(err.into()),
        }
    }

    Ok(())
}
