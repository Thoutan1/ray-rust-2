use serenity::framework::standard::{macros::command, CommandResult, Args};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::Result;
use std::time::Instant;
use serenity::futures::StreamExt;
use tokio::time::Duration;

async fn delete_messages(ctx: &Context, channel: &ChannelId, count: i32) -> Result<()> {
    let mut message_ids: Vec<MessageId> = Vec::new();

    // this might not be the most efficient way to do it lmao
    {
        let mut messages = channel.messages_iter(&ctx).boxed();

        for _ in 0..count {
            let message = messages.next().await;
            message_ids.push(message.unwrap()?.id);
        }
    }

    channel.delete_messages(ctx, message_ids).await?;

    Ok(())
}


#[command]
#[description = "\
Delete messages in bulk.
This can only be used for members that has the \"Manage Messages\" permission.
Note: The purge message is not going to count as a message to delete."]
#[required_permissions(MANAGE_MESSAGES)]
#[only_in("guilds")]
#[usage = "(the amount of messages to be deleted)"]
#[example = "10"]
#[aliases("p")]
#[num_args(1)]
async fn purge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let count_res = args.single::<i32>();

    if count_res.is_err() {
        msg.reply(
            ctx,
            format!("Failed parsing the 1st argument as an integer: {}", count_res.unwrap_err())
        ).await?;
    } else {
        let count = count_res.unwrap();

        if count > 1000 {
            msg.reply(ctx, "Woah, that's too big my dude").await?;

            return Ok(())
        }

        let now = Instant::now();

        delete_messages(ctx, &msg.channel_id, count + 1).await?;
                                                 /* +1 for the command message */

        let message = msg.channel_id.send_message(ctx, |c| {
            c.content(format!("{} message(s) deleted in {}ms", count, now.elapsed().as_millis()))
        }).await?;

        // delay for 5 secs and delete the message
        tokio::time::sleep(Duration::from_secs(5)).await;

        message.delete(ctx).await?;
    }

    Ok(())
}