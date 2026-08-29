//! A dog that is large and wet.
//!
//! A discord bot that listens for twitter embed fails and replies to them with fixupx.com links.

use std::borrow::Cow;
use std::collections::HashMap;
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
    pub replies: tokio::sync::RwLock<HashMap<MessageId, MessageId>>,
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

                if let Some(ret) = self.gen_reply(&msg) {
                    self.replies.write().await.insert(msg.id, msg.reply(&context.http, ret).await.expect("Sending the reply to work").id);
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

                if let Some(ret) = self.gen_reply(&msg) {
                    if let Some(&reply_id) = self.replies.read().await.get(&msg.id) {
                        context.http.edit_message(msg.channel_id, reply_id, &EditMessage::new().content(ret).allowed_mentions(CreateAllowedMentions::new().replied_user(false)), vec![]).await.expect("The reply message to still exist");
                    } else {
                        self.replies.write().await.insert(msg.id, msg.reply(&context.http, ret).await.expect("Sending the reply to work").id);
                    }
                } else if let Some(reply_id) = self.replies.write().await.remove(&msg.id) {
                    context.http.delete_message(msg.channel_id, reply_id, Some("Replied-to message removed failed embeds")).await.expect("Deleting the reply to work");
                }
            },
            FullEvent::MessageDelete {channel_id, deleted_message_id, ..} => if let Some(reply_id) = self.replies.write().await.remove(deleted_message_id) {
                context.http.delete_message(*channel_id, reply_id, Some("Replied-to message deleted")).await.expect("Deleting the reply to work");
            },
            FullEvent::MessageDeleteBulk {channel_id, multiple_deleted_messages_ids, ..} => {
                let reply_ids = self.replies.write().await.extract_if(|k, _| multiple_deleted_messages_ids.contains(k)).map(|(_, v)| v).collect::<Vec<_>>();

                for reply_id in reply_ids {
                    context.http.delete_message(*channel_id, reply_id, Some("Replied-to message deleted")).await.expect("Deleting the reply to work");
                }
            },
            _ => {}
        }
    }
}
