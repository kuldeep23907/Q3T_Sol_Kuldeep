use anchor_lang::Discriminator;
use solana_gpt_oracle::{ContextAccount, Counter, Identity};
use {
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        metadata::{
            create_metadata_accounts_v3, mpl_token_metadata::types::DataV2,
            update_metadata_accounts_v2, CreateMetadataAccountsV3, Metadata,
            UpdateMetadataAccountsV2,
        },
        token::spl_token::instruction::AuthorityType,
        token::{mint_to, set_authority, Mint, MintTo, SetAuthority, Token, TokenAccount},
    },
};

declare_id!("6gXrJnrnoEoUEPsNzgQFuaBJ9tTZAytxNVp5poSAraie");

#[program]
pub mod agent_minter {
    use super::*;

    const AGENT_DESC: &str =
        "You are an AI agent called NFT-Machine which can create NFT token. \
        Users will provide you with 5 words separated by comma to create an NFT image for them. \
        Always create a simple image, black and white, clear. \
        After creating the image, upload the image on IPFS or similar, create a metadata json file, \
        upload that on IPFS as well and provide a metadata URI link. \
        IMPORTANT: always reply in a valid json format. No character before or after. The format is:/\
         {\"uri\": \"your reply\" }, \
        where uri is the URI for the metadata \
        Good luck.";

    const TOKEN_URI: &str =
        "https://shdw-drive.genesysgo.net/4PMP1MG5vYGkT7gnAMb7E5kqPLLjjDzTiAaZ3xRx5Czd/mar1o.json";

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.agent.context = ctx.accounts.llm_context.key();

        // Create the context for the AI agent
        let cpi_program = ctx.accounts.oracle_program.to_account_info();
        let cpi_accounts = solana_gpt_oracle::cpi::accounts::CreateLlmContext {
            payer: ctx.accounts.payer.to_account_info(),
            context_account: ctx.accounts.llm_context.to_account_info(),
            counter: ctx.accounts.counter.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        solana_gpt_oracle::cpi::create_llm_context(cpi_ctx, AGENT_DESC.to_string())?;
        Ok(())
    }

    pub fn interact_agent(
        ctx: Context<InteractAgent>,
        name: String,
        symbol: String,
        text: String,
    ) -> Result<()> {
        validate_text(&text)?;
        // Initialize the agent token
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"mint",
            // ctx.accounts.payer.key().as_ref(),
            &[ctx.bumps.mint_account],
        ]];

        // CPI signed by PDA
        create_metadata_accounts_v3(
            CpiContext::new(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata_account.to_account_info(),
                    mint: ctx.accounts.mint_account.to_account_info(),
                    mint_authority: ctx.accounts.mint_account.to_account_info(), // PDA is mint authority
                    update_authority: ctx.accounts.mint_account.to_account_info(), // PDA is update authority
                    payer: ctx.accounts.payer.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            DataV2 {
                name,
                symbol,
                uri: TOKEN_URI.to_string(), // default URI
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            true, // Is mutable
            true, // Update authority is signer
            None,
        )?;

        let cpi_program = ctx.accounts.oracle_program.to_account_info();
        let cpi_accounts = solana_gpt_oracle::cpi::accounts::InteractWithLlm {
            payer: ctx.accounts.payer.to_account_info(),
            interaction: ctx.accounts.interaction.to_account_info(),
            context_account: ctx.accounts.context_account.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        let disc: [u8; 8] = instruction::CallbackFromAgent::DISCRIMINATOR
            .try_into()
            .expect("Discriminator must be 8 bytes");
        solana_gpt_oracle::cpi::interact_with_llm(
            cpi_ctx,
            text,
            ID,
            disc,
            Some(vec![
                solana_gpt_oracle::AccountMeta {
                    pubkey: ctx.accounts.payer.to_account_info().key(),
                    is_signer: false,
                    is_writable: false,
                },
                solana_gpt_oracle::AccountMeta {
                    pubkey: ctx.accounts.mint_account.to_account_info().key(),
                    is_signer: false,
                    is_writable: true,
                },
                solana_gpt_oracle::AccountMeta {
                    pubkey: ctx
                        .accounts
                        .associated_token_account
                        .to_account_info()
                        .key(),
                    is_signer: false,
                    is_writable: true,
                },
                solana_gpt_oracle::AccountMeta {
                    pubkey: ctx.accounts.token_program.to_account_info().key(),
                    is_signer: false,
                    is_writable: false,
                },
                solana_gpt_oracle::AccountMeta {
                    pubkey: ctx.accounts.system_program.to_account_info().key(),
                    is_signer: false,
                    is_writable: false,
                },
            ]),
        )?;

        Ok(())
    }

    pub fn callback_from_agent(ctx: Context<CallbackFromAgent>, response: String) -> Result<()> {
        // Check if the callback is from the LLM program
        if !ctx.accounts.identity.to_account_info().is_signer {
            return Err(ProgramError::InvalidAccountData.into());
        }

        // Parse the JSON response
        let response: String = response
            .trim()
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .to_string();
        let parsed: serde_json::Value =
            serde_json::from_str(&response).unwrap_or_else(|_| serde_json::json!({}));

        // Extract the reply and amount
        let uri = parsed["uri"].as_str().unwrap_or("NO URI");

        msg!("new uri from ai {}", uri);

        // Mint the agent token to the payer
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"mint",
            // ctx.accounts.user.key().as_ref(),
            &[ctx.bumps.mint_account],
        ]];

        // Invoke the mint_to instruction on the token program
        mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint_account.to_account_info(),
                    to: ctx.accounts.associated_token_account.to_account_info(),
                    authority: ctx.accounts.mint_account.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            1,
        )?;

        //
        // 4️⃣ Revoke mint authority (makes it immutable / NFT)
        //
        set_authority(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                SetAuthority {
                    current_authority: ctx.accounts.mint_account.to_account_info(),
                    account_or_mint: ctx.accounts.mint_account.to_account_info(),
                },
                signer_seeds,
            ),
            AuthorityType::MintTokens,
            None, // revoke
        )?;

        // update uri
        let ctx_program = ctx.accounts.token_metadata_program.to_account_info();
        let ctx_accounts = UpdateMetadataAccountsV2 {
            metadata: ctx.accounts.metadata_account.to_account_info(),
            update_authority: ctx.accounts.mint_account.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(ctx_program, ctx_accounts, signer_seeds);

        let data = Some(DataV2 {
            name: "".to_string(),
            symbol: "".to_string(),
            uri: uri.to_string(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        });

        update_metadata_accounts_v2(ctx, None, data, None, Some(false))?;

        Ok(())
    }
}

