//! vxtwitter.com

use super::*;

/// Embed URL matcher.
static URL: LazyLock<Regex> = LazyLock::new(|| RegexBuilder::new(r"^https?://(?:www\.)?vxtwitter\.com/([^/]+/status/\d+)(/p\d)?").case_insensitive(true).build().expect("???"));



/// If the [`Embed`] is [`EmbedClass::VxTwitter1Of1`], its URL's ID.
pub(super) fn vx_twitter_1_of_1_embed(embed: &Embed) -> Option<Cow<'_, str>> {
    if vx_twitter_1_of_2_embed(embed).is_some() {
        return None;
    }

    if vx_twitter_2_of_2_embed(embed).is_some() {
        return None;
    }

    embed.url.as_deref().and_then(vx_twitter_p1_url)
}

/// If the [`Embed`] is [`EmbedClass::VxTwitter1Of2`], its URL's ID.
pub(super) fn vx_twitter_1_of_2_embed(embed: &Embed) -> Option<Cow<'_, str>> {
    for component in &embed.components {
        if let Component::Container(container) = component {
            for component in &container.components {
                if let ContainerComponent::TextDisplay(text) = component && text.content.contains("Page 1/2") {
                    return embed.url.as_deref().and_then(vx_twitter_p1_url);
                }
            }
        }
    }

    None
}

/// If the [`Embed`] is [`EmbedClass::VxTwitter2Of2`], its URL's ID.
pub(super) fn vx_twitter_2_of_2_embed(embed: &Embed) -> Option<Cow<'_, str>> {
    for component in &embed.components {
        if let Component::Container(container) = component {
            for component in &container.components {
                if let ContainerComponent::TextDisplay(text) = component && text.content.contains("Page 2/2") {
                    return embed.url.as_deref().and_then(vx_twitter_p2_url);
                }
            }
        }
    }

    None
}



/// If the URL is [`UrlClass::VxTwitterP1`], its ID.
pub(super) fn vx_twitter_p1_url(url: &str) -> Option<Cow<'_, str>> {
    let captures = URL.captures(url)?;

    if captures.get(2).is_some_and(|x| x.as_str() == "/p2") {
        return None;
    }

    Some(captures.get(1)?.as_str().into())
}

/// If the URL is [`UrlClass::VxTwitterP2`], its ID.
pub(super) fn vx_twitter_p2_url(url: &str) -> Option<Cow<'_, str>> {
    let captures = URL.captures(url)?;

    if captures.get(2).is_none_or(|x| x.as_str() == "/p1") {
        return None;
    }

    Some(captures.get(1)?.as_str().into())
}
