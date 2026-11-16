//! tumblr.com

use super::*;

/// Embed URL matcher.
static URL: LazyLock<Regex> = LazyLock::new(|| RegexBuilder::new(r"^https?://(?:www\.|([^.]+)\.)?tumblr\.com/((?:[^/]+/)?\d+)" ).case_insensitive(true).build().expect("???"));



/// If the [`Embed`] is [`EmbedClass::GoodTumblr`], its URL's ID.
pub(super) fn good_tumblr_embed(embed: &Embed) -> Option<Cow<'_, str>> {
    if embed.timestamp.is_some() {
        return None;
    }

    embed.url.as_deref().and_then(tumblr_url)
}



/// If the URL is [`UrlClass::Tumblr`], its ID.
pub(super) fn tumblr_url(url: &str) -> Option<Cow<'_, str>> {
    let captures = URL.captures(url)?;

    Some(match captures.get(1) {
        Some(x) => format!("{}/{}", x.as_str(), captures.get(2)?.as_str()).into(),
        None    => captures.get(2)?.as_str().into(),
    })
}
