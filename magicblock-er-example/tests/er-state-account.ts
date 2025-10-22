import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';
import { GetCommitmentSignature } from '@magicblock-labs/ephemeral-rollups-sdk';
import { ErStateAccount } from '../target/types/er_state_account';

describe('er-state-account', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const providerEphemeralRollup = new anchor.AnchorProvider(
    new anchor.web3.Connection(
      process.env.EPHEMERAL_PROVIDER_ENDPOINT ||
        'https://devnet.magicblock.app/',
      {
        wsEndpoint:
          process.env.EPHEMERAL_WS_ENDPOINT ||
          'wss://devnet.magicblock.app/',
      }
    ),
    anchor.Wallet.local()
  );
  console.log(
    'Base Layer Connection: ',
    provider.connection.rpcEndpoint
  );
  console.log(
    'Ephemeral Rollup Connection: ',
    providerEphemeralRollup.connection.rpcEndpoint
  );
  console.log(
    `Current SOL Public Key: ${anchor.Wallet.local().publicKey}`
  );

  before(async function () {
    const balance = await provider.connection.getBalance(
      anchor.Wallet.local().publicKey
    );
    console.log(
      'Current balance is',
      balance / LAMPORTS_PER_SOL,
      ' SOL',
      '\n'
    );
  });

  const program = anchor.workspace
    .erStateAccount as Program<ErStateAccount>;

  const userAccount = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('user'), anchor.Wallet.local().publicKey.toBuffer()],
    program.programId
  )[0];

  const queue_er = new PublicKey(
    '5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc'
  );

  const queue = new PublicKey(
    'Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh'
  );

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.methods
      .initialize()
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log('User Account initialized: ', tx);
  });

  it('Update State!', async () => {
    let userAccountData = await program.account.userAccount.fetch(
      userAccount,
      'processed'
    );
    // console.log('User PDA: ', userAccount.toBase58());
    console.log(
      'user data in normal state before update: ',
      userAccountData.data.toString()
    );
    const tx = await program.methods
      .update(new anchor.BN(23))
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
      })
      .rpc();

    console.log('\nUser Account State Updated: ', tx);

    let userAccountDataAfterUpdate =
      await program.account.userAccount.fetch(
        userAccount,
        'processed'
      );
    // console.log('User PDA: ', userAccount.toBase58());
    console.log(
      'user data in normal state after update: ',
      userAccountDataAfterUpdate.data.toString()
    );
  });

  it('Update State  with Vrf!', async () => {
    const tx = await program.methods
      .updateWithVrf(Math.floor(Math.random() * 256))
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
        oracleQueue: queue,
      })
      .rpc();
    console.log('\nUser Account State Updated: ', tx);

    let userAccountData = await program.account.userAccount.fetch(
      userAccount,
      'processed'
    );
    await new Promise((resolve) => setTimeout(resolve, 20000));

    // console.log('User PDA: ', userAccount.toBase58());
    console.log(
      'user data in vrf state: ',
      userAccountData.data.toString()
    );
  });

  it('Delegate to Ephemeral Rollup!', async () => {
    let tx = await program.methods
      .delegate()
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
        validator: new PublicKey(
          'MAS1Dt9qreoRMQ14YQuhg8UTZMMzDdKhmkZMECCzk57'
        ),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc({ skipPreflight: true });

    console.log('\nUser Account Delegated to Ephemeral Rollup: ', tx);
  });

  it('Update State and Commit to Base Layer!', async () => {
    // await new Promise((resolve) => setTimeout(resolve, 20000));

    let tx = await program.methods
      .updateCommit(new anchor.BN(44))
      .accountsPartial({
        user: providerEphemeralRollup.wallet.publicKey,
        userAccount: userAccount,
      })
      .transaction();

    tx.feePayer = providerEphemeralRollup.wallet.publicKey;

    tx.recentBlockhash = (
      await providerEphemeralRollup.connection.getLatestBlockhash()
    ).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);
    const txHash = await providerEphemeralRollup.sendAndConfirm(
      tx,
      [],
      { skipPreflight: false }
    );
    const txCommitSgn = await GetCommitmentSignature(
      txHash,
      providerEphemeralRollup.connection
    );

    console.log('\nUser Account State Updated: ', txHash);
  });

  it('Update State on ER with Vrf!', async () => {
    let tx = await program.methods
      .updateWithVrfOnEr(Math.floor(Math.random() * 256))
      .accountsPartial({
        user: providerEphemeralRollup.wallet.publicKey,
        userAccount: userAccount,
        oracleQueue: queue_er,
      })
      .transaction();
    // console.log('\nUser Account State Updated: ', tx);

    tx.feePayer = providerEphemeralRollup.wallet.publicKey;

    tx.recentBlockhash = (
      await providerEphemeralRollup.connection.getLatestBlockhash()
    ).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);
    const txHash = await providerEphemeralRollup.sendAndConfirm(
      tx,
      [],
      { skipPreflight: false }
    );
    // const txCommitSgn = await GetCommitmentSignature(
    //   txHash,
    //   providerEphemeralRollup.connection
    // );

    console.log('\nUser Account State Updated: ', txHash);

    // await new Promise((resolve) => setTimeout(resolve, 20000));

    // let userAccountData = await program.account.userAccount.fetch(
    //   userAccount,
    //   'processed'
    // );
    // // console.log('User PDA: ', userAccount.toBase58());
    // console.log(
    //   'user data in vrf state: ',
    //   userAccountData.data.toString()
    // );
  });

  it('Commit and undelegate from Ephemeral Rollup!', async () => {
    await new Promise((resolve) => setTimeout(resolve, 30000));

    let info =
      await providerEphemeralRollup.connection.getAccountInfo(
        userAccount
      );

    console.log('User Account Info: ', info);

    console.log('User account', userAccount.toBase58());

    let tx = await program.methods
      .undelegate()
      .accounts({
        user: providerEphemeralRollup.wallet.publicKey,
      })
      .transaction();

    tx.feePayer = providerEphemeralRollup.wallet.publicKey;

    tx.recentBlockhash = (
      await providerEphemeralRollup.connection.getLatestBlockhash()
    ).blockhash;
    tx = await providerEphemeralRollup.wallet.signTransaction(tx);
    const txHash = await providerEphemeralRollup.sendAndConfirm(
      tx,
      [],
      { skipPreflight: false }
    );
    const txCommitSgn = await GetCommitmentSignature(
      txHash,
      providerEphemeralRollup.connection
    );

    console.log('\nUser Account Undelegated: ', txHash);
  });

  it('Update State!', async () => {
    let tx = await program.methods
      .update(new anchor.BN(45))
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
      })
      .rpc();

    console.log('\nUser Account State Updated: ', tx);
  });

  it('Close Account!', async () => {
    const tx = await program.methods
      .close()
      .accountsPartial({
        user: anchor.Wallet.local().publicKey,
        userAccount: userAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log('\nUser Account Closed: ', tx);
  });
});
