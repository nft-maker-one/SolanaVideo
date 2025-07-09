use anchor_lang::prelude::*;

declare_id!("AmRzTv3uRJcHw87ym7bhWMrC6HAYuTPn5VddFLMTqiHt");


#[program]
pub mod counter {
    use super::*;
    pub fn initialize(ctx:Context<Initialize>,initial_value:u64) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.value = initial_value;
        counter.owner = ctx.accounts.user.key();
        Ok(())
    }

    pub fn increment(ctx:Context<UpdateCounter>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.value = counter.value.checked_add(1).ok_or(CounterError::Overflow)?;
        Ok(())
    }

    pub fn decrement(ctx:Context<UpdateCounter>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.value = counter.value.checked_sub(1).ok_or(CounterError::Underflow)?;
        Ok(())
    }


}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init,payer=user,space=8+8+32)]
    pub counter:Account<'info,Counter>,
    #[account(mut)]
    pub user:Signer<'info>,
    pub system_program:Program<'info,System>
}

#[derive(Accounts)]
pub struct UpdateCounter<'info> {
    #[account(mut)]
    pub counter:Account<'info,Counter>,
    pub owner:Signer<'info>
}

#[account]
pub struct Counter {
    pub value:u64,
    pub owner:Pubkey
}

#[error_code]
pub enum CounterError {
    #[msg("Overflowed")]
    Overflow,
    #[msg("Underflowed")]
    Underflow
}