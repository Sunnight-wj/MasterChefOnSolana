use anchor_lang::prelude::*;

use crate::{events::{EventHeader, UpadteRewardPerSlotEvent}, state::*};

#[derive(Accounts)]
pub struct UpadteRewardPerSlot<'info> {

    #[account(
        mut,
        address = master_chef.load()?.admin,
    )]
    pub admin: Signer<'info>,

    #[
        account(mut)
    ]
    pub master_chef: AccountLoader<'info, MasterChef>,
}

pub fn update_reward_per_slot(ctx: Context<UpadteRewardPerSlot>, lp_token: Pubkey, new_reward_per_slot: u64) -> Result<()> {

    let mut master_chef = ctx.accounts.master_chef.load_mut()?;
    let pool = master_chef.find_pool(&lp_token)?;
    let old_reward_per_slot = pool.reward_per_slot;
    pool.update_pool()?;
    pool.reward_per_slot = new_reward_per_slot;

    emit!(UpadteRewardPerSlotEvent {
        header: EventHeader {
            master_chef: ctx.accounts.master_chef.key(),
            signer: Some(ctx.accounts.admin.key())
        },
        lp_token,
        old_reward_per_slot,
        new_reward_per_slot,
    });
    Ok(())
}