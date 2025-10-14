#[cfg(test)]
mod tests {

    use {
        anchor_lang::{
            prelude::{msg, Clock, SolanaSysvar},
            solana_program::{program_pack::Pack, sysvar::SysvarId},
            AccountDeserialize, InstructionData, Key, ToAccountMetas,
        },
        anchor_spl::{
            associated_token::{self, spl_associated_token_account},
            token::spl_token,
        },
        litesvm::LiteSVM,
        litesvm_token::{
            spl_token::ID as TOKEN_PROGRAM_ID, CreateAssociatedTokenAccount, CreateMint, MintTo,
        },
        solana_account::Account,
        solana_address::Address,
        solana_instruction::Instruction,
        solana_keypair::Keypair,
        solana_message::Message,
        solana_native_token::LAMPORTS_PER_SOL,
        solana_pubkey::Pubkey,
        solana_rpc_client::rpc_client::RpcClient,
        solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID,
        solana_signer::Signer,
        solana_transaction::Transaction,
        std::{path::PathBuf, str::FromStr},
        // borsh::BorshDeserialize;
    };

    static PROGRAM_ID: Pubkey = crate::ID;

    fn setup() -> (
        LiteSVM,
        Keypair,
        Keypair,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
    ) {
        // Initialize LiteSVM and payer
        let mut program = LiteSVM::new();
        let payer = Keypair::new();
        let maker = payer.pubkey();
        let payer2 = Keypair::new();
        let taker = payer2.pubkey();

        // Airdrop some SOL to the payer keypair
        program
            .airdrop(&maker, 1000 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to payer");

        let account = program.get_account(&maker).unwrap();
        let mut lamports = account.lamports;

        msg!("malports remaining {}", lamports);

        // Airdrop some SOL to the payer keypair
        program
            .airdrop(&taker, 1000 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to payer");

        // Load program SO file
        let so_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/anchor_escrow.so");

        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        program.add_program(PROGRAM_ID, &program_data);

        // // Example on how to Load an account from devnet
        // let rpc_client = RpcClient::new("https://api.devnet.solana.com");
        // let account_address =
        //     Address::from_str("DRYvf71cbF2s5wgaJQvAGkghMkRcp5arvsK2w97vXhi2").unwrap();
        // let fetched_account = rpc_client
        //     .get_account(&account_address)
        //     .expect("Failed to fetch account from devnet");

        // program
        //     .set_account(
        //         payer.pubkey(),
        //         Account {
        //             lamports: fetched_account.lamports,
        //             data: fetched_account.data,
        //             owner: Pubkey::from(fetched_account.owner.to_bytes()),
        //             executable: fetched_account.executable,
        //             rent_epoch: fetched_account.rent_epoch,
        //         },
        //     )
        //     .unwrap();

        // msg!("Lamports of fetched account: {}", fetched_account.lamports);

        // Create two mints (Mint A and Mint B) with 6 decimal places and the maker as the authority
        let mint_a = CreateMint::new(&mut program, &payer)
            .decimals(6)
            .authority(&maker)
            .send()
            .unwrap();

        let mint_b = CreateMint::new(&mut program, &payer)
            .decimals(6)
            .authority(&maker)
            .send()
            .unwrap();
        // Create the maker's associated token account for Mint A
        let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
            .owner(&maker)
            .send()
            .unwrap();

        // Create the maker's associated token account for Mint A
        let maker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_b)
            .owner(&maker)
            .send()
            .unwrap();

        // Create the maker's associated token account for Mint A
        let taker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer2, &mint_a)
            .owner(&taker)
            .send()
            .unwrap();

        // Create the maker's associated token account for Mint A
        let taker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &payer2, &mint_b)
            .owner(&taker)
            .send()
            .unwrap();

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let escrow = Pubkey::find_program_address(
            &[b"escrow", maker.as_ref(), &123u64.to_le_bytes()],
            &PROGRAM_ID,
        )
        .0;

