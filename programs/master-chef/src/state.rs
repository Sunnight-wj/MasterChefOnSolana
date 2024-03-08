use anchor_lang::prelude::*;
use fixed::types::I80F48;
use std::fmt::{Debug, Formatter};
use crate::constants::*;
use crate::{check, math_error, set_if_some};
use crate::errors::MasterChefError;
use solana_program::clock::Clock;
use anchor_spl::token::{transfer, Transfer};




#[account]
#[derive(Default)]
pub struct UserInfo {
    pub amount: u64,
    pub reward_debt: WrappedI80F48,
    pub accrued_reward: WrappedI80F48,
}

#[zero_copy(unsafe)]
#[repr(C)]
#[derive(Default)]
pub struct PoolInfo {
    pub reward_token: Pubkey,
    pub lp_token: Pubkey,
    pub lp_supply: u64,
    pub start_slot: u64,
    pub reward_per_slot: u64,
    pub last_reward_slot: u64,
    pub acc_reward_per_share: WrappedI80F48,
    // bool值放第一位时，客户端fetch数据时报错：invalid bool？
    pub initialized: bool,

    pub lp_token_vault: Pubkey,
    pub lp_token_vault_bump: u8,
    pub lp_token_vault_authority_bump: u8,

    pub reward_token_vault: Pubkey,
    pub reward_token_vault_bump: u8,
    pub reward_token_vault_authority_bump: u8,
}

impl PoolInfo {

    pub fn update_pool(&mut self) -> Result<()> {
        let current_slot = Clock::get()?.slot;
        if current_slot <= self.last_reward_slot {
            return Ok(())
        }
        if self.lp_supply == 0 {
            self.last_reward_slot = current_slot;
            return Ok(());
        }
        let slot_delta = current_slot - self.last_reward_slot;
        let reward_amount = slot_delta.checked_mul(self.reward_per_slot).ok_or_else(math_error!())?;
        self.acc_reward_per_share = I80F48::from_num(reward_amount)
            .checked_div(I80F48::from_num(self.lp_supply))
            .ok_or_else(math_error!())?
            .into();
        self.last_reward_slot = current_slot;
        Ok(())
    }

    pub fn deposit_spl_transfer<'b: 'c, 'c: 'b>(
        &self,
        amount: u64,
        accounts: Transfer<'b>,
        program: AccountInfo<'c>,
    ) -> Result<()> {
        check!(
            accounts.to.key.eq(&self.lp_token_vault),
            MasterChefError::InvalidTransfer
        );

        msg!(
            "deposit_spl_transfer: amount: {} from {} to {}, auth {}",
            amount,
            accounts.from.key,
            accounts.to.key,
            accounts.authority.key
        );

        transfer(CpiContext::new(program, accounts), amount)
    }

    pub fn withdraw_sql_transfer<'b: 'c, 'c: 'b>(
        &self,
        amount: u64,
        accounts: Transfer<'b>,
        program: AccountInfo<'c>,
        signer_seeds: &[&[&[u8]]],
    ) -> Result<()> {
        msg!(
            "withdraw_spl_transfer: amount: {} from {} to {}, auth {}",
            amount,
            accounts.from.key,
            accounts.to.key,
            accounts.authority.key
        );

        transfer(
            CpiContext::new_with_signer(program, accounts, signer_seeds),
            amount,
        )
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Debug, Clone)]
pub struct PoolConfig {
    reward_token: Option<Pubkey>,
    lp_token: Option<Pubkey>,
    start_slot: Option<Pubkey>,
    reward_per_slot: Option<Pubkey>,
}

const MAX_POOLS: usize = 8;

#[account(zero_copy(unsafe))]
#[repr(C)]
pub struct MasterChef {
    pub admin: Pubkey,
    pub pools: [PoolInfo; MAX_POOLS],
}

impl MasterChef {
    
    pub fn configure(&mut self, config: &MasterChefConfig) -> Result<()> {
        set_if_some!(self.admin, config.admin);
        Ok(())
    }

    pub fn set_initial_configuration(&mut self, admin_pk: Pubkey) {
        self.admin = admin_pk;
    }

    pub fn get_first_empty_pool(&self) -> Option<usize> {
        self.pools.iter().position(|p| !p.initialized)
    }

    pub fn find_pool(&mut self, lp_token: &Pubkey) -> Result<&mut PoolInfo> {
        let pool = self.pools
        .iter_mut()
        .find(|pool| pool.initialized && pool.lp_token.eq(lp_token))
        .ok_or_else(|| MasterChefError::PoolNotFind)?;

        Ok(pool)
    }

    pub fn create_pool(
        &mut self, 
        reward_token: Pubkey, 
        lp_token: Pubkey, 
        start_slot: u64, 
        reward_per_slot: u64, 
        lp_token_vault: Pubkey, 
        lp_token_vault_bump: u8,
        lp_token_vault_authority_bump: u8,
        reward_token_vault: Pubkey,
        reward_token_vault_bump: u8,
        reward_token_vault_authority_bump: u8,
    ) -> Result<&mut PoolInfo> {
        if self.pools
        .iter()
        .find(|pool| pool.initialized && pool.lp_token.eq(&lp_token))
        .is_some() {
            Err(error!(MasterChefError::PoolAlreadyExists))?
        }
        let empty_index = self.get_first_empty_pool().ok_or_else(|| error!(MasterChefError::PoolSlotsFull))?;
        let current_slot = Clock::get()?.slot;
        self.pools[empty_index] = PoolInfo {
            initialized: true,
            reward_token,
            lp_token,
            lp_supply: 0,
            start_slot,
            reward_per_slot,
            last_reward_slot: if start_slot < current_slot { current_slot } else { start_slot } ,
            acc_reward_per_share: WrappedI80F48 { value: 0},
            lp_token_vault,
            lp_token_vault_bump,
            lp_token_vault_authority_bump,
            reward_token_vault,
            reward_token_vault_bump,
            reward_token_vault_authority_bump,
        };
        Ok(&mut self.pools[empty_index])
    }

}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Debug, Clone)]
pub struct MasterChefConfig {
    pub admin: Option<Pubkey>,
}

#[zero_copy]
#[repr(C)]
#[derive(Default, AnchorDeserialize, AnchorSerialize)]
pub struct  WrappedI80F48 {
    pub value: i128,
}

impl Debug for WrappedI80F48 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", I80F48::from_bits(self.value))
    }
}

impl From<I80F48> for WrappedI80F48 {
    fn from(i: I80F48) -> Self {
        Self { value: i.to_bits()}
    }
}

impl From<WrappedI80F48> for I80F48 {
    fn from(w: WrappedI80F48) -> Self {
        Self::from_bits(w.value)
    }
}

#[derive(Debug, Clone)]
pub enum PoolVaultType {
    LPTokenVault,
    RewardTokenVault,
}

impl PoolVaultType {
    pub fn get_seed(self) -> &'static [u8] {
        match self {
            PoolVaultType::LPTokenVault => LP_TOKEN_VAULT_SEED.as_bytes(),
            PoolVaultType::RewardTokenVault => REWARD_TOKEN_VAULT_SEED.as_bytes(),
        }
    }

    pub fn get_authority_seed(self) -> &'static [u8] {
        match self {
            PoolVaultType::LPTokenVault => LP_TOKEN_VAULT_AUTHORITY_SEED.as_bytes(),
            PoolVaultType::RewardTokenVault => REWARD_TOKEN_VAULT_AUTHORITY_SEED.as_bytes(),
        }
    }
}
