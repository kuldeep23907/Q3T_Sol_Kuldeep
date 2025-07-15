import { createSolanaClient } from 'gill';
import { type KeyPairSigner } from 'gill';
import { loadKeypairSignerFromFile } from 'gill/node';
import { TOKEN_PROGRAM_ADDRESS } from 'gill/programs/token';
import {
  generateKeyPairSigner,
  getMinimumBalanceForRentExemption,
  getExplorerLink,
  getSignatureFromTransaction,
} from 'gill';
import {
  getTokenMetadataAddress,
  getCreateAccountInstruction,
  getInitializeMintInstruction,
  getCreateMetadataAccountV3Instruction,
} from 'gill/programs';
import { signTransactionMessageWithSigners } from 'gill';

import { createTransaction } from 'gill';
import { getMintSize } from 'gill/programs/token';

const tokenProgram = TOKEN_PROGRAM_ADDRESS;
const { rpc, sendAndConfirmTransaction } = createSolanaClient({
  urlOrMoniker: 'devnet', // `mainnet`, `localnet`, etc
});
const mint = await generateKeyPairSigner();
const metadataAddress = await getTokenMetadataAddress(mint);

// const zeroPubkey = await generateKeyPairSigner(new Uint8Array(32));
const { value: latestBlockhash } = await rpc
  .getLatestBlockhash()
  .send();

// This defaults to the file path used by the Solana CLI: `~/.config/solana/id.json`
const signer: KeyPairSigner = await loadKeypairSignerFromFile();
console.log('signer:', signer.address);

//https://devnet.irys.xyz/BD2VCNFHvdkCoBR2UrEAfVz3aopAq5S8cCEm65syXDWX

const space = getMintSize();

const transaction = createTransaction({
  feePayer: signer,
  version: 'legacy',
  instructions: [
    getCreateAccountInstruction({
      space,
      lamports: getMinimumBalanceForRentExemption(space),
      newAccount: mint,
      payer: signer,
      programAddress: tokenProgram,
    }),
    getInitializeMintInstruction(
      {
        mint: mint.address,
        mintAuthority: signer.address,
        freezeAuthority: signer.address,
        decimals: 0,
      },
      {
        programAddress: tokenProgram,
      }
    ),
    getCreateMetadataAccountV3Instruction({
      collectionDetails: null,
      isMutable: true,
      updateAuthority: signer,
      mint: mint.address,
      metadata: metadataAddress,
      mintAuthority: signer,
      payer: signer,
      data: {
        sellerFeeBasisPoints: 500,
        collection: null,
        creators: null,
        uses: null,
        name: 'Generug NFT',
        symbol: 'GENRUG',
        uri: 'https://gateway.irys.xyz/E5Pfby26sCv6coPZUX8subtPos4hPXs8ogfNQ7iqXVnq',
      },
    }),
  ],
  latestBlockhash,
});

const signedTransaction = await signTransactionMessageWithSigners(
  transaction
);

console.log(
  'Explorer:',
  getExplorerLink({
    cluster: 'devnet',
    transaction: getSignatureFromTransaction(signedTransaction),
  })
);

await sendAndConfirmTransaction(signedTransaction);
