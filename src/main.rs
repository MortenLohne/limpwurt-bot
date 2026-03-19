mod chunkroll_predictor;
mod config;
mod db;
mod hiscore_lookup;
mod update_post;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::Utc;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::GatewayIntents;
use serenity::model::id::ChannelId;
use serenity::prelude::*;

use crate::config::{Config, PlayerConfig};

const DB_PATH: &str = "hiscores.db";
const POLL_INTERVAL: Duration = Duration::from_secs(15 * 60);

struct Handler {
    db_conn: Arc<Mutex<rusqlite::Connection>>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if let Err(e) = self.handle_message(ctx, msg).await {
            eprintln!("Error handling message: {:#}", e);
        }
    }
}

impl Handler {
    async fn handle_message(&self, ctx: Context, msg: Message) -> eyre::Result<()> {
        if msg.content.starts_with("!when") {
            let prediction = chunkroll_predictor::predict_chunkroll_date(self.db_conn.clone())?;

            let message = if prediction.rc_exp_left == 0 {
                "Limpwurt is already 99 RC! The world is his oyster!".to_string()
            } else if prediction.days_left < 2.0 {
                "Limpwurt is close to 99 RC! Chunkroll is imminent!".to_string()
            } else {
                let chunkroll_date = chrono::Utc::now().date_naive()
                    + chrono::Days::new((prediction.days_left.ceil() as u64).saturating_sub(0));
                let clog_slot_string = if prediction.clog_slots_left == 0 {
                    "".to_string()
                } else {
                    ", and still needs the earth warrior champion's scroll".to_string()
                };

                format!(
                    "Limpwurt needs another {}k RC exp. He has {} desiccated pages and {}k pure essence banked{}. Chunkroll is estimated on **{}.**",
                    prediction.rc_exp_left.div_ceil(1000),
                    prediction.current_pages,
                    prediction.current_pure_essence.div_ceil(1000),
                    clog_slot_string,
                    chunkroll_date.format("%d %B %Y"),
                )
            };

            msg.reply(&ctx.http, message).await?;
        }
        Ok(())
    }
}

async fn poll_once(
    player: &str,
    conn: Arc<Mutex<rusqlite::Connection>>,
    http: Arc<serenity::http::Http>,
    channels: &HashMap<ChannelId, Vec<PlayerConfig>>,
) -> eyre::Result<()> {
    // Get previous metrics
    let player_owned = player.to_string();
    let conn_clone = Arc::clone(&conn);
    let prev_metrics: HashMap<String, hiscore_lookup::Metric> =
        tokio::task::spawn_blocking(move || {
            let guard = conn_clone.lock().unwrap();
            db::last_scores(&guard, &player_owned)
        })
        .await??
        .into_iter()
        .map(|m| (m.name.clone(), m))
        .collect();

    // Fetch current metrics
    let metrics = hiscore_lookup::fetch_metrics(player).await?;

    // Post update messages in all channels subscribed to the player
    for (channel_id, players) in channels {
        // The tracked player may or may not be subscribed to by this channel
        let Some(player_config) = players
            .iter()
            .find(|player_config| player_config.player_name == player)
        else {
            continue;
        };
        for metric in &metrics {
            let Some(prev_metric) = prev_metrics.get(&metric.name) else {
                continue;
            };

            if !(metric.score > prev_metric.score
                || metric
                    .exp
                    .is_some_and(|exp| prev_metric.exp.is_some_and(|prev_exp| exp > prev_exp)))
            {
                continue;
            }
            let Some(message) =
                update_post::get_update_message(metric, prev_metric, player_config)?
            else {
                continue;
            };
            if let Err(e) = channel_id.say(&http, &message).await {
                eprintln!("Discord send error: {:#}", e);
            }
        }
    }

    // Insert snapshot
    let fetched_at = Utc::now().to_rfc3339();
    let player_cloned = player.to_string();
    let conn_clone = Arc::clone(&conn);
    tokio::task::spawn_blocking(move || {
        let guard = conn_clone.lock().unwrap();
        db::insert_snapshot(&guard, &player_cloned, &fetched_at, &metrics)
    })
    .await??;

    Ok(())
}

async fn player_loop(
    player: String,
    conn: Arc<Mutex<rusqlite::Connection>>,
    http: Arc<serenity::http::Http>,
    channels: Arc<HashMap<ChannelId, Vec<PlayerConfig>>>,
) {
    loop {
        if let Err(e) = poll_once(&player, Arc::clone(&conn), Arc::clone(&http), &channels).await {
            eprintln!("[{}] Error polling: {:#}", player, e);
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().ok();

    let config: Config = serde_json::from_slice(&tokio::fs::read("config.json").await?)?;
    config.validate()?;

    let token =
        std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN environment variable must be set");

    let conn = db::open(DB_PATH)?;
    let conn = Arc::new(Mutex::new(conn));

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            db_conn: conn.clone(),
        })
        .await?;

    let http = Arc::clone(&client.http);
    let channels = Arc::new(config.channels);

    for player in config.players_to_track {
        let conn_clone = Arc::clone(&conn);
        let http_clone = Arc::clone(&http);
        let channels_cloned = Arc::clone(&channels);
        tokio::spawn(player_loop(player, conn_clone, http_clone, channels_cloned));
    }

    client.start().await?;

    Ok(())
}
