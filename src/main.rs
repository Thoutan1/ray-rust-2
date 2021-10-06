mod commands;
mod settings;
mod listeners;

use commands::{fun::*, moderation::*, owner::*, misc::*};
use listeners::{handlers::Handler};

use log::{error};
use pretty_env_logger::formatted_builder;
use reqwest::Client as ReqwestClient;
use serenity::{
    client::bridge::gateway::ShardManager,
    framework::standard::{
        help_commands,
        macros::{group, help, hook},
        Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    http::Http,
    model::{
        channel::Message,
        id::UserId,
    },
    prelude::{Client, Context, TypeMapKey},
};
use std::{collections::HashSet, error::Error, sync::Arc};
use tokio::sync::Mutex;

use crate::settings::settings;

struct ShardManagerContainer;
struct ReqwestContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for ReqwestContainer {
    type Value = ReqwestClient;
}

#[group]
#[commands(ping, version)]
struct Misc;

#[group]
#[commands(cat, eightball, dog)]
struct Fun;

#[group]
#[commands(purge)]
struct Moderation;

#[group]
#[commands(quit)]
struct Owner;

#[help]
#[max_levenshtein_distance(2)]
async fn bot_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let ho = help_options.clone();
    let _ = help_commands::with_embeds(ctx, msg, args, &ho, groups, owners).await;
    Ok(())
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, name: &str, result: CommandResult) {
    match result {
        Ok(()) => {}
        Err(why) => error!("Command '{}' returned an error {:?}", name, why),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Initialize settings
    settings::init();

    // Setup logger
    let mut logger = formatted_builder();
    for (path, level) in &settings().logging.filters {
        logger.filter(Some(path), *level);
    }
    logger.init();

    let http = Http::new_with_token(&settings().bot.token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix(&settings().bot.prefix)
                .on_mention(Some(bot_id))
                .owners(owners)
                .allow_dm(false)
                .case_insensitivity(true)
        })
        .after(after)
        .group(&MISC_GROUP)
        .group(&OWNER_GROUP)
        .group(&FUN_GROUP)
        .group(&MODERATION_GROUP)
        .help(&BOT_HELP);

    let mut client = Client::builder(&settings().bot.token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Something went wrong while building the client.");

    {
        let mut data = client.data.write().await;
        let reqwest_client = ReqwestClient::builder().build()?;

        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<ReqwestContainer>(reqwest_client);
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Something went wrong while registering Ctrl+C handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start_autosharded().await {
        error!("Something went wrong while starting the client: {:?}", why);
    }

    Ok(())
}
