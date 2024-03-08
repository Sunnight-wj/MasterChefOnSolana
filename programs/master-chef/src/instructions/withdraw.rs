use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Transfer};
use fixed::types::I80F48;

use crate::{constants::*, check, constants::LP_TOKEN_VAULT_AUTHORITY_SEED, errors::*, events::{EventHeader, WithdrawEvent}, math_error, pool_signer, MasterChef, PoolVaultType, UserInfo};

#[derive(Accounts)]
#[instruction(lp_token: Pubkey)]
pub struct Withdraw<'info> {

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub master_chef: AccountLoader<'info, MasterChef>,

    /// CHECK: Token mint/authority are checked at transfer
    #[account(mut)]
    pub user_lp_token_account: AccountInfo<'info>,

    #[account(
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
                LP_TOKEN_VAULT_SEED.as_bytes(),
                lp_token.key().as_ref(),
                master_chef.key().as_ref(),
            ],
            bump = master_chef.load_mut()?.find_pool(&lp_token)?.lp_token_vault_bump,
        )
    ]
    pub lp_token_vault: AccountInfo<'info>,

    /// CHECK: Seed constraint check
    #[
        account(
            mut,
            seeds = [
                LP_TOKEN_VAULT_AUTHORITY_SEED.as_bytes(),
                lp_token.as_ref(),
                master_chef.key().as_ref(),
            ],
            bump = master_chef.load_mut()?.find_pool(&lp_token)?.lp_token_vault_authority_bump,
        )
    ]
    pub lp_token_vault_authority: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn withdraw(ctx: Context<Withdraw>, lp_token: Pubkey, amount: u64) -> Result<()> {
    let Withdraw {
        user,
        master_chef: master_chef_loader,
        user_lp_token_account,
        user_info,
        lp_token_vault,
        lp_token_vault_authority,
        token_program,
        ..
    }
    = ctx.accounts;

    let mut master_chef = master_chef_loader.load_mut()?;
    let pool = master_chef.find_pool(&lp_token)?;

    check!(
        user_info.amount >= amount,
        MasterChefError::LPTokenNotEnough
    );
    pool.update_pool()?;

    let pending = I80F48::from_num(user_info.amount)
        .checked_mul(pool.acc_reward_per_share
        .into())
        .ok_or_else(math_error!())?
        .checked_sub(user_info.reward_debt.into())
        .ok_or_else(math_error!())?;
    user_info.accrued_reward = pending.checked_add(user_info.accrued_reward.into()).ok_or_else(math_error!())?.into();
    
    if amount > 0 {
        pool.withdraw_sql_transfer(
            amount, 
            Transfer {
                from: lp_token_vault.to_account_info(),
                to: user_lp_token_account.to_account_info(),
                authority: lp_token_vault_authority.to_account_info(),
            }, 
            token_program.to_account_info(),
            pool_signer!(PoolVaultType::LPTokenVault, lp_token, pool.lp_token_vault_authority_bump, master_chef_loader.key())
        )?;
        user_info.amount -= amount;
        pool.lp_supply -= amount;
    }

    user_info.reward_debt = I80F48::from_num(user_info.amount).checked_mul(pool.acc_reward_per_share.into()).ok_or_else(math_error!())?.into();

    emit!(WithdrawEvent {
        header: EventHeader {
            master_chef: master_chef_loader.key(),
            signer: Some(user.key()),
        },
        lp_token,
        amount,
    });

    Ok(())
}