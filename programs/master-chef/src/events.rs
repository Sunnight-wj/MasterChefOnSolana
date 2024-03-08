use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

use crate::state::MasterChefConfig;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct EventHeader {
    pub master_chef: Pubkey,
    pub signer: Option<Pubkey>,
}

#[event]
pub struct MasterChefInitializeEvent {
    pub header: EventHeader
}

#[event]
pub struct SetAdminEvent {
    pub header: EventHeader,
    pub config: MasterChefConfig,
}

#[event]
pub struct AddPoolEvent {
    pub header: EventHeader,
    pub reward_token: Pubkey,
    pub lp_token: Pubkey,
    pub start_slot: u64,
    pub reward_per_slot: u64,
}

#[event]
pub struct UpadteRewardPerSlotEvent {
    pub header: EventHeader,
    pub lp_token: Pubkey,
    pub old_reward_per_slot: u64,
    pub new_reward_per_slot: u64,
}

#[event]
pub struct DepositEvent {
    pub header: EventHeader,
    pub lp_token: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WithdrawEvent {
    pub header: EventHeader,
    pub lp_token: Pubkey,
    pub amount: u64,
}

#[event]
pub struct ClaimRewardEvent {
    pub header: EventHeader,
    pub lp_token: Pubkey,
    pub amount: u64,
}