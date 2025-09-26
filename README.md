# Authensus

This is a pair of Anchor Solana Programs, which can be integrated into, for example, a Next.js app. They contain:

- A market program that details the rules and constraints for implementing the Fortuna parimutuel market protocol (https://www.researchgate.net/publication/363682812_Fortuna_A_Novel_Staked_Voting_System_for_Distributed_Pari-Mutuel_Gaming).
- A Voting Token program, which allows tokens to be minted for use in the voting phase of the Fortuna protocol.

Things that still need to be implemented:
- Transfer hook restrictions to limit the tokens' transferability to the treasury account only.
- Random restrictions on the markets that are votable to any one user - based on the Fortuna protocol.

## Getting Started

### Installation

#### Install Dependencies

```shell
pnpm install
```

## Apps

### anchor

This is a Solana program written in Rust using the Anchor framework.

#### Sync the program id:

Running this command will create a new keypair in the `anchor/target/deploy` directory and save the address to the
Anchor config file and update the `declare_id!` macro in the `./src/lib.rs` file of the program.

```shell
pnpm anchor keys sync
```

#### Build the program:

The normal build instruction,
```shell
pnpm anchor-build
```
does not work for these Anchor programs due to conflicts between the anchor-spl and proc-macro2 crates in the current solana-program version. To get around this, we can build by running
```shell
anchor build --no-idl
```
followed by each of the following:
```shell
RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor idl build -p voting_tokens -o target/idl/voting_tokens.json -t target/types/voting_tokens.ts
RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor idl build -p market -o target/idl/market.json -t target/types/market.ts
```

#### Run the tests

For the same reasons, we cannot simply execute the usual test command,
```shell
pnpm anchor-test
```
Instead we must run test suites for each program individually:
```shell
RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor test --run tests/voting_tokens/*.ts
RUSTUP_TOOLCHAIN=nightly-2025-04-01 anchor test --run tests/market/*.ts
```

#### Deploy to Devnet

To deploy the programs to DevNet we can use the commands:
```shell
anchor deploy --program-name voting_tokens --provider.cluster devnet
anchor deploy --program-name market --provider.cluster devnet
```
Be aware that you will need some DevNet SOL to deploy
