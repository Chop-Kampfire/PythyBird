# PythyBird Solana Wagering - Session Handoff

## Current Status
**Branch:** `feature/solana-wagering`
**Last Commit:** `92881bc` - Add Solana wagering system for multiplayer races (devnet)
**Date:** January 2025

---

## What Was Built

### 1. Anchor Program (`programs/pythybird-wager/src/lib.rs`)
A complete Solana smart contract for wagering with:
- **Race Account**: Stores lobby code, host, token mint, wager amount, players list, status
- **Escrow PDA**: Program-derived address that holds deposited tokens
- **Instructions**:
  - `create_race` - Host creates race with token and wager amount
  - `deposit_wager` - Players deposit tokens to join
  - `start_race` - Host starts when all deposited
  - `declare_winner` - Host declares winner, program transfers all tokens to winner
  - `cancel_race` - Host cancels race (before start)
  - `claim_refund` - Players reclaim tokens after cancellation

### 2. Database Migration (`migrations/001_add_wagering.sql`)
SQL migration to add wager columns to Supabase tables:
- `lobbies`: wager_enabled, wager_amount, token_mint, token_symbol, token_decimals, race_pda
- `lobby_players`: has_deposited, deposit_signature

### 3. Client Integration (`index.html`)
- Solana Web3.js CDN added
- Wager state management
- Create Lobby modal with wagering options (token selector, amount input)
- Waiting room shows wager info, pot total, deposit status per player
- Deposit button with loading states
- Start button requires all deposits when wagering enabled
- PLAYER_DEPOSIT broadcast event for real-time sync

---

## What's Pending

### Immediate Next Steps

1. **Run Database Migration**
   - Go to Supabase Dashboard → SQL Editor
   - Paste and run `migrations/001_add_wagering.sql`

2. **Install Solana Development Tools** (on your machine)
   ```powershell
   # Install Rust
   winget install Rustlang.Rustup
   rustup default stable

   # Install Solana CLI
   winget install Solana.Solana
   solana config set --url devnet
   solana-keygen new

   # Install Anchor CLI
   cargo install --git https://github.com/coral-xyz/anchor avm --locked
   avm install latest
   avm use latest
   ```

3. **Build and Deploy Program**
   ```bash
   cd "C:\Users\tyron\OneDrive\Documents\Kampfire Vibez\Pythy Bird"
   anchor build
   anchor deploy --provider.cluster devnet
   ```

4. **Update Program ID**
   After deploying, update the program ID in:
   - `Anchor.toml` (line 4)
   - `index.html` - search for `WAGER_PROGRAM_ID`
   - `programs/pythybird-wager/src/lib.rs` - the `declare_id!` macro

5. **Get Devnet SOL for Testing**
   - https://faucet.solana.com/

### Alternative: Solana Playground
If you want to skip local toolchain setup:
1. Go to https://beta.solpg.io/
2. Create new Anchor project
3. Paste contents of `programs/pythybird-wager/src/lib.rs`
4. Build and deploy from browser
5. Copy the program ID back to your code

---

## Key Files Reference

| File | Purpose |
|------|---------|
| `index.html` | Main game - all UI and client logic |
| `programs/pythybird-wager/src/lib.rs` | Solana program (Rust) |
| `Anchor.toml` | Anchor configuration |
| `Cargo.toml` | Rust workspace config |
| `migrations/001_add_wagering.sql` | Supabase schema changes |

---

## Architecture Overview

```
Browser (index.html)
    │
    ├──► Supabase (lobby data, player status)
    │
    └──► Solana Devnet
            │
            └──► PythyBird Wager Program
                    │
                    ├── Race PDA (per lobby)
                    └── Escrow PDA (holds tokens)
```

### Wager Flow
1. Host creates lobby with wager enabled → Creates Race account on-chain
2. Players join lobby → Each deposits tokens to Escrow PDA
3. All deposited + all ready → Host can start race
4. Race ends → Host calls `declare_winner` → Program transfers pot to winner

---

## Testing Checklist

- [ ] Database migration run in Supabase
- [ ] Anchor program built successfully
- [ ] Program deployed to devnet
- [ ] Program ID updated in code
- [ ] Devnet SOL obtained for testing
- [ ] Create wager lobby works
- [ ] Deposit button triggers transaction
- [ ] Deposit status syncs to other players
- [ ] Start button appears when all deposited + ready
- [ ] Winner receives pot after race

---

## Code Locations for Key Functions

### Client-side (index.html)
- `initSolanaConnection()` - Line ~1964
- `depositWager()` - Line ~2091
- `handlePlayerDeposit()` - Line ~2153
- `updateWagerUI()` - Line ~2045
- `createLobby()` with wager params - Line ~2413

### Program (lib.rs)
- `create_race` instruction - Line ~18
- `deposit_wager` instruction - Line ~45
- `declare_winner` instruction - Line ~76
- `Race` account struct - Line ~170

---

## Notes

- The `depositWager()` function currently **simulates** the transaction (for UI testing)
- Once program is deployed, need to implement actual transaction building with:
  - Finding/creating associated token accounts
  - Building the deposit instruction
  - Signing with Phantom wallet
- Host authority model means host could theoretically cheat - acceptable for devnet MVP
- Production would need oracle or consensus-based winner verification

---

## Resume Commands

```bash
# Switch to wagering branch
cd "C:\Users\tyron\OneDrive\Documents\Kampfire Vibez\Pythy Bird"
git checkout feature/solana-wagering

# Check status
git status
git log --oneline -5
```

---

*Last updated: Session ended at Solana toolchain installation step*
