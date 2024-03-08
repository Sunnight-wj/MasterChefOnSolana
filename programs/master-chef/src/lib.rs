pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod macros;
pub mod state;
pub mod utils;


use anchor_lang::prelude::*;
use instructions::*;
use state::*;


declare_id!("24Ri2mS76yjtwPw41RcBRv1AAY6Zkt9cUzMgkjrRoTc3");

#[program]
pub mod master_chef {
    use self::state::MasterChefConfig;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)
    }

    pub fn set_admin(ctx: Context<SetAdmin>, config: MasterChefConfig) -> Result<()> {
        instructions::set_admin(ctx, config)
    }

    pub fn add_pool(
        ctx: Context<AddPool>, 
        reward_token: Pubkey,
        lp_token: Pubkey,
        start_slot: u64,
        reward_per_slot: u64,
    ) -> Result<()> {
        instructions::add_pool(
            ctx, 
            reward_token, 
            lp_token, 
            reward_per_slot, 
            start_slot, 
        )
    }

    pub fn update_reward_per_slot(
        ctx: Context<UpadteRewardPerSlot>,
        lp_token: Pubkey,
        new_reward_per_slot: u64
    ) -> Result<()> {
        instructions::update_reward_per_slot(ctx, lp_token, new_reward_per_slot)
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        lp_token: Pubkey,
        amount: u64,
    ) -> Result<()> {
        instructions::deposit(ctx, lp_token, amount)
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        lp_token: Pubkey,
        amount: u64
    ) -> Result<()> {
        instructions::withdraw(ctx, lp_token, amount)
    }

    pub fn claim_reward(ctx: Context<ClaimReward>, lp_token: Pubkey) ->Result<()> {
        instructions::claim_reward(ctx, lp_token)
    }
}



