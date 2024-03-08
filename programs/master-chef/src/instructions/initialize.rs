use anchor_lang::prelude::*;
use crate::{events::*, state::MasterChef};

#[derive(Accounts)]
pub struct Initialize<'info> {

    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + std::mem::size_of::<MasterChef>(),
    )]
    pub master_chef: AccountLoader<'info, MasterChef>,

    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let master_chef = &mut ctx.accounts.master_chef.load_init()?;
    master_chef.set_initial_configuration(ctx.accounts.admin.key());

    emit!(MasterChefInitializeEvent {
        header: EventHeader {
            master_chef: ctx.accounts.master_chef.key(),
            signer: Some(ctx.accounts.admin.key()),
        }
    });
    Ok(())
}