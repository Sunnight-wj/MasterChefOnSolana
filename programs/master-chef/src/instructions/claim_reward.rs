use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount, Transfer}};
use fixed::types::I80F48;

use crate::{constants::*, events::{ClaimRewardEvent, EventHeader}, math_error, pool_signer, MasterChef, PoolVaultType, UserInfo, WrappedI80F48};

#[derive(Accounts)]
#[instruction(lp_token: Pubkey)]
pub struct ClaimReward<'info> {

    #[account(mut)]
    pub master_chef: AccountLoader<'info, MasterChef>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub reward_mint: Account<'info, Mint>,

    /// CHECK: Token mint/authority are checked at transfer
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = reward_mint,
        associated_token::authority = user,
    )]
    pub user_reward_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            user.key().as_ref(),
            lp_token.as_ref(),
            master_chef.key().as_ref(),
        ],
        bump,
    )]
    pub user_info: Account<'info, UserInfo>,

    /// CHECK: Seed constraint check
    #[
        account(
            mut,
            seeds = [
                REWARD_TOKEN_VAULT_SEED.as_bytes(),
                lp_token.as_ref(),
                master_chef.key().as_ref(),
            ],
            bump = master_chef.load_mut()?.find_pool(&lp_token)?.reward_token_vault_bump,
        )
    ]
    pub reward_token_vault: AccountInfo<'info>,

    /// CHECK: Seed constraint check
    #[
        account(
            mut,
            seeds = [
                REWARD_TOKEN_VAULT_AUTHORITY_SEED.as_bytes(),
                lp_token.as_ref(),
                master_chef.key().as_ref(),
            ],
            bump = master_chef.load_mut()?.find_pool(&lp_token)?.reward_token_vault_authority_bump,
        )
    ]
    pub reward_token_vault_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn claim_reward(ctx: Context<ClaimReward>, lp_token: Pubkey) -> Result<()> {

    let ClaimReward {
        master_chef: master_chef_loader,
        user,
        user_info,
        user_reward_token_account,
        reward_token_vault,
        reward_token_vault_authority,
        token_program,
        ..
    } = ctx.accounts;

    let mut master_chef = master_chef_loader.load_mut()?;
    let pool = master_chef.find_pool(&lp_token)?;

    if user_info.amount == 0 && <WrappedI80F48 as Into<I80F48>>::into(user_info.accrued_reward).is_zero() {
        return  Ok(());
    }

    pool.update_pool()?;

    let pending = I80F48::from_num(user_info.amount)
        .checked_mul(pool.acc_reward_per_share
        .into())
        .ok_or_else(math_error!())?
        .checked_sub(user_info.reward_debt.into())
        .ok_or_else(math_error!())?;

    user_info.reward_debt = I80F48::from_num(user_info.amount).checked_mul(pool.acc_reward_per_share.into()).ok_or_else(math_error!())?.into();
    let reward_amount: u64 = pending.checked_add(user_info.accrued_reward.into()).ok_or_else(math_error!())?.to_num();
    user_info.accrued_reward = I80F48::ZERO.into();
    pool.withdraw_sql_transfer(
        reward_amount, 
        Transfer {
            from: reward_token_vault.to_account_info(),
            to: user_reward_token_account.to_account_info(),
            authority: reward_token_vault_authority.to_account_info(),
    }, 
    token_program.to_account_info(), 
        pool_signer!(PoolVaultType::RewardTokenVault, lp_token, pool.reward_token_vault_authority_bump, master_chef_loader.key())
    )?;

    emit!(ClaimRewardEvent {
        header: EventHeader {
            master_chef: master_chef_loader.key(),
            signer: Some(user.key())
        },
        lp_token,
        amount: reward_amount,
    });

    Ok(())
}