//! [`LargeWetDog::gen_reply`].

use std::borrow::Cow;
use std::fmt::Write;
use std::sync::LazyLock;

use regex::{Regex, RegexBuilder};
use serenity::all::*;

use super::LargeWetDog;

/// The [`Regex`] to check if we should care about a twitter URL.
static TWITTER: LazyLock<Regex> = LazyLock::new(|| RegexBuilder::new(r"^https?://(?:www\.)?(?:x|twitter)\.com/([^/]+/status/\d+)" ).case_insensitive(true).build().expect("???"));
/// The [`Regex`] to check if we should care about a tunblr URL.
static TUMBLR : LazyLock<Regex> = LazyLock::new(|| RegexBuilder::new(r"^https?://(?:www\.|([^.]+)\.)?tumblr\.com/((?:[^/]+/)?\d+)").case_insensitive(true).build().expect("???"));

/// The description for present-but-failed twitter embeds.
const TWITTER_FAIL_DESC: Option<&str> = Some("Age\\-restricted adult content\\. This content might not be appropriate for people under 18 years old\\. To view this media, you’ll need to log in to X\\. Learn more");

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

impl LargeWetDog {
    pub fn gen_reply(&self, msg: &Message) -> Option<String> {
        let mut matches = super::get_urls::get_urls(&msg.content).filter_map(get).collect::<Vec<_>>();

        let not_fail_twitters = msg.embeds.iter()
            .filter    (|embed| embed.description.as_deref() != TWITTER_FAIL_DESC && embed.image.as_ref().is_none_or(|image| !image.url.contains("/media-preview/") && !image.url.contains("/tweet_video_thumb/")))
            .filter_map(|embed| twitter(embed.url.as_deref()?));

        let not_fail_tumblrs = msg.embeds.iter()
            .filter(|embed| embed.timestamp.is_some())
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

            Some(ret)
        } else {
            None
        }
    }
}
