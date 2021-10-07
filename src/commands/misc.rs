use serenity::{
    client::bridge::gateway::ShardId,
    framework::standard::{macros::command, CommandResult},
    model::prelude::Message,
    prelude::Context,
};

use std::fs;

use std::env::current_exe;

use crate::settings::{
    DEFAULT_COLOR,
};

use rand::Rng; 

use chrono::offset::Utc;

use chrono::DateTime;

use chrono::Duration;

use crate::ShardManagerContainer;


#[command]
#[description = "Check if the bot is working"]
async fn ping(context: &Context, message: &Message) -> CommandResult {
    let start = Utc::now();
    let start_ts = start.timestamp();
    let start_ts_ss = start.timestamp_subsec_millis() as i64;
    let mut ping: Message = message.channel_id.send_message(context, |m| m.content(":ping_pong: Pinging!")).await?;
    let end = Utc::now();
    let end_ts = end.timestamp();
    let end_ts_ss = end.timestamp_subsec_millis() as i64;
    let api_response = ((end_ts - start_ts) * 1000) + (end_ts_ss - start_ts_ss);
    let ctx_data = context.data.read().await;
    let shard_manager = match ctx_data.get::<ShardManagerContainer>() {
        Some(shard) => shard,
        None => {
            message.reply(context, "I encountered a problem while getting the shard manager.").await?;
            return Ok(());
        }
    };  
    let num = rand::thread_rng().gen_range(0..100);
    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;
    let runner = match runners.get(&ShardId(context.shard_id)) {
        Some(runner) => runner,
        None => {
            message.reply(context, "Could not find a shard").await?;
            return Ok(());
        }
    };

    let shard_response = match runner.latency {
        Some(latency) => match Duration::from_std(latency) {
            Ok(time) => format!("`{}ms`", time.num_milliseconds()),
            Err(_) => "No latency information available".to_string()
        },
        None => "No data available at the moment.".to_string()
    };

    let response = format!(
        "Pong! Succesfully retrieved the message and shard latencies. :ping_pong:\n\n\
        **API Response Time**: `{}ms`\n\
        **Your Ping IS**: `{}ms`\n\
        **Shard Response Time**: {}",
        api_response, num, shard_response
    );

    ping.edit(context, |message| {
        message.content("");
        message.embed(|embed| {
            embed.color(DEFAULT_COLOR);
            embed.title("Discord Latency Information");
            embed.description(response)
        })
    })
    .await?;

    Ok(())

}


#[command]
#[description = "Check Version of the Bot"]
async fn version(ctx: &Context, msg: &Message) -> CommandResult {
    let exe = current_exe().unwrap();
    let metas = fs::metadata(exe).unwrap();
    let build_date: DateTime<Utc> = metas.created().unwrap().into();
    let build_tz = build_date + chrono::Duration::hours(2);

    msg.reply(
        ctx, 
        format!("\nRay-ts version {}\nBuilt on {}", 
            env!("CARGO_PKG_VERSION"),
            build_tz
        )
    ).await?;
    
    Ok(())
}

