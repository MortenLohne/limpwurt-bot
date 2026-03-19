use std::fmt;

use eyre::ContextCompat as _;

/// Get the contents of all text elements below a node, as a flattened vector
fn texts_below_node(node: &ego_tree::NodeRef<scraper::Node>) -> Vec<String> {
    let mut texts = vec![];
    for child in node.children() {
        if let Some(text) = child.value().as_text()
            && !text.trim().is_empty()
        {
            texts.push(text.trim().to_string());
        }
        texts.extend(texts_below_node(&child));
    }
    texts
}

/// Represents a single highscores metric, i.e. a skill or killcount for a boss
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    // The rank relative to other players
    pub rank: Option<u32>,
    // For skills, this is the level. For bosses, this is the killcount. For minigames, it's the number of completions.
    pub score: u32,
    // Experience points for skills, None otherwise
    pub exp: Option<u32>,
}

impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.name)?;
        match self.rank {
            Some(rank) => write!(f, "Rank {}", rank)?,
            None => write!(f, "--")?,
        }
        write!(f, ", level/kc {}", self.score)?;
        match self.exp {
            Some(exp) => write!(f, ", {} exp", exp)?,
            None => write!(f, "")?,
        }
        Ok(())
    }
}

impl Metric {
    fn from_texts(texts: &[String]) -> eyre::Result<Metric> {
        if texts.len() < 3 {
            eyre::bail!(
                "Only {} words of text, not enough texts to parse a metric",
                texts.len()
            );
        }
        let name = texts[0].clone();
        let rank = if texts[1] == "--" {
            None
        } else {
            Some(
                texts[1]
                    .chars()
                    .filter(|ch| *ch != ',')
                    .collect::<String>()
                    .parse()?,
            )
        };
        let level_score = texts[2]
            .chars()
            .filter(|ch| *ch != ',')
            .collect::<String>()
            .parse()?;
        let exp = texts
            .get(3)
            .map(|exp_str| {
                exp_str
                    .chars()
                    .filter(|ch| *ch != ',')
                    .collect::<String>()
                    .parse()
            })
            .transpose()?;
        Ok(Metric {
            name,
            rank,
            score: level_score,
            exp,
        })
    }
}

/// Fetches the metrics for a given username from the hiscores.
pub async fn fetch_metrics(username: &str) -> eyre::Result<Vec<Metric>> {
    let html = reqwest::get(format!(
        "https://secure.runescape.com/m=hiscore_oldschool_ironman/hiscorepersonal?user1={}",
        username
    ))
    .await?
    .error_for_status()?
    .text()
    .await?;

    let parsed_html = scraper::Html::parse_document(&html);

    let Some(overall_node) = parsed_html.tree.nodes().find(|node_ref| {
        node_ref
            .value()
            .as_text()
            .is_some_and(|text| text.trim() == "Overall")
    }) else {
        eyre::bail!("Couldn't find overall html node");
    };

    let great_great_grandparent = overall_node
        .parent()
        .context("Couldn't find node parent")?
        .parent()
        .context("Couldn't find node parent")?
        .parent()
        .context("Couldn't find node parent")?
        .parent()
        .context("Couldn't find node parent")?;

    let mut metrics: Vec<Metric> = vec![];
    let mut errors = vec![];

    for great_great_aunts in great_great_grandparent.children() {
        let texts_below = texts_below_node(&great_great_aunts);
        if texts_below.is_empty() {
            continue;
        }
        let metric_result = Metric::from_texts(&texts_below);
        match metric_result {
            Ok(metric) => metrics.push(metric),
            Err(err) => errors.push(err),
        };
    }
    if !errors.is_empty() {
        eprintln!("Got {} errors while parsing hiscore metrics", errors.len());
    }
    Ok(metrics)
}
