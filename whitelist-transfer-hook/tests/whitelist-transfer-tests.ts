import * as anchor from '@coral-xyz/anchor';
import { Program, Wallet, BN } from '@coral-xyz/anchor';
import {
  PublicKey,
  Keypair,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
  SendTransactionError,
} from '@solana/web3.js';
import {
  MintLayout,
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
  createAssociatedTokenAccountInstruction,
  mintTo,
  getAssociatedTokenAddressSync,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMintToInstruction,
  createTransferCheckedWithTransferHookInstruction,
  getAssociatedTokenAddress,
} from '@solana/spl-token';
import { WhitelistTransferHook } from '../target/types/whitelist_transfer_hook';

describe('whitelist-transfer-hook', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .WhitelistTransferHook as Program<WhitelistTransferHook>;

  let user: Wallet = provider.wallet as Wallet;
  // let mint;
  // let extraAccountMetaListKeypair;
  // let recipient: Keypair;

  // beforeEach(() => {

  //   extraAccountMetaListKeypair = extraAccountMetaList;
  // });

  // it('Creates token mint successfully', async () => {
  //   // Transaction to create token mint with create_token instruction
  //   const tx = await program.methods
  //     .createToken()
  //     .accounts({
  //       user: user.publicKey,
  //       mint: mintKeypair.publicKey,
  //       extraAccountMetaList: extraAccountMetaListKeypair,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       tokenProgram: TOKEN_2022_PROGRAM_ID,
  //     })
  //     .signers([mintKeypair])
  //     .rpc();

  //   console.log('Transaction signature:', tx);

  //   // Fetch the mint account info and check decimals and mint authority
  //   const mintAccount =
  //     await program.provider.connection.getAccountInfo(
  //       mintKeypair.publicKey
  //     );
  //   if (mintAccount === null) {
  //     throw new Error('Mint account not found');
  //   }

  //   const mintData = MintLayout.decode(mintAccount.data);

  //   // Decimals is at offset 44 in MintLayout
  //   const decimals = mintData.decimals;
  //   // Mint authority is 32 bytes starting offset 0 (first field)
  //   const actualMintAuthority = new PublicKey(mintData.mintAuthority);

  //   console.log('Mint decimals:', decimals);
  //   console.log('Mint authority:', actualMintAuthority.toBase58());

  //   // Assert decimals = 9 as expected
  //   if (decimals !== 9) {
  //     throw new Error(`Expected decimals to be 9, got ${decimals}`);
  //   }

  //   // Assert mint authority is the user wallet
  //   if (!actualMintAuthority.equals(user.publicKey)) {
  //     throw new Error('Mint authority does not match user');
  //   }
  // });

  // it('Adds user to whitelist', async () => {
  //   const admin = provider.wallet.publicKey;

  //   const [whitelistPDA, whitelistBump] =
  //     await PublicKey.findProgramAddress(
  //       [
  //         Buffer.from('whitelist'), // first seed
  //         user.publicKey.toBuffer(), // second seed (user key as bytes)
  //       ],
  //       program.programId // your on-chain program ID
  //     );

  //   const tx = await program.methods
  //     .addToWhitelist()
  //     .accounts({
  //       admin,
  //       user: user.publicKey,
  //       whitelist: whitelistPDA,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .rpc();

  //   console.log('Add to whitelist tx signature:', tx);

  //   // Fetch whitelist account to confirm it exists and has expected data
  //   const whitelistAccount = await program.account.whitelist.fetch(
  //     whitelistPDA
  //   );
  //   // Add assertions as per your Whitelist struct fields, example:
  //   // assert.ok(whitelistAccount.someFlag === true);
  // });

  it.only('Initializes mint and extra account meta list, whitelist user, then calls transfer_hook', async () => {
    // const mintKeypair = anchor.web3.Keypair.generate();
    // const mint = mintKeypair.publicKey;

    const [mint] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('mint')],
      program.programId
    );
    // let recipient = Keypair.generate();

    const [extraAccountMetaList] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from('extra-account-metas'), mint.toBuffer()],
        program.programId
      );

    const sourceTokenUserAta = await getAssociatedTokenAddress(
      mint,
      user.publicKey,
      false, // allowOwnerOffCurve = false (always false unless you know it's needed)
      TOKEN_2022_PROGRAM_ID
    );

    console.log(mint);

    console.log(sourceTokenUserAta);

    // 1. Create mint with create_token instruction
    await program.methods
      .createToken()
      .accounts({
        user: user.publicKey,
        mint,
        sourceTokenAta: sourceTokenUserAta,
        extraAccountMetaList,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      // .signers([user.payer, mintKeypair])
      .rpc();

    // // 2. Add user to whitelist
    // const admin = provider.wallet.publicKey;

    // const [whitelistPDA, whitelistBump] =
    //   await PublicKey.findProgramAddress(
    //     [
    //       Buffer.from('whitelist'), // first seed
    //       user.publicKey.toBuffer(), // second seed (user key as bytes)
    //     ],
    //     program.programId // your on-chain program ID
    //   );

    // const tx = await program.methods
    //   .addToWhitelist()
    //   .accounts({
    //     admin,
    //     user: user.publicKey,
    //     whitelist: whitelistPDA,
    //     systemProgram: SystemProgram.programId,
    //   })
    //   .rpc();

    // const amount = 100 * 10 ** 9;

    // const recpTokenUserAta = await getAssociatedTokenAddress(
    //   mintKeypair.publicKey,
    //   recipient.publicKey,
    //   false, // allowOwnerOffCurve = false (always false unless you know it's needed)
    //   TOKEN_2022_PROGRAM_ID
    // );

    // await program.methods
    //   .transferToken(new BN(amount))
    //   .accounts({
    //     sourceToken: sourceTokenUserAta,
    //     mint: mintKeypair.publicKey,
    //     destinationToken: recpTokenUserAta,
    //     owner: user.publicKey,
    //     extraAccountMetaList: extraAccountMetaListKeypair,
    //     whitelist: whitelistPDA,
    //     systemProgram: anchor.web3.SystemProgram.programId,
    //     tokenProgram: TOKEN_2022_PROGRAM_ID,
    //   })
    //   .rpc();
  });
});
