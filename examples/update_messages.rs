//! Replays the hiscores database through the update message system (`metric_updates` and
//! `MetricUpdates::update_messages`), and prints the latest messages it would have posted in
//! Discord, using the current code and `config.json`.
//!
//! Usage:
//!     cargo run --example update_messages -- [-n COUNT] [--player NAME] [--channel ID]
//!                                            [--db PATH] [--config PATH]
//!
//! Each hiscores update may post several messages, which are printed together as one post, one
//! bullet per Discord message. When the same snapshot produces identical messages in several
//! channels, the post is printed once, listing all the channels. To avoid splitting a post, slightly
//! more than COUNT messages may be printed.

use std::collections::HashMap;

use limpwurt_chunk_roll::config::Config;
use limpwurt_chunk_roll::hiscore_lookup::Metric;
use limpwurt_chunk_roll::update_post::metric_updates;
use rusqlite::{Connection, OpenFlags, OptionalExtension, Statement, params};
use serenity::all::ChannelId;

const USAGE: &str =
    "Usage: update_messages [-n COUNT] [--player NAME] [--channel ID] [--db PATH] [--config PATH]";

struct Args {
    count: usize,
    player: Option<String>,
    channel: Option<u64>,
    db_path: String,
    config_path: String,
}

fn parse_args() -> eyre::Result<Args> {
    let mut args = Args {
        count: 10,
        player: None,
        channel: None,
        db_path: "hiscores.db".to_string(),
        config_path: "config.json".to_string(),
    };
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        let mut value = || {
            iter.next()
                .ok_or_else(|| eyre::eyre!("Missing value for {arg}\n{USAGE}"))
        };
        match arg.as_str() {
            "-n" => args.count = value()?.parse()?,
            "--player" => args.player = Some(value()?),
            "--channel" => args.channel = Some(value()?.parse()?),
            "--db" => args.db_path = value()?,
            "--config" => args.config_path = value()?,
            _ => return Err(eyre::eyre!("Unknown argument {arg}\n{USAGE}")),
        }
    }
    Ok(args)
}

/// All the messages posted to a set of channels after a single hiscores update
struct DiscordPost {
    fetched_at: String,
    player: String,
    channel_ids: Vec<ChannelId>,
    messages: Vec<String>,
}

fn load_metrics(stmt: &mut Statement, snapshot_id: i64) -> rusqlite::Result<Vec<Metric>> {
    stmt.query_map(params![snapshot_id], |row| {
        Ok(Metric {
            name: row.get(0)?,
            rank: row.get(1)?,
            score: row.get(2)?,
            exp: row.get(3)?,
        })
    })?
    .collect()
}

fn main() -> eyre::Result<()> {
    let args = parse_args()?;

    let config: Config = serde_json::from_slice(&std::fs::read(&args.config_path)?)?;
    config.validate()?;

    // Sorted, so that the output is deterministic
    let mut channels: Vec<_> = config
        .channels
        .iter()
        .filter(|(channel_id, _)| args.channel.is_none_or(|c| channel_id.get() == c))
        .collect();
    channels.sort_by_key(|(channel_id, _)| **channel_id);
    if channels.is_empty() {
        eyre::bail!("Channel {:?} is not in {}", args.channel, args.config_path);
    }

    // Resolve the player filter to the exact name used in the database
    let player_filter = match &args.player {
        None => None,
        Some(name) => Some(
            channels
                .iter()
                .flat_map(|(_, players)| players.iter())
                .find(|p| p.player_name.eq_ignore_ascii_case(name))
                .map(|p| p.player_name.clone())
                .ok_or_else(|| eyre::eyre!("Player {name} is not configured in any channel"))?,
        ),
    };

    // Read-only, so this is safe to run while the bot is writing to the database
    let conn = Connection::open_with_flags(&args.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut snapshots_stmt = conn.prepare(
        "SELECT id, player, fetched_at FROM snapshots
         WHERE ?1 IS NULL OR player = ?1
         ORDER BY id DESC",
    )?;
    let mut prev_snapshot_stmt = conn.prepare(
        "SELECT id FROM snapshots WHERE player = ?1 AND id < ?2 ORDER BY id DESC LIMIT 1",
    )?;
    let mut metrics_stmt = conn
        .prepare("SELECT name, rank, score, exp FROM metrics WHERE snapshot_id = ?1 ORDER BY id")?;

    // Walking backwards, each snapshot is first the newer side of a pair, and then the older side
    // of the next pair for that player. Cache the older side, to avoid loading it twice.
    let mut older_metrics_cache: HashMap<String, (i64, Vec<Metric>)> = HashMap::new();
    let mut posts: Vec<DiscordPost> = Vec::new();
    let mut message_count = 0;

    let mut snapshots = snapshots_stmt.query(params![player_filter])?;
    while message_count < args.count
        && let Some(row) = snapshots.next()?
    {
        let snapshot_id: i64 = row.get(0)?;
        let player: String = row.get(1)?;
        let fetched_at: String = row.get(2)?;

        if !channels
            .iter()
            .any(|(_, players)| players.iter().any(|p| p.player_name == player))
        {
            continue;
        }

        // The bot compares each snapshot against the previous snapshot of the same player
        let Some(prev_snapshot_id) = prev_snapshot_stmt
            .query_row(params![player, snapshot_id], |row| row.get::<_, i64>(0))
            .optional()?
        else {
            continue;
        };

        let metrics = match older_metrics_cache.remove(&player) {
            Some((cached_id, metrics)) if cached_id == snapshot_id => metrics,
            _ => load_metrics(&mut metrics_stmt, snapshot_id)?,
        };
        let prev_metrics = load_metrics(&mut metrics_stmt, prev_snapshot_id)?;

        let updates = metric_updates(
            &metrics,
            &prev_metrics
                .iter()
                .map(|m| (m.name.clone(), m.clone()))
                .collect(),
        );
        older_metrics_cache.insert(player.clone(), (prev_snapshot_id, prev_metrics));

        if updates.is_empty() {
            continue;
        }

        // Same logic as in `poll_once`
        let mut snapshot_posts: Vec<DiscordPost> = Vec::new();
        for (channel_id, players) in &channels {
            let Some(player_config) = players.iter().find(|p| p.player_name == player) else {
                continue;
            };
            let messages = updates.update_messages(player_config);
            if messages.is_empty() {
                continue;
            }
            match snapshot_posts.iter_mut().find(|p| p.messages == messages) {
                Some(post) => post.channel_ids.push(**channel_id),
                None => {
                    message_count += messages.len();
                    snapshot_posts.push(DiscordPost {
                        fetched_at: fetched_at.clone(),
                        player: player.clone(),
                        channel_ids: vec![**channel_id],
                        messages,
                    })
                }
            }
        }
        posts.extend(snapshot_posts);
    }

    // Print oldest first, like a Discord channel
    for post in posts.iter().rev() {
        let time = chrono::DateTime::parse_from_rfc3339(&post.fetched_at)?
            .to_utc()
            .format("%Y-%m-%d %H:%M UTC");
        let channels = post
            .channel_ids
            .iter()
            .map(|id| format!("#{id}"))
            .collect::<Vec<_>>()
            .join(", ");
        println!("── {time} · {} → {channels}", post.player);
        for message in &post.messages {
            println!("• {message}");
        }
        println!();
    }
    if message_count < args.count {
        println!("(Only found {message_count} messages in the database)");
    }

    Ok(())
}
