# Pyth Flappy

A browser-based Flappy Bird-style game powered by real-time cryptocurrency price data from the [Pyth Network](https://pyth.network).

## How It Works

### Real-Time Price Feeds

The game connects to Pyth Network's Hermes endpoint using Server-Sent Events (SSE) to stream live price data. Each price update includes:

- **Price** - The current asset price
- **Confidence Interval** - Pyth's measure of price uncertainty
- **Timestamp** - When the price was recorded

### Price-Driven Obstacles

Unlike traditional Flappy Bird where obstacles are randomly placed, Pyth Flappy generates obstacle gaps based on actual market movements:

1. The game tracks a rolling window of the last 30 price updates
2. Each price is normalized within the min/max range of recent prices
3. The obstacle gap position mirrors the green price line on screen
4. As the price moves up, gaps shift upward — and vice versa

This means **volatile markets create more challenging gameplay** as the gaps move more dramatically between obstacles.

### The Green Price Line

A semi-transparent green line traces the recent price history across the game canvas. This same calculation determines where obstacle gaps appear, so watching the price line helps you anticipate upcoming gaps.

### Gameplay

- Press **SPACE** or **CLICK** to flap and stay airborne
- Navigate through the gaps in the purple obstacles
- Each obstacle passed earns 1 point
- Hitting an obstacle or boundary ends the game

### Online Leaderboard

Connect your Phantom (Solana) wallet to:
- Submit scores to the global leaderboard
- Track your personal best
- Filter scores by price feed

### Available Price Feeds

Choose from hundreds of Pyth price feeds across multiple asset classes:
- Crypto (BTC, ETH, SOL, etc.)
- Forex (EUR/USD, GBP/USD, etc.)
- Equities
- Commodities
- And more

---

Built with Pyth Network price feeds. No server required — runs entirely in the browser.
