# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Pyth Flappy is a browser-based Flappy Bird-style game that generates obstacles using real-time cryptocurrency price data from the Pyth Network. The entire application is contained in a single `index.html` file with embedded CSS and JavaScript.

## Development

Open `index.html` directly in a browser - no build tools or server required. For development with live reload, use any local server (e.g., `python -m http.server` or VS Code Live Server extension).

## Architecture

### Single-File Structure
Everything is in `index.html`:
- CSS styles (lines 10-540) - CSS custom properties for theming in `:root`, includes feed discovery UI styles
- HTML markup (lines 641-752) - Game canvas, overlays, feed discovery components, and side panels
- JavaScript game engine (lines 754-1290)

### Key JavaScript Components

**State Management** (`state` object): Centralized game state including:
- Price feed data (`currentPrice`, `priceHistory`, `currentFeedSymbol`)
- Feed discovery (`allFeeds`, `filteredFeeds`, `searchQuery`, `selectedAssetClass`)
- Game state (`isPlaying`, `score`, `playerY`, `obstacles`)
- SSE connection (`eventSource`, `connected`)

**Feed Discovery System**: Dynamic feed selection with search and asset class filtering:
- `fetchAllFeeds()`: Fetches all available feeds from Hermes `/v2/price_feeds` endpoint
- `initFeedDiscovery()`: Sets up event handlers for search input, asset class dropdown
- `filterFeeds()`: Filters feeds by search query and asset class
- `renderFeedResults()`: Renders filtered feed list (limited to 50 for performance)
- `selectFeed(feedId, symbol)`: Updates state and connects to selected feed

**Price Feed Connection**: Uses Server-Sent Events (SSE) to stream real-time prices from Pyth Hermes endpoint (`https://hermes.pyth.network`). Price levels directly control obstacle gap positions.

**Game Loop**: Standard `requestAnimationFrame` loop with:
- `update()`: Physics, collision detection, obstacle generation
- `render()`: Canvas drawing (grid, price line, obstacles, player)

**Obstacle Generation**: Gap positions use the same normalization as the green price line display - obstacles track the current price position within the min/max range of recent prices.

### Game Constants
Tunable values: `GRAVITY`, `JUMP_FORCE`, `PLAYER_SIZE`, `OBSTACLE_WIDTH`, `OBSTACLE_GAP`, `OBSTACLE_SPEED`, `OBSTACLE_INTERVAL`

### Asset Class Categories
Available filter categories: All, Commodities, Crypto, Crypto Index, Crypto NAV, Crypto Redemption Rate, ECO, Equity, FX, Kalshi, Metal, Rates
