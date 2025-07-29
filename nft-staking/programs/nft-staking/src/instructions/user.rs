use crate::state::UserAccount;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct User<'info> {
    #[account[mut]]
    pub user: Signer<'info>,
    #[account[
      init,
      payer=user,
      seeds=[b"user", user.key().as_ref()],
      bump,
      space = 8 + UserAccount::INIT_SPACE
    ]]
    pub user_account: Account<'info, UserAccount>,
    pub system_program: Program<'info, System>,
}

impl<'info> User<'info> {
    pub fn create_user_account(&mut self, bumps: UserBumps) -> Result<()> {
        self.user_account.set_inner(UserAccount {
            points: 0,
            amount_staked: 0,
            bump: bumps.user_account,
        });
        Ok(())
    }
}

// pub fn handler(ctx: Context<Config>) -> Result<()> {
//     // msg!("Greetings from: {{:?}}", ctx.program_id);
//     Ok(())
// }
