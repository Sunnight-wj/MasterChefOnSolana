use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Transfer};
use fixed::types::I80F48;

use crate::{constants::*, events::{DepositEvent, EventHeader}, math_error, MasterChef, UserInfo};

#[derive(Accounts)]
#[instruction(lp_token: Pubkey)]
pub struct Deposit<'info> {

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub master_chef: AccountLoader<'info, MasterChef>,

    /// CHECK: Token mint/authority are checked at transfer
    #[account(mut)]
    pub user_lp_token_account: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<MasterChef>(),
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

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}


pub fn deposit(ctx: Context<Deposit>, lp_token: Pubkey, amount: u64) -> Result<()> {
    let Deposit {
        master_chef: master_chef_loader,
        user,
        user_lp_token_account,
        lp_token_vault,
        token_program,
        user_info,
        ..
    } = ctx.accounts;


    let mut master_chef = master_chef_loader.load_mut()?;
    let pool = master_chef.find_pool(&lp_token)?;

    pool.update_pool()?;

    if user_info.amount > 0 {
        let pending = I80F48::from_num(user_info.amount)
            .checked_mul(pool.acc_reward_per_share
            .into())
            .ok_or_else(math_error!())?
            .checked_sub(user_info.reward_debt.into())
            .ok_or_else(math_error!())?;
        user_info.accrued_reward = pending.checked_add(user_info.accrued_reward.into()).ok_or_else(math_error!())?.into();
    }

    if amount > 0 {
        pool.deposit_spl_transfer(
            amount, 
            Transfer {
                from: user_lp_token_account.to_account_info(),
                to: lp_token_vault.to_account_info(),
                authority: user.to_account_info(),
            }, 
            token_program.to_account_info(),
        )?;
        user_info.amount += amount;
        pool.lp_supply += amount;
    }
    user_info.reward_debt = I80F48::from_num(user_info.amount).checked_mul(pool.acc_reward_per_share.into()).ok_or_else(math_error!())?.into();
    emit!(DepositEvent {
        header: EventHeader {
            master_chef: master_chef_loader.key(),
            signer: Some(user.key()),
        },
        lp_token,
        amount,
    });
    Ok(())
}