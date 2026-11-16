//! twitter.com

use super::*;

/** Embed URL matcher.        */ static URL         : LazyLock<Regex> = LazyLock::new(|| RegexBuilder::new(r"^https?://(?:www\.)?(?:x|twitter)\.com/([^/]+/status/\d+)"                                                                                                            ).case_insensitive(true).build().expect("???"));
/** Fail description matcher. */ static FAIL_DESC   : LazyLock<Regex> = LazyLock::new(|| RegexBuilder::new(r"Age\\-restricted adult content\\. This content might not be appropriate for people under 18 years old\\. To view this media, you’ll need to log in to X\\. Learn more").case_insensitive(true).build().expect("???"));
/** Fail image URL matcher.   */ static FAIL_IMG_URL: LazyLock<Regex> = LazyLock::new(|| RegexBuilder::new(r"/(media-preview|tweet_video_thumb)/"                                                                                                                                  ).case_insensitive(true).build().expect("???"));



/// If the [`Embed`] is [`EmbedClass::GoodTwitter`], its URL's ID.
pub(super) fn good_twitter_embed(embed: &Embed) -> Option<Cow<'_, str>> {
    if embed.description.as_deref().is_some_and(|description| FAIL_DESC.is_match(description)) {
        return None;
    }

    if embed.image.as_ref().is_some_and(|image| FAIL_IMG_URL.is_match(&image.url)) {
        return None;
    }

    embed.url.as_deref().and_then(twitter_url)
}



/// If the URL is [`UrlClass::Twitter`], its ID.
pub(super) fn twitter_url(url: &str) -> Option<Cow<'_, str>> {
    Some(URL.captures(url)?.get(1)?.as_str().into())
}
