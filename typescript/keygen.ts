import { Keypair } from '@solana/web3.js';
import bs58 from 'bs58'
import * as prompt from 'prompt-sync

let kp = Keypair.generate();
console.log(`solana wallet ${kp.publicKey.toBase58()}`);
console.log(`[${kp.secretKey}]`);

