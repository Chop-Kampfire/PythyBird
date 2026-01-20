-- PythyBird Wagering Schema Migration
-- Run this in Supabase SQL Editor

-- Add wager columns to lobbies table
ALTER TABLE lobbies ADD COLUMN IF NOT EXISTS wager_enabled BOOLEAN DEFAULT false;
ALTER TABLE lobbies ADD COLUMN IF NOT EXISTS wager_amount BIGINT DEFAULT 0;
ALTER TABLE lobbies ADD COLUMN IF NOT EXISTS token_mint VARCHAR(44);
ALTER TABLE lobbies ADD COLUMN IF NOT EXISTS token_symbol VARCHAR(20);
ALTER TABLE lobbies ADD COLUMN IF NOT EXISTS token_decimals SMALLINT DEFAULT 9;
ALTER TABLE lobbies ADD COLUMN IF NOT EXISTS race_pda VARCHAR(44);

-- Add deposit tracking to lobby_players table
ALTER TABLE lobby_players ADD COLUMN IF NOT EXISTS has_deposited BOOLEAN DEFAULT false;
ALTER TABLE lobby_players ADD COLUMN IF NOT EXISTS deposit_signature VARCHAR(88);

-- Create index for wager-enabled lobbies
CREATE INDEX IF NOT EXISTS idx_lobbies_wager ON lobbies(wager_enabled) WHERE wager_enabled = true;

-- Comment: RLS policies should already allow public read/write from existing setup
