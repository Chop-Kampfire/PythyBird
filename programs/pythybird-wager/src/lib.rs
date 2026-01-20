use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

declare_id!("11111111111111111111111111111111");

/// Maximum number of players per race
pub const MAX_PLAYERS: usize = 4;

/// Lobby code length
pub const LOBBY_CODE_LEN: usize = 6;

#[program]
pub mod pythybird_wager {
    use super::*;

    /// Create a new race with wagering
    /// Only the host can create a race
    pub fn create_race(
        ctx: Context<CreateRace>,
        lobby_code: String,
        wager_amount: u64,
    ) -> Result<()> {
        require!(lobby_code.len() == LOBBY_CODE_LEN, WagerError::InvalidLobbyCode);
        require!(wager_amount > 0, WagerError::InvalidWagerAmount);

        let race = &mut ctx.accounts.race;
        let mut code_bytes = [0u8; LOBBY_CODE_LEN];
        code_bytes.copy_from_slice(lobby_code.as_bytes());

        race.lobby_code = code_bytes;
        race.host = ctx.accounts.host.key();
        race.token_mint = ctx.accounts.token_mint.key();
        race.wager_amount = wager_amount;
        race.escrow_bump = ctx.bumps.escrow;
        race.players = Vec::new();
        race.status = RaceStatus::Waiting;
        race.winner = None;
        race.created_at = Clock::get()?.unix_timestamp;

        msg!("Race created: {} with wager {} tokens", lobby_code, wager_amount);
        Ok(())
    }

    /// Deposit wager to join the race
    /// Any player can deposit to join (including host)
    pub fn deposit_wager(ctx: Context<DepositWager>) -> Result<()> {
        let race = &mut ctx.accounts.race;
        let player = ctx.accounts.player.key();

        // Validations
        require!(race.status == RaceStatus::Waiting, WagerError::RaceNotWaiting);
        require!(race.players.len() < MAX_PLAYERS, WagerError::RaceFull);
        require!(!race.players.contains(&player), WagerError::AlreadyDeposited);

        // Transfer tokens from player to escrow
        let cpi_accounts = Transfer {
            from: ctx.accounts.player_token_account.to_account_info(),
            to: ctx.accounts.escrow.to_account_info(),
            authority: ctx.accounts.player.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, race.wager_amount)?;

        // Add player to race
        race.players.push(player);

        msg!("Player {} deposited {} tokens", player, race.wager_amount);
        Ok(())
    }

    /// Start the race (transition from Waiting to Racing)
    /// Only host can start, requires at least 2 players
    pub fn start_race(ctx: Context<StartRace>) -> Result<()> {
        let race = &mut ctx.accounts.race;

        require!(race.status == RaceStatus::Waiting, WagerError::RaceNotWaiting);
        require!(ctx.accounts.host.key() == race.host, WagerError::NotHost);
        require!(race.players.len() >= 2, WagerError::NotEnoughPlayers);

        race.status = RaceStatus::Racing;

        msg!("Race started with {} players", race.players.len());
        Ok(())
    }

    /// Declare the winner and distribute the pot
    /// Only host can declare winner, race must be in Racing status
    pub fn declare_winner(ctx: Context<DeclareWinner>, winner: Pubkey) -> Result<()> {
        let race = &mut ctx.accounts.race;

        // Validations
        require!(race.status == RaceStatus::Racing, WagerError::RaceNotRacing);
        require!(ctx.accounts.host.key() == race.host, WagerError::NotHost);
        require!(race.players.contains(&winner), WagerError::WinnerNotInRace);

        // Calculate total pot
        let total_pot = race.wager_amount.checked_mul(race.players.len() as u64)
            .ok_or(WagerError::Overflow)?;

        // Transfer entire pot to winner
        let race_key = ctx.accounts.race.key();
        let seeds = &[
            b"escrow",
            race_key.as_ref(),
            &[race.escrow_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.escrow.to_account_info(),
            to: ctx.accounts.winner_token_account.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        token::transfer(cpi_ctx, total_pot)?;

        // Update race state
        race.winner = Some(winner);
        race.status = RaceStatus::Completed;

        msg!("Winner {} received {} tokens", winner, total_pot);
        Ok(())
    }

    /// Cancel the race (host only, before racing starts)
    /// Sets status to Cancelled, allowing players to claim refunds
    pub fn cancel_race(ctx: Context<CancelRace>) -> Result<()> {
        let race = &mut ctx.accounts.race;

        require!(race.status == RaceStatus::Waiting, WagerError::RaceNotWaiting);
        require!(ctx.accounts.host.key() == race.host, WagerError::NotHost);

        race.status = RaceStatus::Cancelled;

        msg!("Race cancelled by host");
        Ok(())
    }

    /// Claim refund after race cancellation
    /// Any player who deposited can claim their wager back
    pub fn claim_refund(ctx: Context<ClaimRefund>) -> Result<()> {
        let race = &mut ctx.accounts.race;
        let player = ctx.accounts.player.key();

        // Validations
        require!(race.status == RaceStatus::Cancelled, WagerError::RaceNotCancelled);
        require!(race.players.contains(&player), WagerError::NotInRace);

        // Transfer refund from escrow to player
        let race_key = ctx.accounts.race.key();
        let seeds = &[
            b"escrow",
            race_key.as_ref(),
            &[race.escrow_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.escrow.to_account_info(),
            to: ctx.accounts.player_token_account.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        token::transfer(cpi_ctx, race.wager_amount)?;

        // Remove player from race to prevent double refund
        race.players.retain(|p| p != &player);

        msg!("Player {} claimed refund of {} tokens", player, race.wager_amount);
        Ok(())
    }
}

// ============================================
// ACCOUNTS
// ============================================

#[derive(Accounts)]
#[instruction(lobby_code: String)]
pub struct CreateRace<'info> {
    #[account(
        init,
        payer = host,
        space = 8 + Race::INIT_SPACE,
        seeds = [b"race", lobby_code.as_bytes()],
        bump
    )]
    pub race: Account<'info, Race>,

    #[account(
        init,
        payer = host,
        seeds = [b"escrow", race.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = escrow,
    )]
    pub escrow: Account<'info, TokenAccount>,

