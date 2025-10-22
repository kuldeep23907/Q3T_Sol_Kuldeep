use anchor_lang::prelude::*;

use crate::state::UserAccount;

use ephemeral_rollups_sdk::{anchor::commit, ephem::commit_accounts};
use ephemeral_vrf_sdk::anchor::vrf;
use ephemeral_vrf_sdk::instructions::{create_request_randomness_ix, RequestRandomnessParams};
use ephemeral_vrf_sdk::types::SerializableAccountMeta;

#[vrf]
#[derive(Accounts)]
pub struct UpdateUserVrf<'info> {
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,
    /// CHECK: The oracle queue
    #[account(mut)]
    pub oracle_queue: AccountInfo<'info>,
}

impl<'info> UpdateUserVrf<'info> {
    pub fn update_user_vrf(&mut self, seeds: u8) -> Result<()> {
        // vrf call

        let ix = create_request_randomness_ix(RequestRandomnessParams {
            payer: self.user.key(),
            oracle_queue: self.oracle_queue.key(),
            callback_program_id: crate::ID,
            callback_discriminator: crate::instruction::CallbackUpdateUserAccount::DISCRIMINATOR
                .to_vec(),
            caller_seed: [seeds; 32],
            // Specify any account that is required by the callback
            accounts_metas: Some(vec![SerializableAccountMeta {
                pubkey: self.user_account.key(),
                is_signer: false,
                is_writable: true,
            }]),
            ..Default::default()
        });
        self.invoke_signed_vrf(&self.user.to_account_info(), &ix)?;
        Ok(())
    }

    pub fn update_user_vrf_on_er(&mut self, seeds: u8) -> Result<()> {
        // vrf call

        let ix = create_request_randomness_ix(RequestRandomnessParams {
            payer: self.user.key(),
            oracle_queue: self.oracle_queue.key(),
            callback_program_id: crate::ID,
            callback_discriminator:
                crate::instruction::CallbackUpdateUserAccountOnEr::DISCRIMINATOR.to_vec(),
            caller_seed: [seeds; 32],
            // Specify any account that is required by the callback
            accounts_metas: Some(vec![SerializableAccountMeta {
                pubkey: self.user_account.key(),
                is_signer: false,
                is_writable: true,
            }]),
            ..Default::default()
        });
        self.invoke_signed_vrf(&self.user.to_account_info(), &ix)?;
        Ok(())
    }
}
#[commit]
#[derive(Accounts)]
pub struct CallbackUpdateUserAccount<'info> {
    /// This check ensure that the vrf_program_identity (which is a PDA) is a singer
    /// enforcing the callback is executed by the VRF program trough CPI
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", user_account.user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,
}

impl<'info> CallbackUpdateUserAccount<'info> {
    // Consume Randomness
    pub fn callback_update_user_account(&mut self, randomness: [u8; 32]) -> Result<()> {
        let rnd_u8 = ephemeral_vrf_sdk::rnd::random_u8_with_range(&randomness, 1, 255);
        msg!("Consuming random number: {:?}", rnd_u8);
        let player = &mut self.user_account;
        player.data = rnd_u8 as u64;

        Ok(())
    }

    // Consume Randomness
    pub fn callback_update_user_account_on_er(&mut self, randomness: [u8; 32]) -> Result<()> {
        let rnd_u8 = ephemeral_vrf_sdk::rnd::random_u8_with_range(&randomness, 1, 255);
        msg!("Consuming random number: {:?}", rnd_u8);
        let player = &mut self.user_account;
        player.data = rnd_u8 as u64;

        commit_accounts(
            &self.vrf_program_identity.to_account_info(),
            vec![&self.user_account.to_account_info()],
            &self.magic_context,
            &self.magic_program,
        )?;

        Ok(())
    }
}
