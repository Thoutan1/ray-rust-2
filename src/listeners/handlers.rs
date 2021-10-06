use serenity::{
    async_trait,
    model::{
        event::ResumedEvent,
        gateway::Ready,
        prelude::{Activity, GuildId},
    },
    prelude::{Context, EventHandler},
};
use log::{info};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, _: Context, guilds: Vec<GuildId>) {
        info!("Connected to {} guilds.", guilds.len());
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        if let Some(shard) = ready.shard {
            info!(
                "Connected as [Name: {}] [ID: {}] [Shard: {}/{}]",
                ready.user.name,
                ready.user.id,
                shard[0] + 1,
                shard[1]
            );
        } else {
            info!(
                "Connected as [Name: {}] [ID: {}]",
                ready.user.name, ready.user.id
            );
        }

        ctx.set_activity(Activity::playing(&format!(
            "CODM"
        )))
        .await;
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Reconnected")
    }
}