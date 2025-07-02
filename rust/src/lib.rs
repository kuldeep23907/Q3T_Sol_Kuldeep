#[cfg(test)]
mod tests {
    // use solana_sdk::{
    //     pubkey::Pubkey,
    //     signature::{Keypair, Signer},
    // };

    use solana_client::rpc_client::RpcClient;
    use solana_program::{
        hash::{hash, Hash},
        pubkey::Pubkey,
        system_instruction::transfer,
        system_program,
    };
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        message::Message,
        signature::{read_keypair_file, Keypair, Signer},
        transaction::Transaction,
    };
    use std::str::FromStr;

    #[test]
    fn keygen() {
        // Create a new keypair
        let kp = Keypair::new();
        println!(
            "You've generated a new Solana wallet: {}",
            kp.pubkey().to_string()
        );
        println!("");
        println!("To save your wallet, copy and paste the following into a JSON file:");
        println!("{:?}", kp.to_bytes());
    }
    #[test]
    fn airdrop() {
        const RPC_URL: &str = "https://api.devnet.solana.com";
        let keypair = read_keypair_file("dev_wallet.json").expect("Couldn't find wallet file");
        let client = RpcClient::new(RPC_URL);

        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000u64) {
            Ok(sig) => {
                println!("Success! Check your TX here:");
                println!("https://explorer.solana.com/tx/{}?cluster=devnet", sig);
            }
            Err(err) => {
                println!("Airdrop failed: {}", err);
            }
        }
    }
    #[test]
    fn transfer_sol() {
        const RPC_URL: &str = "https://api.devnet.solana.com";
        let keypair = read_keypair_file("dev_wallet.json").expect("Couldn't find wallet file");
        let rpc_client = RpcClient::new(RPC_URL);

        let pubkey = keypair.pubkey();

        let message_bytes = b"I verify my Solana Keypair!";
        let sig = keypair.sign_message(message_bytes);
        let sig_hashed = hash(sig.as_ref());

        // Verify the signature using the public key
        match sig.verify(&pubkey.to_bytes(), &sig_hashed.to_bytes()) {
            true => println!("signature verified"),
            false => println!("signature failed"),
        }

        let to_pubkey = Pubkey::from_str("3z9kkCZ932XGac88iCm7bG4X8KFJb77aoLtC5XpGe1e2").unwrap();
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("failed blockchash");
        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, 1_000_000)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("failed txn");
        println!(
            "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn transfer_sol_all() {
        const RPC_URL: &str = "https://api.devnet.solana.com";
        let keypair = read_keypair_file("dev_wallet.json").expect("Couldn't find wallet file");
        let rpc_client = RpcClient::new(RPC_URL);

        let to_pubkey = Pubkey::from_str("3z9kkCZ932XGac88iCm7bG4X8KFJb77aoLtC5XpGe1e2").unwrap();
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("failed blockchash");

        let balance = rpc_client
            .get_balance(&keypair.pubkey())
            .expect("fialed bal");

        let message = Message::new_with_blockhash(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance)],
            Some(&keypair.pubkey()),
            &recent_blockhash,
        );

        let fee = rpc_client
            .get_fee_for_message(&message)
            .expect("failed fee for msg");

        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance - fee)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("failed txn");
        println!(
            "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn submit_rs_turbine() {
        const RPC_URL: &str = "https://api.devnet.solana.com";
        let signer = read_keypair_file("dev_wallet.json").expect("Couldn't find wallet file");
        let rpc_client = RpcClient::new(RPC_URL);
        let mint = Keypair::new();
        let turbin3_prereq_program =
            Pubkey::from_str("TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM").unwrap();
        let collection = Pubkey::from_str("5ebsp5RChCGK7ssRZMVMufgVZhd2kFbNaotcZ5UvytN2").unwrap();
        let mpl_core_program =
            Pubkey::from_str("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d").unwrap();
        let system_program = system_program::id();

        let signer_pubkey = signer.pubkey();
        let seeds = &[b"prereqs", signer_pubkey.as_ref()];
        let (prereq_pda, _bump) = Pubkey::find_program_address(seeds, &turbin3_prereq_program);
        let data = vec![77, 124, 82, 163, 21, 133, 181, 206];

        let authority_seeds = &[b"collection", collection.as_ref()];
        let (authority_prereq_pda, _authority_bump) =
            Pubkey::find_program_address(authority_seeds, &turbin3_prereq_program);

        let accounts = vec![
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new(prereq_pda, false),
            AccountMeta::new(mint.pubkey(), true),
            AccountMeta::new(collection, false),
            AccountMeta::new_readonly(authority_prereq_pda, false),
            AccountMeta::new_readonly(mpl_core_program, false),
            AccountMeta::new_readonly(system_program, false),
        ];

        let blockchash = rpc_client
            .get_latest_blockhash()
            .expect("failed to get recent blockhash");
        let instruction = Instruction {
            program_id: turbin3_prereq_program,
            accounts,
            data,
        };

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some((&signer.pubkey())),
            &[&signer, &mint],
            blockchash,
        );

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("failed to send txn");
        println!(
            "Success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }
}
