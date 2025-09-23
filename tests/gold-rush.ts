import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { GoldRush } from "../target/types/gold_rush";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import {
  createMint,
  createAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccount,
} from "@solana/spl-token";
import { BN } from "bn.js";
import { expect } from "chai";

enum MarketType {
  GoldPrice = 0,
  StockPrice = 1,
}

describe("Gold Rust Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.goldRush as Program<GoldRush>;

  let admin: Keypair;
  let keeper: Keypair;
  let treasury: Keypair;
  let bettorWin1: Keypair;
  let bettorWin2: Keypair;
  let bettorLost1: Keypair;

  let tokenMint: PublicKey;
  let mintAuthority: Keypair;
  let adminTokenAccount: PublicKey;
  let keeperTokenAccount: PublicKey;
  let treasuryTokenAccount: PublicKey;
  let bettorWin1TokenAccount: PublicKey;
  let bettorWin2TokenAccount: PublicKey;
  let bettorLost1TokenAccount: PublicKey;

  let configPda: PublicKey;
  let round1Pda: PublicKey;
  let round1VaultPda: PublicKey;
  let betWinner1Pda: PublicKey;
  let betWinner2Pda: PublicKey;
  let betLoser1Pda: PublicKey;

  const startGoldPrice = 3_787_630;
  const endGoldPrice = 3_900_000;

  before(async () => {
    // generate keypairs
    admin = Keypair.generate();
    keeper = Keypair.generate();
    treasury = Keypair.generate();
    bettorWin1 = Keypair.generate();
    bettorWin2 = Keypair.generate();
    bettorLost1 = Keypair.generate();
    mintAuthority = Keypair.generate();

    // aidrop SOL
    await Promise.all([
      provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(admin.publicKey, 2_000_000_000)
      ),
      provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(keeper.publicKey, 1e9)
      ),
      provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(treasury.publicKey, 1e9)
      ),
      provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(bettorWin1.publicKey, 1e9)
      ),
      provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(bettorWin2.publicKey, 1e9)
      ),
      provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(bettorLost1.publicKey, 1e9)
      ),
      provider.connection.confirmTransaction(
        await provider.connection.requestAirdrop(mintAuthority.publicKey, 1e9)
      ),
    ]);

    // Create Mint and Token Accounts
    console.log("Creating token mint...");
    tokenMint = await createMint(
      provider.connection,
      mintAuthority,
      mintAuthority.publicKey,
      null,
      9
    );

    console.log("Token Mint created:", tokenMint.toString());

    // Create Associated Token Accounts for all users
    console.log("Creating token accounts...");
    adminTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      admin,
      tokenMint,
      admin.publicKey
    );

    keeperTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      keeper,
      tokenMint,
      keeper.publicKey
    );

    treasuryTokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      treasury,
      tokenMint,
      treasury.publicKey
    );

    bettorWin1TokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      bettorWin1,
      tokenMint,
      bettorWin1.publicKey
    );

    bettorWin2TokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      bettorWin2,
      tokenMint,
      bettorWin2.publicKey
    );

    bettorLost1TokenAccount = await createAssociatedTokenAccount(
      provider.connection,
      bettorLost1,
      tokenMint,
      bettorLost1.publicKey
    );

    // Mint some tokens to bettors for testing
    console.log("Minting tokens to users...");
    const mintAmount = 1000_000_000_000; // 1000 GRT dengan 9 decimals

    await Promise.all([
      mintTo(
        provider.connection,
        mintAuthority,
        tokenMint,
        adminTokenAccount,
        mintAuthority.publicKey,
        mintAmount,
        [mintAuthority]
      ),
      mintTo(
        provider.connection,
        mintAuthority,
        tokenMint,
        bettorWin1TokenAccount,
        mintAuthority.publicKey,
        mintAmount,
        [mintAuthority]
      ),
      mintTo(
        provider.connection,
        mintAuthority,
        tokenMint,
        bettorWin2TokenAccount,
        mintAuthority.publicKey,
        mintAmount,
        [mintAuthority]
      ),
      mintTo(
        provider.connection,
        mintAuthority,
        tokenMint,
        bettorLost1TokenAccount,
        mintAuthority.publicKey,
        mintAmount,
        [mintAuthority]
      ),
    ]);

    console.log("Token accounts created and tokens minted successfully!");

    // derive pdas
    [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );
    [round1Pda] = PublicKey.findProgramAddressSync(
      [Buffer.from("round"), new BN(1).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    [round1VaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), round1Pda.toBuffer()],
      program.programId
    );
  });

  it("1. Initialize - Successfully initializes the program", async () => {
    try {
      const tx = await program.methods
        .initialize(
          [keeper.publicKey],
          tokenMint,
          treasury.publicKey,
          2_000, // 2%
          2_500, // 2.5%
          new anchor.BN(10_000_000), // 10 GRT
          1_000, // 1 %
          1_000, // 1 %
          1_000 // 1 %
        )
        .accounts({
          signer: admin.publicKey,
          config: configPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      console.log("Signature", tx);

      // verify
      const configAccount = await program.account.config.fetch(configPda);
      expect(configAccount.treasury.toString()).to.equal(
        treasury.publicKey.toString()
      );
    } catch (err) {
      if (err.message.includes("already in use")) {
        console.log("Program already initialized");
        const configAccount = await program.account.config.fetch(configPda);
        expect(configPda).to.be.not.null;
      } else {
        throw err;
      }
    }
  });

  it("2. Create Round - Successfully creates a round", async () => {
    try {
      const now = Math.floor(Date.now() / 1000);
      const startTime = now + 5; // start in 5 seconds
      const endTime = startTime + 60; // end in 60 seconds after start

      const tx = await program.methods
        .createRound(
          {
            goldPrice: {},
          },
          new anchor.BN(startTime),
          new anchor.BN(endTime)
        )
        .accounts({
          signer: admin.publicKey,
          config: configPda,
          round: round1Pda,
          vault: round1VaultPda,
          mint: tokenMint,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([admin])
        .rpc();

      console.log("Signature", tx);

      // verify
      const roundAccount = await program.account.round.fetch(round1Pda);
      console.log("startTime", roundAccount.startTime.toNumber());
      expect(roundAccount.id.toNumber()).to.equal(1);
    } catch (err) {
      console.error("Error creating round:", err);
      throw err;
    }
  });

  it("3. Start Round - Successfully starts a round", async () => {
    try {
      // wait until now >= start_time
      const latestRound = await program.account.round.fetch(round1Pda);
      const waitSeconds = Math.max(
        0,
        latestRound.startTime.toNumber() - Math.floor(Date.now() / 1000) + 1
      );
      if (waitSeconds > 0) {
        await new Promise((resolve) => setTimeout(resolve, waitSeconds * 1000));
      }

      // Retry loop handle time in cluster vs wall clock
      const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
      let tx: string | null = null;
      const maxAttempts = 20;
      for (let attempt = 1; attempt <= maxAttempts; attempt++) {
        try {
          tx = await program.methods
            .startRound(new anchor.BN(startGoldPrice))
            .accounts({
              signer: keeper.publicKey,
              config: configPda,
              round: round1Pda,
              systemProgram: SystemProgram.programId,
            })
            .signers([keeper])
            .rpc();
          break;
        } catch (err: any) {
          const msg = err?.message || "";
          if (msg.includes("RoundNotReady") && attempt < maxAttempts) {
            await sleep(1000);
            continue;
          }
          throw err;
        }
      }
      if (!tx) throw new Error("Failed to start round after retries");

      console.log("Signature", tx);

      // verify
      const roundAccount = await program.account.round.fetch(round1Pda);
      expect(roundAccount.status).to.deep.equal({ active: {} });
      expect(roundAccount.lockedPrice?.toString()).to.equal(
        new anchor.BN(startGoldPrice).toString()
      );
    } catch (err) {
      console.error("Error starting round:", err);
      throw err;
    }
  });
});
