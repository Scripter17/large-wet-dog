//! [`LargeWetDog::gen_reply`].

use std::sync::LazyLock;
use std::borrow::Cow;

use regex::{Regex, RegexBuilder};
use serenity::all::*;

use super::LargeWetDog;

mod twitter   ; use twitter   ::*;
mod tumblr    ; use tumblr    ::*;
mod vx_twitter; use vx_twitter::*;

/// A classified url.
#[derive(Debug)]
struct ClassifiedUrl<'a> {
    /** The [`UrlClass`]. */ class: UrlClass,
    /** The URL's ID.     */ id   : Cow<'a, str>,
}

/// A classified embed.
#[derive(Debug)]
struct ClassifiedEmbed<'a> {
    /** The [`EmbedClass`].   */ class: EmbedClass,
    /** The embed's URL's ID. */ id   : Cow<'a, str>,
}

/// The class of a URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UrlClass {
    /** Twitter.          */ Twitter    ,
    /** Tumblr.           */ Tumblr     ,
    /** Vxtwitter page 1. */ VxTwitterP1,
    /** Vxtwitter page 2. */ VxTwitterP2,
}

/// The class of an embed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmbedClass {
    /** A good twitter embed.          */ GoodTwitter  ,
    /** A good tumblr embed.           */ GoodTumblr   ,
    /** A page 1 of 1 vxtwitter embed. */ VxTwitter1Of1,
    /** A page 1 of 2 vxtwitter embed. */ VxTwitter1Of2,
    /** A page 2 of 2 vxtwitter embed. */ VxTwitter2Of2,
}

fn classify_url(url: &str) -> Option<ClassifiedUrl<'_>> {
    match url {
        url if let Some(id) = twitter_url      (url) => Some(ClassifiedUrl {class: UrlClass::Twitter    , id}),
        url if let Some(id) = tumblr_url       (url) => Some(ClassifiedUrl {class: UrlClass::Tumblr     , id}),
        url if let Some(id) = vx_twitter_p1_url(url) => Some(ClassifiedUrl {class: UrlClass::VxTwitterP1, id}),
        url if let Some(id) = vx_twitter_p2_url(url) => Some(ClassifiedUrl {class: UrlClass::VxTwitterP2, id}),
        _ => None
    }
}

fn classify_embed(embed: &Embed) -> Option<ClassifiedEmbed<'_>> {
    match embed {
        embed if let Some(id) = good_twitter_embed     (embed) => Some(ClassifiedEmbed {class: EmbedClass::GoodTwitter  , id}),
        embed if let Some(id) = good_tumblr_embed      (embed) => Some(ClassifiedEmbed {class: EmbedClass::GoodTumblr   , id}),
        embed if let Some(id) = vx_twitter_1_of_1_embed(embed) => Some(ClassifiedEmbed {class: EmbedClass::VxTwitter1Of1, id}),
        embed if let Some(id) = vx_twitter_1_of_2_embed(embed) => Some(ClassifiedEmbed {class: EmbedClass::VxTwitter1Of2, id}),
        embed if let Some(id) = vx_twitter_2_of_2_embed(embed) => Some(ClassifiedEmbed {class: EmbedClass::VxTwitter2Of2, id}),
        _ => None
    }
}

impl LargeWetDog {
    pub fn gen_reply(&self, msg: &Message) -> Option<String> {
        let mut urls = super::get_urls::get_urls(&msg.content).filter_map(classify_url).collect::<Vec<_>>();

        let embeds = msg.embeds.iter().filter_map(classify_embed).collect::<Vec<_>>();

        for embed in &embeds {
            if embed.class == EmbedClass::VxTwitter1Of2 {
                for url in urls.iter_mut() {
                    if url.class == UrlClass::VxTwitterP1 && url.id == embed.id {
                        url.class = UrlClass::VxTwitterP2;
                    }
                }
            }
        }

        for embed in embeds {
            match embed.class {
                EmbedClass::GoodTwitter   => urls.retain(|url| !(matches!(url.class, UrlClass::Twitter                            ) && url.id == embed.id)),
                EmbedClass::GoodTumblr    => urls.retain(|url| !(matches!(url.class, UrlClass::Tumblr                             ) && url.id == embed.id)),
                EmbedClass::VxTwitter1Of1 => urls.retain(|url| !(matches!(url.class, UrlClass::VxTwitterP1                        ) && url.id == embed.id)),
                EmbedClass::VxTwitter1Of2 => urls.retain(|url| !(matches!(url.class, UrlClass::VxTwitterP1                        ) && url.id == embed.id)),
                EmbedClass::VxTwitter2Of2 => urls.retain(|url| !(matches!(url.class, UrlClass::VxTwitterP1 | UrlClass::VxTwitterP2) && url.id == embed.id)),
            }
        }

        let mut rets = Vec::new();

        for url in urls {
            match url {
                ClassifiedUrl {class: UrlClass::Twitter    , id   } => rets.push(format!("https://vxtwitter.com/{id}"   )),
                ClassifiedUrl {class: UrlClass::Tumblr     , id   } => rets.push(format!("https://txtumblr.com/{id}"    )),
                ClassifiedUrl {class: UrlClass::VxTwitterP1, id: _} => {},
                ClassifiedUrl {class: UrlClass::VxTwitterP2, id   } => rets.push(format!("https://vxtwitter.com/{id}/p2")),
            }
        }

        let mut i = 0;

        while i < rets.len() {
            let mut j = 0;

            while j < i {
                if rets[i] == rets[j] {
                    rets.remove(i);
                    i -= 1;
                } else {
                    j += 1;
                }
            }

            i += 1;
        }

        if rets.is_empty() {
            return None;
        }

        let mut ret = rets.join("\n");

        if msg.content.contains("||") {
            ret = format!("Found \\|\\|; Assuming all embeds are spoilers\n||{ret}||");
        }

        Some(ret)
    }
}
