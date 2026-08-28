//! A dog that is large and wet.
//!
//! A discord bot that listens for twitter embed fails and replies to them with fixupx.com links.

use std::fmt::Write;
use std::sync::Arc;
use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;
use serenity::all::*;
use clap::Parser;

mod parse;

/// A dog that is large and wet.
///
/// A discord bot that listens for twitter embed fails and replies to them with fixupx.com links.
#[derive(Debug, Parser)]
pub struct Args {
    
}

/// [`main`].
#[derive(Debug, Error)]
pub enum LwdError {
    /** [`serenity::Error`].               **/ #[error(transparent)] SerenityError  (#[from] serenity::Error              ),
    /** [`serenity::secrets::TokenError`]. **/ #[error(transparent)] TokenErrorError(#[from] serenity::secrets::TokenError),
}

#[tokio::main]
async fn main() -> Result<(), LwdError> {
    let _ = Args::parse();

    ClientBuilder::new(Token::from_env("LWD_TOKEN")?, GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT)
        .event_handler(Arc::new(LargeWetDog))
        .activity(serenity::gateway::ActivityData::custom("Largeing and wetting"))
        .await?.start().await?;

    Ok(())
}

/// A dog that is large and wet.
#[derive(Debug)]
pub struct LargeWetDog;

/// The [`Regex`] to check if we should care about a message.
static MSG_FILTER: LazyLock<Regex> = LazyLock::new(|| Regex::new( r"https?://(?:www\.)?(?:x|twitter)\.com/[^/]+/status/\d+"  ).expect("???"));

/// The [`Regex`] to check if we should care about a URL.
static ID_GETTER : LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^https?://(?:www\.)?(?:x|twitter)\.com/([^/]+/status/\d+)").expect("???"));

/// The description for present-but-failed embeds.
const FAIL_DESC: Option<&str> = Some("Age\\-restricted adult content\\. This content might not be appropriate for people under 18 years old\\. To view this media, you’ll need to log in to X\\. Learn more");

#[serenity::async_trait]
impl EventHandler for LargeWetDog {
    async fn dispatch(&self, context: &Context, event: &FullEvent) {
        if let FullEvent::Message {new_message: msg, ..} = event && MSG_FILTER.captures(&msg.content).is_some() {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;

            let msg = context.http.get_message(msg.channel_id, msg.id).await.expect("The message to still exist");

            let mut paths = parse::parse(&msg.content).filter_map(|x| Some(ID_GETTER.captures(x)?.get(1)?.as_str())).collect::<Vec<_>>();

            let not_fail_paths = msg.embeds.iter()
                .filter    (|embed| embed.description.as_deref() != FAIL_DESC)
                .filter    (|embed| embed.image.as_ref().is_some_and(|image| !image.url.contains("/media-preview/")))
                .filter_map(|embed| Some(ID_GETTER.captures(embed.url.as_deref()?)?.get(1)?.as_str()));

            for not_fail_path in not_fail_paths {
                paths.retain(|&path| path != not_fail_path);
            }

            if !paths.is_empty() {
                let mut ret = String::new();

                for path in paths {
                    writeln!(ret, "https://fixupx.com/{path}").expect("???");
                }

                if msg.content.contains("||") {
                    ret = format!("Found \\|\\|; Assuming all embeds are spoilers\n||{ret}||");
                }

                msg.reply(&context.http, ret).await.expect("Sending the reply to work");
            }
        }
    }
}
