#[cfg(test)]
mod tests {

    use {
        anchor_lang::{prelude::msg, AccountDeserialize, InstructionData, Key, ToAccountMetas},
        anchor_spl::token_2022::spl_token_2022::state::Account,
        anchor_spl::{
            associated_token::{
                self, get_associated_token_address_with_program_id, spl_associated_token_account,
            },
            token::spl_token,
            token_2022::{
                self, spl_token_2022::extension::StateWithExtensions,
                spl_token_2022::native_mint::ID, ID as token_2022_ID,
            }, // spl_token::{LAMPORTS_PER_SOL}
        },
        litesvm::LiteSVM,
        litesvm_token::{
            spl_token::ID as TOKEN_PROGRAM_ID, CreateAssociatedTokenAccount, CreateMint, MintTo,
            MintToChecked,
        },
        solana_address::Address,
        solana_instruction::Instruction,
        solana_keypair::Keypair,
        solana_message::Message,
        solana_pubkey::Pubkey,
        solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID,
        solana_signer::Signer,
        solana_transaction::Transaction,
        std::{path::PathBuf, str::FromStr},
        transfer_hook::accounts::{AddToWhiteList, InitializeExtraAccountMetaList}, // transfer_hook::initialize_extra_account_meta_list,
        transfer_hook::instruction::AddToWhitelist as AddWhitelist,
        transfer_hook::instruction::InitializeExtraAccountMetaList as InitExtraAccount,
    };

    static PROGRAM_ID: Pubkey = crate::ID;

    pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

