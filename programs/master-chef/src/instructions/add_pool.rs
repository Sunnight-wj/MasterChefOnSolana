use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::{constants::*, events::*, state::MasterChef};

#[derive(Accounts)]
pub struct AddPool<'info> {

    #[account(
        mut,
        address = master_chef.load()?.admin,
    )]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub master_chef: AccountLoader<'info, MasterChef>,

    pub lp_mint: Box<Account<'info, Mint>>,

    pub reward_mint: Box<Account<'info, Mint>>,

    /// CHECK: ⋐ ͡⋄ ω ͡⋄ ⋑
    #[account(
        seeds = [
            LP_TOKEN_VAULT_AUTHORITY_SEED.as_bytes(),
            lp_mint.key().as_ref(),
            master_chef.key().as_ref(),
        ],
        bump,
    )]
    pub lp_token_vault_authority: AccountInfo<'info>,

    #[
        account(
            init,
            payer = admin,
            token::mint = lp_mint,
            token::authority = lp_token_vault_authority,
            seeds = [
                LP_TOKEN_VAULT_SEED.as_bytes(),
                lp_mint.key().as_ref(),
                master_chef.key().as_ref(),
            ],
            bump,
        )
    ]
    pub lp_token_vault: Box<Account<'info, TokenAccount>>,


    /// CHECK: ⋐ ͡⋄ ω ͡⋄ ⋑
    #[
        account(
            seeds = [
                REWARD_TOKEN_VAULT_AUTHORITY_SEED.as_bytes(),
                lp_mint.key().as_ref(),
                master_chef.key().as_ref(),
            ],
            bump,
        )
    ]
    pub reward_token_vault_authority: AccountInfo<'info>,

    #[
        account(
            init,
            payer = admin,
            token::mint = reward_mint,
            token::authority = reward_token_vault_authority,
            seeds = [
                REWARD_TOKEN_VAULT_SEED.as_bytes(),
                lp_mint.key().as_ref(),
                master_chef.key().as_ref(),
            ],
            bump,
        )
    ]
    pub reward_token_vault: Box<Account<'info, TokenAccount>>,

    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn add_pool(
    ctx: Context<AddPool>, 
    reward_token: Pubkey, 
    lp_token: Pubkey, 
    reward_per_slot: u64, 
    start_slot: u64, 
) -> Result<()> {
    let AddPool{
        master_chef: master_chef_loader,
        reward_token_vault,
        lp_token_vault,
        ..
    } = ctx.accounts;
    
    let mut master_chef = master_chef_loader.load_mut()?;
    let lp_token_vault_bump = *ctx.bumps.get("lp_token_vault").unwrap();
    let lp_token_vault_authority_bump = *ctx.bumps.get("lp_token_vault_authority").unwrap();
    let reward_token_vault_bump = *ctx.bumps.get("reward_token_vault").unwrap();
    let reward_token_vault_authority_bump = *ctx.bumps.get("reward_token_vault_authority").unwrap();

    let _ = master_chef.create_pool(
        reward_token, 
        lp_token, 
        start_slot, 
        reward_per_slot, 
        lp_token_vault.key(),
        lp_token_vault_bump, 
        lp_token_vault_authority_bump,
        reward_token_vault.key(),
        reward_token_vault_bump,
        reward_token_vault_authority_bump,
    );

    emit!(AddPoolEvent {
        header: EventHeader {
            master_chef: master_chef_loader.key(),
            signer: Some(ctx.accounts.admin.key())
        },
        reward_token,
        lp_token,
        start_slot,
        reward_per_slot,
    });
    Ok(())
}