# Limpwurt bot

Tracks osrs players (Like Limpwurt) on the hiscores, and posts achievements in Discord channels.

Example `config.json`:
```
{
  "playersToTrack": ["OneChunkUp"],
  "channels": {
    "812770607527231488": [
      {
        "playerName": "OneChunkUp",
        "playerAlias": "Limpwurt",
        "playerExplanation": "",
        "showExpGain": false,
        "showLevelups": true,
        "showKcIncreases": true,
        "metricsWhitelist": [],
        "metricsBlacklist": [
          "Overall",
          "Clue Scrolls (all)",
          "Brutus",
          "The Royal Titans"
        ]
      }
    ]
  }
}

```
