use anchor_lang::prelude::*;

declare_id!("Pm6nvqg8yuykRXCRJUT6yp1q2Y6hqcB7bzZFujLoLWm");

#[program]
pub mod anvil {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
