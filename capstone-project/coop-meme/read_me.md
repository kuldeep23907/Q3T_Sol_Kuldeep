How to test on devnet?

Run the following commands. Since this program interacts with Raydium CPMM, we need to build our program with Raydium CPMM's devnet program addresses.

1. `anchor build -- --features devnet`
2. `anchor test -- --features devnet`

