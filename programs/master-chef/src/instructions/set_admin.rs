use anchor_lang::prelude::*;

use crate::{events::{EventHeader, SetAdminEvent}, state::{MasterChef, MasterChefConfig}};

#[derive(Accounts)]
pub struct SetAdmin<'info> {

    #[account(
        address = master_chef.load()?.admin,
    )]
    admin: Signer<'info>,

    #[account(mut)]
    pub master_chef: AccountLoader<'info, MasterChef>,
}

pub fn set_admin(ctx: Context<SetAdmin>, config: MasterChefConfig) -> Result<()> {
    let master_chef = &mut ctx.accounts.master_chef.load_mut()?;
    master_chef.configure(&config)?;

    emit!(SetAdminEvent {
        header: EventHeader {
            master_chef: ctx.accounts.master_chef.key(),
            signer: Some(ctx.accounts.admin.key()),
        },
        config,
    });

    Ok(())
}