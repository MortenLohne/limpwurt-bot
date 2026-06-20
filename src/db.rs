use rusqlite::{Connection, params};

use crate::hiscore_lookup::Metric;

pub fn open(path: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS snapshots (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            player     TEXT    NOT NULL,
            fetched_at TEXT    NOT NULL
        );
        CREATE TABLE IF NOT EXISTS metrics (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            snapshot_id INTEGER NOT NULL REFERENCES snapshots(id),
            name        TEXT    NOT NULL,
            rank        INTEGER,
            score       INTEGER NOT NULL,
            exp         INTEGER
        );
        CREATE TABLE IF NOT EXISTS update_posts (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            snapshot_id INTEGER NOT NULL REFERENCES snapshots(id),
            channel_id  INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_snapshots_player ON snapshots(player);
        CREATE INDEX IF NOT EXISTS idx_metrics_snapshot_id ON metrics(snapshot_id);
        CREATE INDEX IF NOT EXISTS idx_update_posts_channel_id ON update_posts(channel_id);",
    )?;
    Ok(conn)
}

pub fn insert_snapshot(
    conn: &Connection,
    player: &str,
    fetched_at: &str,
    metrics: &[Metric],
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO snapshots (player, fetched_at) VALUES (?1, ?2)",
        params![player, fetched_at],
    )?;
    let snapshot_id = conn.last_insert_rowid();
    for metric in metrics {
        conn.execute(
            "INSERT INTO metrics (snapshot_id, name, rank, score, exp) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                snapshot_id,
                metric.name,
                metric.rank,
                metric.score,
                metric.exp
            ],
        )?;
    }
    Ok(snapshot_id)
}

/// Returns all metrics from the most recent snapshot for the given player.
pub fn last_scores(conn: &Connection, player: &str) -> rusqlite::Result<Vec<Metric>> {
    let mut stmt = conn.prepare(
        "SELECT m.name, m.rank, m.score, m.exp
         FROM metrics m
         JOIN snapshots s ON m.snapshot_id = s.id
         WHERE s.player = ?1
           AND s.id = (
               SELECT id FROM snapshots WHERE player = ?1 ORDER BY id DESC LIMIT 1
           )",
    )?;
    let rows = stmt.query_map(params![player], |row| {
        Ok(Metric {
            name: row.get(0)?,
            rank: row.get(1)?,
            score: row.get(2)?,
            exp: row.get(3)?,
        })
    })?;
    rows.collect()
}

/// When an update message is sent, log in the database, to avoid spamming Discord
pub fn insert_limpwurt_message(
    conn: &Connection,
    snapshot_id: i64,
    channel_id: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO update_posts (snapshot_id, channel_id) VALUES (?1, ?2)",
        params![snapshot_id, channel_id],
    )?;
    Ok(())
}

/// Returns the metrics from the most recent snapshot that triggered an update message for the given player,
/// along with the timestamp of the snapshot.
pub fn get_last_update_post_metrics(
    conn: &Connection,
    player: &str,
    channel_id: i64,
) -> rusqlite::Result<(Vec<Metric>, String)> {
    let mut stmt = conn.prepare(
        "SELECT m.name, m.rank, m.score, m.exp, s.fetched_at
         FROM metrics m
         JOIN snapshots s ON m.snapshot_id = s.id
         WHERE s.id = (
             SELECT up.snapshot_id
             FROM update_posts up
             JOIN snapshots s2 ON up.snapshot_id = s2.id
             WHERE s2.player = ?1 AND up.channel_id = ?2
             ORDER BY up.id DESC
             LIMIT 1
         )",
    )?;
    let rows = stmt.query_map(params![player, channel_id], |row| {
        Ok((
            Metric {
                name: row.get(0)?,
                rank: row.get(1)?,
                score: row.get(2)?,
                exp: row.get(3)?,
            },
            row.get::<_, String>(4)?,
        ))
    })?;

    let mut metrics = Vec::new();
    let mut fetched_at = String::new();
    for row in rows {
        let (metric, ts) = row?;
        fetched_at = ts;
        metrics.push(metric);
    }
    Ok((metrics, fetched_at))
}