    fn setup() -> (
        LiteSVM,
        Keypair, // admin keypair
        Pubkey,  // admin
        Keypair, // user keypair
        Pubkey,  // user
        Pubkey,  // user ata
        Pubkey,  // config
        Pubkey,  // vault
        Keypair, // mint keypair
        Pubkey,  // mint
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
    ) {
        let transfer_hook_program: Pubkey =
            Pubkey::from_str("CyusCQe7fFM4BxiqEbs5Wj7YroPuXSHtMCXy4NgZvT5v").unwrap();
        // Initialize LiteSVM and payer
        let mut program = LiteSVM::new();
        let admin_keypair = Keypair::new();
        let admin = admin_keypair.pubkey();
        let user_keypair = Keypair::new();
        let user = user_keypair.pubkey();
        let mint_keypair = Keypair::new();
        let mint_token = mint_keypair.pubkey();

        // Airdrop some SOL to the payer keypair
        program
            .airdrop(&admin, 1000 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to payer");

        // Airdrop some SOL to the payer keypair
        program
            .airdrop(&user, 1000 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to payer");

        // Load program SO file
        let so_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/vault.so");

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        program.add_program(PROGRAM_ID, &program_data);

        // Load program SO file
        let so_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/transfer_hook.so");

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        program.add_program(transfer_hook_program, &program_data);

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let config = Pubkey::find_program_address(&[b"config"], &PROGRAM_ID).0;

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let user_position =
            Pubkey::find_program_address(&[b"user", mint_token.key().as_ref()], &PROGRAM_ID).0;

        // Derive the PDA for the vault associated token account using the escrow PDA and Mint A
        let vault = associated_token::get_associated_token_address_with_program_id(
            &config,
            &mint_token,
            &token_2022_ID,
        );

        let user_ata = associated_token::get_associated_token_address_with_program_id(
            &user,
            &mint_token,
            &token_2022_ID,
        );

        let extra_account_metalist = Pubkey::find_program_address(
            &[b"extra-account-metas", mint_token.key().as_ref()],
            &transfer_hook_program,
        )
        .0;

        let whitelist = Pubkey::find_program_address(&[b"whitelist"], &transfer_hook_program).0;

        let associated_token_program = spl_associated_token_account::ID;
        let token_program = token_2022_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        // Return the LiteSVM instance and payer keypair
        (
            program,
            admin_keypair,
            admin,
            user_keypair,
            user,
            user_ata,
            config,
            vault,
            mint_keypair,
            mint_token,
            user_position,
            associated_token_program,
            token_program,
            system_program,
            transfer_hook_program,
            extra_account_metalist,
            whitelist,
        )
    }

    #[test]
    fn test_init_vault() {
        let (
            mut program,
            admin_keypair,
            admin,
            user_keypair,
            user,
            user_ata,
            config,
            vault,
            mint_keypair,
            mint_token,
            user_position,
            associated_token_program,
            token_program,
            system_program,
            hook_program,
            extra_account_metalist,
            whitelist,
        ) = setup();

        msg!("admin {}\n", admin);
        msg!("user {}\n", user);
        msg!("user ara {}\n", user_ata);
        msg!("config {}\n", config);
        msg!("vault {}\n", vault);
        msg!("mint token {}\n", mint_token);

        let init_vault_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::InitVault {
                admin: admin,
                mint: mint_token,
                config: config,
                vault: vault,
                hook_program: hook_program,
                associated_token_program: associated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::InitVault {}.data(),
        };

        let message = Message::new(&[init_vault_ix], Some(&admin));
        let recent_blockhash = program.latest_blockhash();

        let transaction =
            Transaction::new(&[&admin_keypair, &mint_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nInit vauult sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        let config_account = program.get_account(&config).unwrap();
        let config_data =
            crate::state::Config::try_deserialize(&mut config_account.data.as_ref()).unwrap();
        assert_eq!(config_data.admin, admin);
        assert_eq!(config_data.mint, mint_token);
        assert_eq!(config_data.vault, vault);

        let init_extra_account_metalist_ix = Instruction {
            program_id: hook_program,
            accounts: InitializeExtraAccountMetaList {
                payer: admin,
                mint: mint_token,
                extra_account_meta_list: extra_account_metalist,
                whitelist: whitelist,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: InitExtraAccount {}.data(),
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[init_extra_account_metalist_ix], Some(&admin));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&admin_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();
    }

    #[test]
    fn test_mint_token() {
        let (
            mut program,
            admin_keypair,
            admin,
            user_keypair,
            user,
            user_ata,
            config,
            vault,
            mint_keypair,
            mint_token,
            user_positions,
            associated_token_program,
            token_program,
            system_program,
            hook_program,
            extra_account_metalist,
            whitelist,
        ) = setup();

        let init_vault_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::InitVault {
                admin: admin,
                mint: mint_token,
                config: config,
                vault: vault,
                associated_token_program: associated_token_program,
                token_program: token_program,
                system_program: system_program,
                hook_program: hook_program,
            }
            .to_account_metas(None),
            data: crate::instruction::InitVault {}.data(),
        };

        let message = Message::new(&[init_vault_ix], Some(&admin));
        let recent_blockhash = program.latest_blockhash();

        let transaction =
            Transaction::new(&[&admin_keypair, &mint_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx1 = program.send_transaction(transaction).unwrap();

        let init_extra_account_metalist_ix = Instruction {
            program_id: hook_program,
            accounts: InitializeExtraAccountMetaList {
                payer: admin,
                mint: mint_token,
                extra_account_meta_list: extra_account_metalist,
                whitelist: whitelist,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: InitExtraAccount {}.data(),
        };

        let message = Message::new(&[init_extra_account_metalist_ix], Some(&admin));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&admin_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx2 = program.send_transaction(transaction).unwrap();

        // MintTo::new(
        //     &mut program,
        //     &admin_keypair,
        //     &mint_token,
        //     &user_ata,
        //     10_000_000,
        // )
        // .token_program_id(&token_2022_ID)
        // .send()
        // .unwrap();

        let mint_token_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::MintToken {
                admin: admin,
                user: user,
                mint: mint_token,
                user_ata: user_ata,
                config: config,
                associated_token_program: associated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::MintToken {}.data(),
        };

        let message = Message::new(&[mint_token_ix], Some(&admin));
        let recent_blockhash = program.latest_blockhash();
        let transaction = Transaction::new(&[&admin_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        MintToChecked::new(
            &mut program,
            &admin_keypair,
            &mint_token,
            &user_ata,
            10_000_000,
        )
        .token_program_id(&token_2022_ID)
        .decimals(6)
        .owner(&admin_keypair)
        // .signers(&[&admin.pubkey()])
        .send()
        .unwrap();

        // Log transaction details
        msg!("\n\nMint token sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        let user_ata_account = program.get_account(&user_ata).unwrap();
        let user_ata_data: StateWithExtensions<Account> =
            StateWithExtensions::<Account>::unpack(&user_ata_account.data).unwrap();
        assert_eq!(user_ata_data.base.amount, 1010_000_000);
    }

    #[test]
    fn test_deposit_token() {
        let (
            mut program,
            admin_keypair,
            admin,
            user_keypair,
            user,
            user_ata,
            config,
            vault,
            mint_keypair,
            mint_token,
            user_position,
            associated_token_program,
            token_program,
            system_program,
            hook_program,
            extra_account_metalist,
            whitelist,
        ) = setup();

        let init_vault_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::InitVault {
                admin: admin,
                mint: mint_token,
                config: config,
                vault: vault,
                associated_token_program: associated_token_program,
                token_program: token_program,
                system_program: system_program,
                hook_program: hook_program,
            }
            .to_account_metas(None),
            data: crate::instruction::InitVault {}.data(),
        };

        let message = Message::new(&[init_vault_ix], Some(&admin));
        let recent_blockhash = program.latest_blockhash();

        let transaction =
            Transaction::new(&[&admin_keypair, &mint_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx1 = program.send_transaction(transaction).unwrap();

        let init_extra_account_metalist_ix = Instruction {
            program_id: hook_program,
            accounts: InitializeExtraAccountMetaList {
                payer: admin,
                mint: mint_token,
                extra_account_meta_list: extra_account_metalist,
                whitelist: whitelist,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: InitExtraAccount {}.data(),
        };

        let message = Message::new(&[init_extra_account_metalist_ix], Some(&admin));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&admin_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx2 = program.send_transaction(transaction).unwrap();

        let mint_token_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::MintToken {
                admin: admin,
                user: user,
                mint: mint_token,
                user_ata: user_ata,
                config: config,
                associated_token_program: associated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::MintToken {}.data(),
        };

        let message = Message::new(&[mint_token_ix], Some(&admin));
        let recent_blockhash = program.latest_blockhash();
        let transaction = Transaction::new(&[&admin_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx3 = program.send_transaction(transaction).unwrap();

        let whitelist_tx = Instruction {
            program_id: hook_program,
            accounts: AddToWhiteList {
                signer: admin,
                mint: mint_token,
                user: user,
                whitelist: whitelist,
            }
            .to_account_metas(None),
            data: AddWhitelist {
                amount: 100_000_000,
            }
            .data(),
        };

        let message = Message::new(&[whitelist_tx], Some(&admin));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&admin_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx4 = program.send_transaction(transaction).unwrap();

        let deposit_token_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Trade {
                user: user,
                mint: mint_token,
                user_ata: user_ata,
                config: config,
                vault: vault,
                user_position: user_position,
                associated_token_program: associated_token_program,
                token_program: token_program,
                system_program: system_program,
                hook_program: hook_program,
                whitelist: whitelist,
                extra_account_meta: extra_account_metalist,
            }
            .to_account_metas(None),
            data: crate::instruction::Deposit { amount: 99_000_000 }.data(),
        };

        let message = Message::new(&[deposit_token_ix], Some(&user));
        let recent_blockhash = program.latest_blockhash();
        let transaction = Transaction::new(&[&user_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx5: litesvm::types::TransactionMetadata =
            program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\nDeposit token sucessfull");
        msg!("CUs Consumed: {}", tx3.compute_units_consumed);
        msg!("Tx Signature: {}", tx3.signature);

        let vault_ata_account = program.get_account(&vault).unwrap();
        let vault_ata_data: StateWithExtensions<Account> =
            StateWithExtensions::<Account>::unpack(&vault_ata_account.data).unwrap();
        assert_eq!(vault_ata_data.base.amount, 99_000_000);
    }

    #[test]
    fn test_withdraw_token() {
        let (
            mut program,
            admin_keypair,
            admin,
            user_keypair,
            user,
            user_ata,
            config,
            vault,
            mint_keypair,
            mint_token,
            user_position,
            associated_token_program,
            token_program,
            system_program,
            hook_program,
            extra_account_metalist,
            whitelist,
        ) = setup();

        let init_vault_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::InitVault {
                admin: admin,
                mint: mint_token,
                config: config,
                vault: vault,
                associated_token_program: associated_token_program,
                token_program: token_program,
                system_program: system_program,
                hook_program: hook_program,
            }
            .to_account_metas(None),
            data: crate::instruction::InitVault {}.data(),
        };

        let message = Message::new(&[init_vault_ix], Some(&admin));
        let recent_blockhash = program.latest_blockhash();

        let transaction =
            Transaction::new(&[&admin_keypair, &mint_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx1 = program.send_transaction(transaction).unwrap();

        let init_extra_account_metalist_ix = Instruction {
            program_id: hook_program,
            accounts: InitializeExtraAccountMetaList {
                payer: admin,
                mint: mint_token,
                extra_account_meta_list: extra_account_metalist,
                whitelist: whitelist,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: InitExtraAccount {}.data(),
        };

        let message = Message::new(&[init_extra_account_metalist_ix], Some(&admin));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&admin_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx2 = program.send_transaction(transaction).unwrap();

        let mint_token_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::MintToken {
                admin: admin,
                user: user,
                mint: mint_token,
                user_ata: user_ata,
                config: config,
                associated_token_program: associated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::MintToken {}.data(),
        };

        let message = Message::new(&[mint_token_ix], Some(&admin));
        let recent_blockhash = program.latest_blockhash();
        let transaction = Transaction::new(&[&admin_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx3 = program.send_transaction(transaction).unwrap();

        let whitelist_tx = Instruction {
            program_id: hook_program,
            accounts: AddToWhiteList {
                signer: admin,
                mint: mint_token,
                user: user,
                whitelist: whitelist,
            }
            .to_account_metas(None),
            data: AddWhitelist {
                amount: 100_000_000,
            }
            .data(),
        };

        let message = Message::new(&[whitelist_tx], Some(&admin));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&admin_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx4 = program.send_transaction(transaction).unwrap();

        let deposit_token_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Trade {
                user: user,
                mint: mint_token,
                user_ata: user_ata,
                config: config,
                vault: vault,
                user_position: user_position,
                associated_token_program: associated_token_program,
                token_program: token_program,
                system_program: system_program,
                hook_program: hook_program,
                whitelist: whitelist,
                extra_account_meta: extra_account_metalist,
            }
            .to_account_metas(None),
            data: crate::instruction::Deposit { amount: 99_000_000 }.data(),
        };

        let message = Message::new(&[deposit_token_ix], Some(&user));
        let recent_blockhash = program.latest_blockhash();
        let transaction = Transaction::new(&[&user_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx5: litesvm::types::TransactionMetadata =
            program.send_transaction(transaction).unwrap();

        let whitelist_tx = Instruction {
            program_id: hook_program,
            accounts: AddToWhiteList {
                signer: admin,
                mint: mint_token,
                user: config,
                whitelist: whitelist,
            }
            .to_account_metas(None),
            data: AddWhitelist {
                amount: 100_000_000,
            }
            .data(),
        };

        let message = Message::new(&[whitelist_tx], Some(&admin));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&admin_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx6 = program.send_transaction(transaction).unwrap();

        let withdraw_token_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Trade {
                user: user,
                mint: mint_token,
                user_ata: user_ata,
                config: config,
                vault: vault,
                user_position: user_position,
                associated_token_program: associated_token_program,
                token_program: token_program,
                system_program: system_program,
                hook_program: hook_program,
                whitelist: whitelist,
                extra_account_meta: extra_account_metalist,
            }
            .to_account_metas(None),
            data: crate::instruction::Withdraw { amount: 15_000_000 }.data(),
        };

        let message = Message::new(&[withdraw_token_ix], Some(&user));
        let recent_blockhash = program.latest_blockhash();
        let transaction = Transaction::new(&[&user_keypair], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx7: litesvm::types::TransactionMetadata =
            program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\nWithdraw token sucessfull");
        msg!("CUs Consumed: {}", tx7.compute_units_consumed);
        msg!("Tx Signature: {}", tx7.signature);

        let vault_ata_account = program.get_account(&vault).unwrap();
        let vault_ata_data: StateWithExtensions<Account> =
            StateWithExtensions::<Account>::unpack(&vault_ata_account.data).unwrap();
        assert_eq!(vault_ata_data.base.amount, 84_000_000);
    }
}
