import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { AnchorVault } from '../target/types/anchor_vault';
import BN from 'bn.js';
describe('anchor-vault', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .anchorVault as Program<AnchorVault>;

  it('Is initialized!', async () => {
    const [vaultStatePda] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from('state'), provider.wallet.publicKey.toBuffer()],
        program.programId
      );

    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log('Your transaction signature', tx);

    const vaultState = await program.account.vaultState.fetch(
      vaultStatePda
    );
    console.log('Vault state data:', vaultState);
  });

  it('is checking deposit', async () => {
    const [vaultStatePda] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from('state'), provider.wallet.publicKey.toBuffer()],
        program.programId
      );

    const [vaultPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from('vault'), vaultStatePda.toBuffer()],
      program.programId
    );

    const vault_balance_before_deposit =
      await provider.connection.getBalance(vaultPda);

    // Add your test here.
    const tx = await program.methods.deposit(new BN(1_000_000)).rpc();
    // console.log('Your transaction signature', tx);

    const vault_balance_after_deposit =
      await provider.connection.getBalance(vaultPda);

    console.log(
      'Vault  balance:',
      vault_balance_after_deposit - vault_balance_before_deposit
    );
  });

  it('is checking withdraw', async () => {
    const [vaultStatePda] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from('state'), provider.wallet.publicKey.toBuffer()],
        program.programId
      );

    const [vaultPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from('vault'), vaultStatePda.toBuffer()],
      program.programId
    );

    const tx = await program.methods.deposit(new BN(2_000_000)).rpc();

    const vault_balance_before_withdraw =
      await provider.connection.getBalance(vaultPda);

    const user_balance_before_withdraw =
      await provider.connection.getBalance(provider.wallet.publicKey);

    // Add your test here.
    const tx2 = await program.methods
      .withdraw(new BN(1_000_000))
      .rpc();
    // console.log('Your transaction signature', tx);

    const vault_balance_after_withdraw =
      await provider.connection.getBalance(vaultPda);

    const user_balance_after_withdraw =
      await provider.connection.getBalance(provider.wallet.publicKey);

    console.log(
      'Vault  balance:',
      vault_balance_after_withdraw - vault_balance_before_withdraw
    );

    console.log(
      'User  balance:',
      user_balance_after_withdraw - user_balance_before_withdraw
    );
  });

  it('is checking close vault', async () => {
    const [vaultStatePda] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from('state'), provider.wallet.publicKey.toBuffer()],
        program.programId
      );

    const [vaultPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from('vault'), vaultStatePda.toBuffer()],
      program.programId
    );

    const tx = await program.methods.deposit(new BN(2_000_000)).rpc();

    const user_balance_before_close =
      await provider.connection.getBalance(provider.wallet.publicKey);

    // Add your test here.
    const tx2 = await program.methods.closeVault().rpc();
    // console.log('Your transaction signature', tx);

    const user_balance_after_close =
      await provider.connection.getBalance(provider.wallet.publicKey);

    console.log(
      'User  balance:',
      user_balance_after_close - user_balance_before_close
    );

    console.log(
      'vault balance',
      await provider.connection.getBalance(vaultPda)
    );
  });
});
