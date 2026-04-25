# Limpwurt bot

Tracks osrs players (Like Limpwurt) on the hiscores, and posts achievements in Discord channels.

Example `config.json`:
```
{
  "playersToTrack": [{ "name": "OneChunkUp", "type": "main" }],
  "channels": {
    "812770607527231488": [
      {
        "playerName": "OneChunkUp",
        "playerAlias": "Limpwurt",
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

## To add bot to a server:
Click link: https://discord.com/oauth2/authorize?client_id=1478063186497241273&permissions=2112&integration_type=0&scope=bot

Click "Add to Server"
