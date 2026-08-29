//! A dog that is large and wet.
//!
//! A discord bot that listens for twitter embed fails and replies to them with fixupx.com links.

use std::borrow::Cow;
use std::fmt::Write;
use std::sync::Arc;
use std::sync::LazyLock;

use regex::Regex;
use thiserror::Error;
use serenity::all::*;
use clap::Parser;

mod parse;

/// A discord bot that fixes some embed fails.
///
/// https://github.com/Scripter17/large-wet-dog
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

    println!(env!("CARGO_PKG_REPOSITORY"));

    ClientBuilder::new(Token::from_env("LWD_TOKEN")?, GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT)
        .event_handler(Arc::new(LargeWetDog))
        .activity(serenity::gateway::ActivityData::custom(env!("CARGO_PKG_REPOSITORY")))
        .await?.start().await?;

    Ok(())
}

/// A dog that is large and wet.
#[derive(Debug)]
pub struct LargeWetDog;

/// The [`Regex`] to check if we should care about a twitter URL.
static TWITTER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^https?://(?:www\.)?(?:x|twitter)\.com/([^/]+/status/\d+)" ).expect("???"));
/// The [`Regex`] to check if we should care about a tunblr URL.
static TUMBLR : LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^https?://(?:www\.|([^.]+)\.)?tumblr\.com/((?:[^/]+/)?\d+)").expect("???"));

/// The description for present-but-failed twitter embeds.
const TWITTER_FAIL: Option<&str> = Some("Age\\-restricted adult content\\. This content might not be appropriate for people under 18 years old\\. To view this media, you’ll need to log in to X\\. Learn more");
/// The description for present-but-failed tumblr embeds.
const TUMBLR_FAIL: Option<&str> = Some("Tumblr is a place to express yourself, discover yourself, and bond over the stuff you love. It's where your interests connect you with your people.");

/// Extract a twitter URL's key.
fn twitter(url: &str) -> Option<Cow<'_, str>> {
    Some(TWITTER.captures(url)?.get(1)?.as_str().into())
}

/// Extract a tumblr URL's key.
fn tumblr(url: &str) -> Option<Cow<'_, str>> {
    let captures = TUMBLR.captures(url)?;

    Some(match captures.get(1) {
        Some(x) => format!("{}/{}", x.as_str(), captures.get(2)?.as_str()).into(),
        None    => captures.get(2)?.as_str().into(),
    })
}

/// The category of a URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Category {
    /// Twitter.
    Twitter,
    /// Tumblr.
    Tumblr,
}

/// Get a URL's [`Category`] and key.
fn get(url: &str) -> Option<(Category, Cow<'_, str>)> {
    #[expect(clippy::manual_map, reason = "Less ugly this way.")]
    if let Some(x) = twitter(url) {
        Some((Category::Twitter, x))
    } else if let Some(x) = tumblr(url) {
        Some((Category::Tumblr, x))
    } else {
        None
    }
}

#[serenity::async_trait]
impl EventHandler for LargeWetDog {
    async fn dispatch(&self, context: &Context, event: &FullEvent) {
        match event {
            FullEvent::Ready {data_about_bot, ..} => {
                println!();
                println!("Connected!");
                println!();
                println!("Install to your account: https://discord.com/oauth2/authorize?client_id={0}"          , data_about_bot.application.id);
                println!("Insrall to a server    : https://discord.com/oauth2/authorize?client_id={0}&scope=bot", data_about_bot.application.id);
            },
            FullEvent::Message {new_message: msg, ..} => {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                let msg = context.http.get_message(msg.channel_id, msg.id).await.expect("The message to still exist");

                let mut matches = parse::parse(&msg.content).filter_map(get).collect::<Vec<_>>();

                let not_fail_twitters = msg.embeds.iter()
                    .filter    (|embed| embed.description.as_deref() != TWITTER_FAIL)
                    .filter    (|embed| embed.image.as_ref().is_some_and(|image| !image.url.contains("/media-preview/") && !image.url.contains("/tweet_video_thumb/")))
                    .filter_map(|embed| twitter(embed.url.as_deref()?));

                let not_fail_tumblrs = msg.embeds.iter()
                    .filter(|embed| embed.description.as_deref() != TUMBLR_FAIL)
                    .filter_map(|embed| tumblr(embed.url.as_deref()?));

                for not_fail_twitter in not_fail_twitters {matches.retain(|x| !(x.0 == Category::Twitter && x.1 == not_fail_twitter));}
                for not_fail_tumblr  in not_fail_tumblrs  {matches.retain(|x| !(x.0 == Category::Tumblr  && x.1 == not_fail_tumblr ));}

                if !matches.is_empty() {
                    let mut ret = String::new();

                    for x in matches {
                        match x {
                            (Category::Twitter, x) => writeln!(ret, "https://fixupx.com/{x}"  ).expect("???"),
                            (Category::Tumblr , x) => writeln!(ret, "https://txtumblr.com/{x}").expect("???"),
                        }
                    }

                    if msg.content.contains("||") {
                        ret = format!("Found \\|\\|; Assuming all embeds are spoilers\n||{ret}||");
                    }

                    msg.reply(&context.http, ret).await.expect("Sending the reply to work");
                }
            },
            _ => {}
        }
    }
}
