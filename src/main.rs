mod chunkroll_predictor;
mod config;
mod db;
pub mod drop_simulator;
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

use crate::config::{Config, PlayerConfig, PlayerToTrack};
use crate::update_post::metric_updates;

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
        if msg.channel_id == 871879186732707853
            && msg
                .content
                .split_whitespace()
                .next()
                .is_some_and(|word| word == "!when")
        {
            let prediction = {
                let db_conn_guard = self.db_conn.lock().unwrap();
                let metrics = db::last_scores(&db_conn_guard, "OneChunkUp")?;
                chunkroll_predictor::predict_chunkroll_date(&metrics)?
            };

            msg.reply(
                ctx,
                format!(
                    "Limpwurt still needs {} pieces of Dagon'hai robes, and has killed {} chaos dwarves so far. Chunkroll is estimated on **{}**, and between **{}** and **{}** with 95% confidence.",
                    prediction.clogs_left,
                    prediction.chaos_dwarf_kc,
                    prediction.average_chunkroll_date.format("%d %B %Y"),
                    prediction.lower_bound_chunkroll_date.format("%d %B %Y"),
                    prediction.upper_bound_chunkroll_date.format("%d %B %Y"),
                ),
            )
            .await?;
        }
        Ok(())
    }
}

async fn poll_once(
    player: &PlayerToTrack,
    conn: Arc<Mutex<rusqlite::Connection>>,
    http: Arc<serenity::http::Http>,
    channels: &HashMap<ChannelId, Vec<PlayerConfig>>,
) -> eyre::Result<()> {
    // Get previous metrics
    let player_name = player.name.clone();
    let conn_clone = Arc::clone(&conn);
    let prev_metrics: HashMap<String, hiscore_lookup::Metric> =
        tokio::task::spawn_blocking(move || {
            let guard = conn_clone.lock().unwrap();
            db::last_scores(&guard, &player_name)
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
            .find(|player_config| player_config.player_name == player.name)
        else {
            continue;
        };

        let metric_updates = metric_updates(&metrics, &prev_metrics);

        if metric_updates.triggers_post(player_config) {
            println!("Triggered post for {}", player.name);
            let message = metric_updates.get_full_update_message(player_config);
            println!("{}", message);
        } else if !metric_updates.is_empty() {
            println!("Got update for {}, but no post triggered", player.name);
            let message = metric_updates.get_full_update_message(player_config);
            println!("{}", message);
        }

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

            if let Some(message) =
                update_post::get_update_message(metric, prev_metric, player_config)?
            {
                if let Err(e) = channel_id.say(&http, &message).await {
                    eprintln!("Discord send error: {:#}", e);
                }
            }
        }

        if *channel_id == 871879186732707853
            && player_config.player_name.eq_ignore_ascii_case("OneChunkUp")
            && (metric_updates.metric_was_updated("Hitpoints")
                || metric_updates.metric_was_updated("Collections Logged"))
        {
            let prediction = chunkroll_predictor::predict_chunkroll_date(&metrics)?;
            if prediction.clogs_left == 0 {
                continue;
            }
            let message = format!(
                "Limpwurt still needs {} pieces of Dagon'hai robes, and has killed {} chaos dwarves so far. Chunkroll is estimated on **{}**, and between **{}** and **{}** with 95% confidence.",
                prediction.clogs_left,
                prediction.chaos_dwarf_kc,
                prediction.average_chunkroll_date.format("%d %B %Y"),
                prediction.lower_bound_chunkroll_date.format("%d %B %Y"),
                prediction.upper_bound_chunkroll_date.format("%d %B %Y"),
            );
            channel_id.say(&http, message).await?;
        }
    }

    // Insert snapshot
    let fetched_at = Utc::now().to_rfc3339();
    let player_name = player.name.clone();
    let conn_clone = Arc::clone(&conn);
    tokio::task::spawn_blocking(move || {
        let guard = conn_clone.lock().unwrap();
        db::insert_snapshot(&guard, &player_name, &fetched_at, &metrics)
    })
    .await??;

    Ok(())
}

async fn player_loop(
    player: PlayerToTrack,
    conn: Arc<Mutex<rusqlite::Connection>>,
    http: Arc<serenity::http::Http>,
    channels: Arc<HashMap<ChannelId, Vec<PlayerConfig>>>,
) {
    loop {
        if let Err(e) = poll_once(&player, Arc::clone(&conn), Arc::clone(&http), &channels).await {
            eprintln!("[{}] Error polling: {:#}", player.name, e);
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
