//! A dog that is large and wet.
//!
//! A discord bot that listens for twitter embed fails and replies to them with fixupx.com links.

use std::borrow::Cow;
use std::collections::{HashMap, hash_map::Entry};
use std::sync::Arc;

use thiserror::Error;
use serenity::all::*;
use clap::Parser;

mod reply;
mod get_urls;

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
        .event_handler(Arc::new(LargeWetDog::default()))
        .activity(serenity::gateway::ActivityData::custom(env!("CARGO_PKG_REPOSITORY")))
        .await?.start().await?;

    Ok(())
}

/// A dog that is large and wet.
#[derive(Debug, Default)]
pub struct LargeWetDog {
    pub replies: tokio::sync::Mutex<HashMap<MessageId, (Option<Timestamp>, MessageId)>>,
}

#[serenity::async_trait]
impl EventHandler for LargeWetDog {
    async fn dispatch(&self, context: &Context, event: &FullEvent) {
        match event {
            FullEvent::Ready {data_about_bot, ..} => {
                println!();
                println!("Connected!");
                println!();
                println!("Insrall to a server: https://discord.com/oauth2/authorize?client_id={0}&scope=bot", data_about_bot.application.id);
            },
            FullEvent::Message {new_message: msg, ..} => {
                let mut msg = Cow::Borrowed(msg);

                for _ in 0..5 {
                    if !msg.embeds.is_empty() {
                        break;
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                    msg = Cow::Owned(context.http.get_message(msg.channel_id, msg.id).await.expect("The message to still exist"));
                }

                if let Some(ret) = self.gen_reply(&msg) && let Entry::Vacant(entry) = self.replies.lock().await.entry(msg.id) {
                    entry.insert((msg.edited_timestamp, msg.reply(&context.http, ret).await.expect("Sending the reply to work").id));
                }
            },
            FullEvent::MessageUpdate {event: MessageUpdateEvent {message: msg, ..}, ..} => {
                let mut msg = Cow::Borrowed(msg);

                for _ in 0..5 {
                    if !msg.embeds.is_empty() {
                        break;
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                    msg = Cow::Owned(context.http.get_message(msg.channel_id, msg.id).await.expect("The message to still exist"));
                }

                match self.replies.lock().await.entry(msg.id) {
                    Entry::Occupied(mut entry) => if entry.get().0 < msg.edited_timestamp {
                        match self.gen_reply(&msg) {
                            Some(ret) => {entry.insert((msg.edited_timestamp, context.http.edit_message(msg.channel_id, entry.get().1, &EditMessage::new().content(ret).allowed_mentions(CreateAllowedMentions::new().replied_user(false)), vec![]).await.expect("The reply message to still exist").id));},
                            None      => context.http.delete_message(msg.channel_id, entry.remove().1, Some("Replied-to message removed failed embeds")).await.expect("Deleting the reply to work"),
                        }
                    },
                    Entry::Vacant(entry) => if let Some(ret) = self.gen_reply(&msg) {
                        entry.insert((msg.edited_timestamp, msg.reply(&context.http, ret).await.expect("Sending the reply to work").id));
                    },
                }
            },
            FullEvent::MessageDelete {channel_id, deleted_message_id, ..} => if let Some((_, reply_id)) = self.replies.lock().await.remove(deleted_message_id) {
                context.http.delete_message(*channel_id, reply_id, Some("Replied-to message deleted")).await.expect("Deleting the reply to work");
            },
            FullEvent::MessageDeleteBulk {channel_id, multiple_deleted_messages_ids, ..} => {
                let replies = self.replies.lock().await.extract_if(|k, _| multiple_deleted_messages_ids.contains(k)).map(|(_, v)| v).collect::<Vec<_>>();

                for (_, reply_id) in replies {
                    context.http.delete_message(*channel_id, reply_id, Some("Replied-to message deleted")).await.expect("Deleting the reply to work");
                }
            },
            _ => {}
        }
    }
}