    pub token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub host: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct DepositWager<'info> {
    #[account(
        mut,
        seeds = [b"race", race.lobby_code.as_ref()],
        bump
    )]
    pub race: Account<'info, Race>,

    #[account(
        mut,
        seeds = [b"escrow", race.key().as_ref()],
        bump = race.escrow_bump,
    )]
    pub escrow: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = player_token_account.mint == race.token_mint,
        constraint = player_token_account.owner == player.key(),
    )]
    pub player_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct StartRace<'info> {
    #[account(
        mut,
        seeds = [b"race", race.lobby_code.as_ref()],
        bump
    )]
    pub race: Account<'info, Race>,

    pub host: Signer<'info>,
}

#[derive(Accounts)]
pub struct DeclareWinner<'info> {
    #[account(
        mut,
        seeds = [b"race", race.lobby_code.as_ref()],
        bump
    )]
    pub race: Account<'info, Race>,

    #[account(
        mut,
        seeds = [b"escrow", race.key().as_ref()],
        bump = race.escrow_bump,
    )]
    pub escrow: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = winner_token_account.mint == race.token_mint,
    )]
    pub winner_token_account: Account<'info, TokenAccount>,

    pub host: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CancelRace<'info> {
    #[account(
        mut,
        seeds = [b"race", race.lobby_code.as_ref()],
        bump
    )]
    pub race: Account<'info, Race>,

    pub host: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimRefund<'info> {
    #[account(
        mut,
        seeds = [b"race", race.lobby_code.as_ref()],
        bump
    )]
    pub race: Account<'info, Race>,

    #[account(
        mut,
        seeds = [b"escrow", race.key().as_ref()],
        bump = race.escrow_bump,
    )]
    pub escrow: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = player_token_account.mint == race.token_mint,
        constraint = player_token_account.owner == player.key(),
    )]
    pub player_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

// ============================================
// STATE
// ============================================

#[account]
#[derive(InitSpace)]
pub struct Race {
    /// 6-character lobby code
    pub lobby_code: [u8; LOBBY_CODE_LEN],

    /// Host wallet address (authority for race control)
    pub host: Pubkey,

    /// Token mint for the wager
    pub token_mint: Pubkey,

    /// Wager amount per player (in token's smallest unit)
    pub wager_amount: u64,

    /// Bump seed for escrow PDA
    pub escrow_bump: u8,

    /// Players who have deposited (max 4)
    #[max_len(MAX_PLAYERS)]
    pub players: Vec<Pubkey>,

    /// Current race status
    pub status: RaceStatus,

    /// Winner pubkey (set after race ends)
    pub winner: Option<Pubkey>,

    /// Unix timestamp when race was created
    pub created_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum RaceStatus {
    Waiting,
    Racing,
    Completed,
    Cancelled,
}

// ============================================
// ERRORS
// ============================================

#[error_code]
pub enum WagerError {
    #[msg("Invalid lobby code length")]
    InvalidLobbyCode,

    #[msg("Wager amount must be greater than 0")]
    InvalidWagerAmount,

    #[msg("Race is not in waiting status")]
    RaceNotWaiting,

    #[msg("Race is not in racing status")]
    RaceNotRacing,

    #[msg("Race is not cancelled")]
    RaceNotCancelled,

    #[msg("Race is full (max 4 players)")]
    RaceFull,

    #[msg("Player has already deposited")]
    AlreadyDeposited,

    #[msg("Only the host can perform this action")]
    NotHost,

    #[msg("Need at least 2 players to start")]
    NotEnoughPlayers,

    #[msg("Winner is not a participant in this race")]
    WinnerNotInRace,

    #[msg("Player is not in this race")]
    NotInRace,

    #[msg("Arithmetic overflow")]
    Overflow,
}
