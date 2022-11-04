use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod anchor_lottery {
    use super::*;

    pub fn init_lottery(ctx: Context<InitLottery>) -> Result<()> {
        let lottery = &mut ctx.accounts.lottery;
        lottery.bump = *ctx.bumps.get("lottery").unwrap();
        lottery.manager = ctx.accounts.manager.key();
        lottery.token_mint = ctx.accounts.token_mint.key();
        lottery.token_account = ctx.accounts.token_account.key();
        lottery.token_amount = 0;

        Ok(())
    }

    pub fn enter_player(ctx: Context<EnterPlayer>, amount: u64) -> Result<()> {
        // TODO: set participation fee
        let lottery = &mut ctx.accounts.lottery;
        lottery.token_amount += amount;
        let player = Player {
            address: ctx.accounts.player.key(),
            token_account: ctx.accounts.player_token_account.key()
        };
        lottery.players.push(player);

        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.player_token_account.to_account_info(),
                    to: ctx.accounts.lottery_token_account.to_account_info(),
                    authority: ctx.accounts.player.to_account_info()
                },
            )
            , amount)?;

        Ok(())
    }

    pub fn pick_winner(ctx: Context<PickWinner>) -> Result<()> {
        let lottery = &mut ctx.accounts.lottery;
        lottery.winner = lottery.pick_winner();
        
        Ok(())
    }

    pub fn claim_prize(ctx: Context<ClaimPrize>) -> Result<()> {
        let lottery = &mut ctx.accounts.lottery;
        let amount = lottery.token_amount;
        lottery.token_amount = 0;

        let manager = &ctx.accounts.manager;
        let lottery_token_account = &ctx.accounts.lottery_token_account;
        let player_token_account = &ctx.accounts.player_token_account;
        let winner_token_account = lottery.winner.token_account;

        require!(player_token_account.key() == winner_token_account, LotteryError::InvalidWinnerError);

        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: lottery_token_account.to_account_info(),
                    to: player_token_account.to_account_info(),
                    authority: lottery.to_account_info()
                },
                &[&["lottery".as_ref(), manager.key().as_ref(), &[lottery.bump]]]
            )
            , amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitLottery<'info> {
    #[account(mut)]
    manager: Signer<'info>,
    token_mint: Account<'info, Mint>,
    #[account(
        init, 
        payer = manager, 
        space = 8 + Lottery::LEN, 
        seeds = ["lottery".as_bytes(), manager.key().as_ref()], 
        bump 
    )]
    pub lottery: Account<'info, Lottery>,
    #[account(
        init, 
        payer = manager, 
        token::mint = token_mint, 
        token::authority = lottery
    )]
    token_account: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct EnterPlayer<'info> {
    #[account(mut)]
    player: Signer<'info>,
    #[account(
        mut,
        seeds = ["lottery".as_bytes(), lottery.manager.as_ref()],
        bump = lottery.bump
    )]
    pub lottery: Account<'info, Lottery>,
    #[account(mut, constraint = player_token_account.mint == lottery_token_account.mint)]
    player_token_account: Account<'info, TokenAccount>,
    #[account(mut, constraint = lottery_token_account.key() == lottery.token_account)]
    lottery_token_account: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct PickWinner<'info> {
    manager: Signer<'info>,
    #[account(
        mut,
        seeds = ["lottery".as_bytes(), lottery.manager.as_ref()],
        bump = lottery.bump
    )]
    pub lottery: Account<'info, Lottery>,
}

#[derive(Accounts)]
pub struct ClaimPrize<'info> {
    manager: Signer<'info>,
    #[account(
        mut,
        seeds = ["lottery".as_bytes(), lottery.manager.as_ref()],
        bump = lottery.bump
    )]
    pub lottery: Account<'info, Lottery>,
    #[account(
        mut,
        constraint = player_token_account.mint == lottery_token_account.mint
    )]
    player_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = lottery_token_account.key() == lottery.token_account,
    )]
    lottery_token_account: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>
}

#[account]
pub struct Lottery {
    manager: Pubkey,      // 32
    players: Vec<Player>, // max 3 players. 4 + 64 * 3
    winner: Player, // 64

    token_mint: Pubkey,    // 32
    token_account: Pubkey, // 32
    token_amount: u64,      // 8

    bump: u8, // 1
}

impl Lottery {
    pub const LEN: usize = 32 + (4 + Player::LEN * 3) + 64 + 32 + 32 + 8 + 1;

    fn random() -> usize {
        // TODO: use oracle to get verifiable random number
        return 1;
    }

    pub fn pick_winner(&self) -> Player {
        let winner = self.players[Lottery::random()];
        return winner;
    }
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug, PartialEq, Copy)]
#[derive(Default)]
pub struct Player {
    address: Pubkey, // 32
    token_account: Pubkey // 32
}

impl Player {
    pub const LEN: usize = 32 + 32;
}

#[error_code]
pub enum LotteryError {
    #[msg("The player is not the winner of lottery prize")]
    InvalidWinnerError,
}