        // Derive the PDA for the vault associated token account using the escrow PDA and Mint A
        let vault = associated_token::get_associated_token_address(&escrow, &mint_a);

        lamports = account.lamports;

        msg!("malports remaining {} after all accounr set", lamports);

        // Return the LiteSVM instance and payer keypair
        (
            program,
            payer,
            payer2,
            maker,
            taker,
            mint_a,
            mint_b,
            escrow,
            vault,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
        )
    }

    #[test]
    fn test_make() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let (
            mut program,
            payer,
            payer2,
            maker,
            taker,
            mint_a,
            mint_b,
            escrow,
            vault,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
        ) = setup();

        // // Get the maker's public key from the payer keypair
        // let maker = payer.pubkey();

        msg!("Mint A: {}\n", mint_a);

        msg!("Mint B: {}\n", mint_b);

        msg!("Maker ATA A: {}\n", maker_ata_a);

        msg!("Escrow PDA: {}\n", escrow);

        msg!("Vault PDA: {}\n", vault);

        // msg!("payer balance {}\n", maker.)

        // Define program IDs for associated token program, token program, and system program
        let asspciated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Make {
                maker: maker,
                mint_a: mint_a,
                mint_b: mint_b,
                maker_ata_a: maker_ata_a,
                escrow: escrow,
                vault: vault,
                associated_token_program: asspciated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Make {
                deposit: 10,
                seed: 123u64,
                receive: 10,
            }
            .data(),
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let account = program.get_account(&maker).unwrap();
        let lamports = account.lamports;

        msg!("malports remaining {}", lamports);

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        // Verify the vault account and escrow account data after the "Make" instruction
        let vault_account = program.get_account(&vault).unwrap();
        let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
        assert_eq!(vault_data.amount, 10);
        assert_eq!(vault_data.owner, escrow);
        assert_eq!(vault_data.mint, mint_a);

        let escrow_account = program.get_account(&escrow).unwrap();
        let escrow_data =
            crate::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
        assert_eq!(escrow_data.seed, 123u64);
        assert_eq!(escrow_data.maker, maker);
        assert_eq!(escrow_data.mint_a, mint_a);
        assert_eq!(escrow_data.mint_b, mint_b);
        assert_eq!(escrow_data.receive, 10);
    }

    #[test]
    fn test_take() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let (
            mut program,
            payer,
            payer2,
            maker,
            taker,
            mint_a,
            mint_b,
            escrow,
            vault,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
        ) = setup();

        // // Get the maker's public key from the payer keypair
        // let maker = payer.pubkey();

        msg!("Mint A: {}\n", mint_a);

        msg!("Mint B: {}\n", mint_b);

        msg!("Maker ATA A: {}\n", maker_ata_a);

        msg!("Escrow PDA: {}\n", escrow);

        msg!("Vault PDA: {}\n", vault);

        // Define program IDs for associated token program, token program, and system program
        let asspciated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Make {
                maker: maker,
                mint_a: mint_a,
                mint_b: mint_b,
                maker_ata_a: maker_ata_a,
                escrow: escrow,
                vault: vault,
                associated_token_program: asspciated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Make {
                deposit: 10,
                seed: 123u64,
                receive: 10,
            }
            .data(),
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx1 = program.send_transaction(transaction).unwrap();

        // Mint 1,000 tokens (with 6 decimal places) of Mint B to the takers's associated token account
        MintTo::new(&mut program, &payer, &mint_b, &taker_ata_b, 1000000000)
            .send()
            .unwrap();

        // Create the "Make" instruction to deposit tokens into the escrow
        let take_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Take {
                taker: taker,
                maker: maker,
                mint_a: mint_a,
                mint_b: mint_b,
                taker_ata_a: taker_ata_a,
                taker_ata_b: taker_ata_b,
                maker_ata_b: maker_ata_b,
                escrow: escrow,
                vault: vault,
                associated_token_program: asspciated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Take {}.data(),
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[take_ix], Some(&payer2.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&payer2], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\nTake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        let maker_ata_b_account = program.get_account(&maker_ata_b).unwrap();
        let maker_ata_b_data: spl_token::state::Account =
            spl_token::state::Account::unpack(&maker_ata_b_account.data).unwrap();
        assert_eq!(maker_ata_b_data.amount, 10);

        let taker_ata_a_account = program.get_account(&taker_ata_a).unwrap();
        let taker_ata_a_data: spl_token::state::Account =
            spl_token::state::Account::unpack(&taker_ata_a_account.data).unwrap();
        assert_eq!(taker_ata_a_data.amount, 10);
    }

    #[test]
    fn test_refund() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let (
            mut program,
            payer,
            payer2,
            maker,
            taker,
            mint_a,
            mint_b,
            escrow,
            vault,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
        ) = setup();

        // // Get the maker's public key from the payer keypair
        // let maker = payer.pubkey();

        msg!("Mint A: {}\n", mint_a);

        msg!("Mint B: {}\n", mint_b);

        msg!("Maker ATA A: {}\n", maker_ata_a);

        msg!("Escrow PDA: {}\n", escrow);

        msg!("Vault PDA: {}\n", vault);

        // msg!("payer balance {}\n", maker.)

        // Define program IDs for associated token program, token program, and system program
        let asspciated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        let maker_ata_a_account = program.get_account(&maker_ata_a).unwrap();
        let maker_ata_a_data: spl_token::state::Account =
            spl_token::state::Account::unpack(&maker_ata_a_account.data).unwrap();

        let maker_ata_a_balance_before_make: u64 = maker_ata_a_data.amount;

        msg!(
            "maker ata a balance before make {}",
            maker_ata_a_balance_before_make
        );
        // Create the "Make" instruction to deposit tokens into the escrow
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Make {
                maker: maker,
                mint_a: mint_a,
                mint_b: mint_b,
                maker_ata_a: maker_ata_a,
                escrow: escrow,
                vault: vault,
                associated_token_program: asspciated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Make {
                deposit: 10,
                seed: 123u64,
                receive: 10,
            }
            .data(),
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx1 = program.send_transaction(transaction).unwrap();

        let maker_ata_a_account = program.get_account(&maker_ata_a).unwrap();
        let maker_ata_a_data: spl_token::state::Account =
            spl_token::state::Account::unpack(&maker_ata_a_account.data).unwrap();
        let maker_ata_a_balance_after_make: u64 = maker_ata_a_data.amount;

        msg!(
            "maker ata a balance after make {}",
            maker_ata_a_balance_after_make
        );

        assert_eq!(maker_ata_a_balance_after_make, 1000000000 - 10);

        // Create the "Make" instruction to deposit tokens into the escrow
        let refund_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Refund {
                maker: maker,
                mint_a: mint_a,
                maker_ata_a: maker_ata_a,
                escrow: escrow,
                vault: vault,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::Refund {}.data(),
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[refund_ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);
        // Send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\nRefund transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        let maker_ata_a_account = program.get_account(&maker_ata_a).unwrap();
        let maker_ata_a_data: spl_token::state::Account =
            spl_token::state::Account::unpack(&maker_ata_a_account.data).unwrap();
        let maker_ata_a_balance_after_refund: u64 = maker_ata_a_data.amount;

        assert_eq!(maker_ata_a_balance_after_refund, 1000000000);
    }

    #[test]
    fn test_make_with_interval() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let (
            mut program,
            payer,
            payer2,
            maker,
            taker,
            mint_a,
            mint_b,
            escrow,
            vault,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
        ) = setup();

        // // Get the maker's public key from the payer keypair
        // let maker = payer.pubkey();

        msg!("Mint A: {}\n", mint_a);

        msg!("Mint B: {}\n", mint_b);

        msg!("Maker ATA A: {}\n", maker_ata_a);

        msg!("Escrow PDA: {}\n", escrow);

        msg!("Vault PDA: {}\n", vault);

        // msg!("payer balance {}\n", maker.)

        // Define program IDs for associated token program, token program, and system program
        let asspciated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::MakeInterval {
                maker: maker,
                mint_a: mint_a,
                mint_b: mint_b,
                maker_ata_a: maker_ata_a,
                escrow: escrow,
                vault: vault,
                associated_token_program: asspciated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::MakeWithInterval {
                seed: 123u64,
                deposit: 10,
                interval: 120,
                receive: 10,
            }
            .data(),
        };

        // let clock_account = program.get_account(&Clock::id()).unwrap();
        // let clock = Clock::try_from_slice(&clock_account.data).unwrap();
        // let current_timestamp = clock.unix_timestamp;

        // let clock = Clock::get();
        // let current_time = clock.unwrap().unix_timestamp as u64;

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let account = program.get_account(&maker).unwrap();
        let lamports = account.lamports;

        msg!("malports remaining {}", lamports);

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        // Verify the vault account and escrow account data after the "Make" instruction
        let vault_account = program.get_account(&vault).unwrap();
        let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
        assert_eq!(vault_data.amount, 10);
        assert_eq!(vault_data.owner, escrow);
        assert_eq!(vault_data.mint, mint_a);

        let escrow_account = program.get_account(&escrow).unwrap();
        let escrow_data =
            crate::state::EscrowInterval::try_deserialize(&mut escrow_account.data.as_ref())
                .unwrap();
        assert_eq!(escrow_data.seed, 123u64);
        // assert_eq!(escrow_data.interval, current_timestamp + 120);
        assert_eq!(escrow_data.maker, maker);
        assert_eq!(escrow_data.mint_a, mint_a);
        assert_eq!(escrow_data.mint_b, mint_b);
        assert_eq!(escrow_data.receive, 10);
    }

    #[test]
    fn test_take_with_interval() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let (
            mut program,
            payer,
            payer2,
            maker,
            taker,
            mint_a,
            mint_b,
            escrow,
            vault,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
        ) = setup();

        // // Get the maker's public key from the payer keypair
        // let maker = payer.pubkey();

        msg!("Mint A: {}\n", mint_a);

        msg!("Mint B: {}\n", mint_b);

        msg!("Maker ATA A: {}\n", maker_ata_a);

        msg!("Escrow PDA: {}\n", escrow);

        msg!("Vault PDA: {}\n", vault);

        // Define program IDs for associated token program, token program, and system program
        let asspciated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        let mut initial_clock = program.get_sysvar::<Clock>();
        initial_clock.unix_timestamp = 100;
        program.set_sysvar::<Clock>(&initial_clock);

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::MakeInterval {
                maker: maker,
                mint_a: mint_a,
                mint_b: mint_b,
                maker_ata_a: maker_ata_a,
                escrow: escrow,
                vault: vault,
                associated_token_program: asspciated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::MakeWithInterval {
                deposit: 10,
                seed: 123u64,
                interval: 120,
                receive: 10,
            }
            .data(),
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx1 = program.send_transaction(transaction).unwrap();

        // Mint 1,000 tokens (with 6 decimal places) of Mint B to the takers's associated token account
        MintTo::new(&mut program, &payer, &mint_b, &taker_ata_b, 1000000000)
            .send()
            .unwrap();

        let mut clock = program.get_sysvar::<Clock>();
        clock.unix_timestamp = 250;
        program.set_sysvar::<Clock>(&clock);

        // Create the "Make" instruction to deposit tokens into the escrow
        let take_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::TakeInterval {
                taker: taker,
                maker: maker,
                mint_a: mint_a,
                mint_b: mint_b,
                taker_ata_a: taker_ata_a,
                taker_ata_b: taker_ata_b,
                maker_ata_b: maker_ata_b,
                escrow: escrow,
                vault: vault,
                associated_token_program: asspciated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::TakeWithInterval {}.data(),
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[take_ix], Some(&payer2.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&payer2], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\nTake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        let maker_ata_b_account = program.get_account(&maker_ata_b).unwrap();
        let maker_ata_b_data: spl_token::state::Account =
            spl_token::state::Account::unpack(&maker_ata_b_account.data).unwrap();
        assert_eq!(maker_ata_b_data.amount, 10);

        let taker_ata_a_account = program.get_account(&taker_ata_a).unwrap();
        let taker_ata_a_data: spl_token::state::Account =
            spl_token::state::Account::unpack(&taker_ata_a_account.data).unwrap();
        assert_eq!(taker_ata_a_data.amount, 10);
    }

    #[test]
    fn test_refund_with_interval() {
        // Setup the test environment by initializing LiteSVM and creating a payer keypair
        let (
            mut program,
            payer,
            payer2,
            maker,
            taker,
            mint_a,
            mint_b,
            escrow,
            vault,
            maker_ata_a,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
        ) = setup();

        // // Get the maker's public key from the payer keypair
        // let maker = payer.pubkey();

        msg!("Mint A: {}\n", mint_a);

        msg!("Mint B: {}\n", mint_b);

        msg!("Maker ATA A: {}\n", maker_ata_a);

        msg!("Escrow PDA: {}\n", escrow);

        msg!("Vault PDA: {}\n", vault);

        // msg!("payer balance {}\n", maker.)

        // Define program IDs for associated token program, token program, and system program
        let asspciated_token_program = spl_associated_token_account::ID;
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = SYSTEM_PROGRAM_ID;

        // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
        MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000000000)
            .send()
            .unwrap();

        let maker_ata_a_account = program.get_account(&maker_ata_a).unwrap();
        let maker_ata_a_data: spl_token::state::Account =
            spl_token::state::Account::unpack(&maker_ata_a_account.data).unwrap();

        let maker_ata_a_balance_before_make: u64 = maker_ata_a_data.amount;

        msg!(
            "maker ata a balance before make {}",
            maker_ata_a_balance_before_make
        );

        let mut initial_clock = program.get_sysvar::<Clock>();
        initial_clock.unix_timestamp = 100;
        program.set_sysvar::<Clock>(&initial_clock);

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::MakeInterval {
                maker: maker,
                mint_a: mint_a,
                mint_b: mint_b,
                maker_ata_a: maker_ata_a,
                escrow: escrow,
                vault: vault,
                associated_token_program: asspciated_token_program,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::MakeWithInterval {
                deposit: 10,
                seed: 123u64,
                interval: 120,
                receive: 10,
            }
            .data(),
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx1 = program.send_transaction(transaction).unwrap();

        let maker_ata_a_account = program.get_account(&maker_ata_a).unwrap();
        let maker_ata_a_data: spl_token::state::Account =
            spl_token::state::Account::unpack(&maker_ata_a_account.data).unwrap();
        let maker_ata_a_balance_after_make: u64 = maker_ata_a_data.amount;

        msg!(
            "maker ata a balance after make {}",
            maker_ata_a_balance_after_make
        );

        assert_eq!(maker_ata_a_balance_after_make, 1000000000 - 10);

        let mut clock = program.get_sysvar::<Clock>();
        clock.unix_timestamp = 250;
        program.set_sysvar::<Clock>(&clock);

        // Create the "Make" instruction to deposit tokens into the escrow
        let refund_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::RefundInterval {
                maker: maker,
                mint_a: mint_a,
                maker_ata_a: maker_ata_a,
                escrow: escrow,
                vault: vault,
                token_program: token_program,
                system_program: system_program,
            }
            .to_account_metas(None),
            data: crate::instruction::RefundWithInterval {}.data(),
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[refund_ix], Some(&payer.pubkey()));
        let recent_blockhash = program.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);
        // Send the transaction and capture the result
        let tx = program.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\nRefund transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);

        let maker_ata_a_account = program.get_account(&maker_ata_a).unwrap();
        let maker_ata_a_data: spl_token::state::Account =
            spl_token::state::Account::unpack(&maker_ata_a_account.data).unwrap();
        let maker_ata_a_balance_after_refund: u64 = maker_ata_a_data.amount;

        assert_eq!(maker_ata_a_balance_after_refund, 1000000000);
    }
}
