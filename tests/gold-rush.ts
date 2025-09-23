import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { GoldRush } from "../target/types/gold_rush";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { BN } from "bn.js";
import { expect } from "chai";

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
  let configPda: PublicKey;
  let round1Pda: PublicKey;
  let roundVault1Pda: PublicKey;
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
    ]);

    // derive pdas
    [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );
  });

  it("1. Initialized - Successfully initialized the program", async () => {
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
});
