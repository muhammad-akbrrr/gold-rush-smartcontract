import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { GoldRush } from "../target/types/gold_rush";
import {
  AccountMeta,
  Keypair,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import {
  createMint,
  createAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createAssociatedTokenAccount,
} from "@solana/spl-token";
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

    // Mint tokens
    console.log("Minting tokens to users...");
    const mintAmount = 1000_000_000_000; // 1000 GRT

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
      [Buffer.from("round"), new anchor.BN(1).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    [round1VaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), round1Pda.toBuffer()],
      program.programId
    );
  });

  it("Initialize - Successfully initializes the program", async () => {
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

  it("Create Round - Successfully creates a round", async () => {
    try {
      const now = Math.floor(Date.now() / 1000);
      const startTime = now + 5; // start in 5 seconds
      const endTime = startTime + 20; // end in 20 seconds after start

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
      expect(roundAccount.id.toNumber()).to.equal(1);
    } catch (err) {
      console.error("Error creating round:", err);
      throw err;
    }
  });

  it("Start Round - Successfully starts a round", async () => {
    try {
      const startPrice = 3_787_630;

      // Wait for 10s
      console.log("Waiting for 10 seconds until start time reached...");
      await new Promise((resolve) => setTimeout(resolve, 10000));

      // Retry loop handle time in cluster vs wall clock
      const tx = await program.methods
        .startRound(new anchor.BN(startPrice))
        .accounts({
          signer: keeper.publicKey,
          config: configPda,
          round: round1Pda,
          systemProgram: SystemProgram.programId,
        })
        .signers([keeper])
        .rpc();

      console.log("Signature", tx);

      // verify
      const roundAccount = await program.account.round.fetch(round1Pda);
      expect(roundAccount.status).to.deep.equal({ active: {} });
      expect(roundAccount.lockedPrice?.toString()).to.equal(
        new anchor.BN(startPrice).toString()
      );
    } catch (err) {
      console.error("Error starting round:", err);
      throw err;
    }
  });

  it("Place Bet (Round 1 - Up Winner 1) - Successfully places a bet", async () => {
    try {
      const amount = new anchor.BN(10_000_000); // 10 GRT
      const direction = { up: {} };

      const roundAccount = await program.account.round.fetch(round1Pda);
      const nextBetId = roundAccount.totalBets.addn(1);
      [betWinner1Pda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("bet"),
          round1Pda.toBuffer(),
          bettorWin1.publicKey.toBuffer(),
          nextBetId.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

      const tx = await program.methods
        .placeBet(amount, direction)
        .accounts({
          signer: bettorWin1.publicKey,
          config: configPda,
          round: round1Pda,
          bet: betWinner1Pda,
          vault: round1VaultPda,
          tokenAccount: bettorWin1TokenAccount,
          mint: tokenMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([bettorWin1])
        .rpc();

      console.log("Signature", tx);

      // verify
      const betAccount = await program.account.bet.fetch(betWinner1Pda);
      expect(betAccount.amount.toString()).to.eq(amount.toString());
      expect(betAccount.bettor.toString()).to.equal(
        bettorWin1.publicKey.toString()
      );
    } catch (err) {
      console.error("Error placing bet:", err);
      throw err;
    }
  });

  it("Place Bet (Round 1 - Percentage Winner 2) - Successfully places a bet", async () => {
    try {
      const amount = new anchor.BN(15_000_000); // 15 GRT
      const direction = {
        percentageChangeBps: { 0: 10 },
      };

      const roundAccount = await program.account.round.fetch(round1Pda);
      const nextBetId = roundAccount.totalBets.addn(1);
      [betWinner2Pda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("bet"),
          round1Pda.toBuffer(),
          bettorWin2.publicKey.toBuffer(),
          nextBetId.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

      const tx = await program.methods
        .placeBet(amount, direction)
        .accounts({
          signer: bettorWin2.publicKey,
          config: configPda,
          round: round1Pda,
          bet: betWinner2Pda,
          vault: round1VaultPda,
          tokenAccount: bettorWin2TokenAccount,
          mint: tokenMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([bettorWin2])
        .rpc();

      console.log("Signature", tx);

      // verify
      const betAccount = await program.account.bet.fetch(betWinner2Pda);
      expect(betAccount.amount.toString()).to.eq(amount.toString());
      expect(betAccount.bettor.toString()).to.equal(
        bettorWin2.publicKey.toString()
      );
    } catch (err) {
      console.error("Error placing bet:", err);
      throw err;
    }
  });

  it("Place Bet (Round 1 - Down Lost 1) - Successfully places a bet", async () => {
    try {
      const amount = new anchor.BN(20_000_000); // 20 GRT
      const direction = { down: {} };

      const roundAccount = await program.account.round.fetch(round1Pda);
      const nextBetId = roundAccount.totalBets.addn(1);
      [betLoser1Pda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("bet"),
          round1Pda.toBuffer(),
          bettorLost1.publicKey.toBuffer(),
          nextBetId.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

      const tx = await program.methods
        .placeBet(amount, direction)
        .accounts({
          signer: bettorLost1.publicKey,
          config: configPda,
          round: round1Pda,
          bet: betLoser1Pda,
          vault: round1VaultPda,
          tokenAccount: bettorLost1TokenAccount,
          mint: tokenMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([bettorLost1])
        .rpc();

      console.log("Signature", tx);

      // verify
      const betAccount = await program.account.bet.fetch(betLoser1Pda);
      expect(betAccount.amount.toString()).to.eq(amount.toString());
      expect(betAccount.bettor.toString()).to.equal(
        bettorLost1.publicKey.toString()
      );
    } catch (err) {
      console.error("Error placing bet:", err);
      throw err;
    }
  });

  it("Settle Round - Successfully settles a round", async () => {
    try {
      const endPrice = new anchor.BN(3_900_000);
      const round = await program.account.round.fetch(round1Pda);
      let remainingAccounts: Array<AccountMeta> = [];

      // TODO: Dynamic remaning accounts from total bet
      // for (let i = 1; i <= round.totalBets.toNumber(); i++) {
      //   const [betPda] = PublicKey.findProgramAddressSync(
      //     [
      //       Buffer.from("bet"),
      //       round1Pda.toBuffer(),
      //       bettorWin1.publicKey.toBuffer(),
      //       new anchor.BN(i).toArrayLike(Buffer, "le", 8),
      //     ],
      //     program.programId
      //   );
      //   remainingAccounts.push({
      //     pubkey: betPda,
      //     isSigner: false,
      //     isWritable: true,
      //   });
      // }

      const [betPd1a] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("bet"),
          round1Pda.toBuffer(),
          bettorWin1.publicKey.toBuffer(),
          new anchor.BN(1).toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );
      remainingAccounts.push({
        pubkey: betPd1a,
        isSigner: false,
        isWritable: true,
      });

      const [betPd2a] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("bet"),
          round1Pda.toBuffer(),
          bettorWin2.publicKey.toBuffer(),
          new anchor.BN(2).toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );
      remainingAccounts.push({
        pubkey: betPd2a,
        isSigner: false,
        isWritable: true,
      });

      const [betPd3a] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("bet"),
          round1Pda.toBuffer(),
          bettorLost1.publicKey.toBuffer(),
          new anchor.BN(3).toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );
      remainingAccounts.push({
        pubkey: betPd3a,
        isSigner: false,
        isWritable: true,
      });

      console.log(
        "Remaining acounts generated with total: ",
        remainingAccounts.length
      );

      // Wait for 20s
      console.log("Waiting for 20 seconds until end time reached...");
      await new Promise((resolve) => setTimeout(resolve, 20000));

      const tx = await program.methods
        .settleRound(endPrice)
        .accounts({
          signer: keeper.publicKey,
          config: configPda,
          round: round1Pda,
          roundVault: round1VaultPda,
          treasury: treasury.publicKey,
          treasuryTokenAccount: treasuryTokenAccount,
          mint: tokenMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts(remainingAccounts)
        .signers([keeper])
        .rpc();

      console.log("Signature", tx);

      // verify
      const betAccount = await program.account.bet.fetch(betWinner1Pda);
      expect(betAccount.status).to.deep.equal({ won: {} });
    } catch (err) {
      console.error("Error settling round:", err);
      throw err;
    }
  });
});