pub fn validate_text(text: &String) -> Result<()> {
    // Split by commas
    let parts: Vec<&str> = text
        .split(',')
        .map(|s| s.trim()) // trim spaces around words
        .filter(|s| !s.is_empty()) // ignore accidental empty parts
        .collect();

    // Ensure exactly 5 words
    require!(parts.len() == 5, CustomError::InvalidWordCount);

    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + 32,
        seeds = [b"agent"],
        bump
    )]
    pub agent: Account<'info, Agent>,
    /// CHECK: Checked in oracle program
    #[account(mut)]
    pub llm_context: AccountInfo<'info>,
    /// CHECK: Checked in oracle program
    #[account(mut)]
    pub counter: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: Checked oracle id
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(text: String)]
pub struct InteractAgent<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: Checked in oracle program
    #[account(mut)]
    pub interaction: AccountInfo<'info>,
    #[account(seeds = [b"agent"], bump)]
    pub agent: Account<'info, Agent>,
    #[account(address = agent.context)]
    pub context_account: Account<'info, ContextAccount>,
    // Create mint account: uses Same PDA as address of the account and mint/freeze authority
    #[account(
        init,
        seeds = [b"mint"],
        bump,
        payer = payer,
        mint::decimals = 0,
        mint::authority = mint_account.key(),
        mint::freeze_authority = mint_account.key(),

    )]
    pub mint_account: Account<'info, Mint>,
    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), mint_account.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata_account: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_account,
        associated_token::authority = payer,
    )]
    pub associated_token_account: Account<'info, TokenAccount>,
    // #[account(
    //     mut,
    //     seeds = [b"mint"],
    //     bump
    // )]
    // pub mint_account: Account<'info, Mint>,
    /// CHECK: Checked oracle id
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CallbackFromAgent<'info> {
    /// CHECK: Checked in oracle program
    pub identity: Account<'info, Identity>,
    /// CHECK: The user wo did the interaction
    pub user: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [b"mint"],
        bump
    )]
    pub mint_account: Account<'info, Mint>,
    /// CHECK: Validate address by deriving pda
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), mint_account.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata_account: UncheckedAccount<'info>,
    #[account(
        mut,
        associated_token::mint = mint_account,
        associated_token::authority = user,
    )]
    pub associated_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metadata>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct Agent {
    pub context: Pubkey,
}

#[error_code]
pub enum CustomError {
    #[msg("Text must contain exactly 5 comma-separated words.")]
    InvalidWordCount,
}
