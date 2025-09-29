import * as anchor from "@coral-xyz/anchor";
import { SystemProgram, Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { getProviderAndProgram } from "./helpers/env";
import { createAta, createMintToken } from "./helpers/token";
import { deriveConfigPda } from "./helpers/pda";

describe("initialize", () => {
  const { provider, program } = getProviderAndProgram();

  let admin: Keypair;
  let keeper: Keypair;
  let treasury: Keypair;
  let tokenMint: PublicKey;
  let configPda: PublicKey;

  before(async () => {
    admin = (provider.wallet as any).payer as Keypair;
    keeper = Keypair.generate();
    treasury = Keypair.generate();

    const { mint } = await createMintToken(provider.connection, admin, 9);
    tokenMint = mint;
    await createAta(provider.connection, mint, admin);
    configPda = deriveConfigPda(program.programId);
  });

  it("fails with invalid params", async () => {
    // invalid: empty keepers
    const badKeeperList: PublicKey[] = [];
    const badMint = tokenMint; // still valid, focus on keepers here
    const badTreasury = treasury.publicKey;
    try {
      await program.methods
        .initialize(
          badKeeperList,
          badMint,
          badTreasury,
          2_000,
          2_500,
          new anchor.BN(10_000_000),
          new anchor.BN(10),
          1_000,
          2_000,
          1_000
        )
        .accounts({
          signer: admin.publicKey,
          config: deriveConfigPda(program.programId),
          systemProgram: SystemProgram.programId,
        } as any)
        .signers([admin])
        .rpc();
      throw new Error("should fail");
    } catch (e: any) {
      const parsed = (anchor as any).AnchorError?.parse?.(e?.logs);
      if (parsed) {
        expect(parsed.error.errorCode.code).to.eq("InvalidKeeperAuthorities");
      } else {
        const msg = e?.error?.errorMessage || e?.message || "";
        expect(msg).to.match(/InvalidKeeperAuthorities|invalid keeper/i);
      }
    }
  });

  it("happy path", async () => {
    try {
      await program.methods
        .initialize(
          [keeper.publicKey],
          tokenMint,
          treasury.publicKey,
          2_000,
          2_500,
          new anchor.BN(10_000_000),
          new anchor.BN(10),
          1_000,
          2_000,
          1_000
        )
        .accounts({
          signer: admin.publicKey,
          config: configPda,
          systemProgram: SystemProgram.programId,
        } as any)
        .signers([admin])
        .rpc();
    } catch (e: any) {
      const msg = e?.error?.errorMessage || e?.message || "";
      if (!/already/i.test(msg)) throw e;
    }

    const cfg = await program.account.config.fetch(configPda);
    expect(cfg.admin.toString()).to.eq(admin.publicKey.toString());
    expect(
      cfg.keeperAuthorities.map((k: PublicKey) => k.toString())
    ).to.include(keeper.publicKey.toString());
    expect(cfg.tokenMint.toString()).to.eq(tokenMint.toString());
    expect(cfg.treasury.toString()).to.eq(treasury.publicKey.toString());
    expect(cfg.minBetAmount.toString()).to.eq("10000000");
  });

  it("fails if already initialized", async () => {
    try {
      await program.methods
        .initialize(
          [keeper.publicKey],
          tokenMint,
          treasury.publicKey,
          2_000,
          2_500,
          new anchor.BN(10_000_000),
          new anchor.BN(10),
          1_000,
          2_000,
          1_000
        )
        .accounts({
          signer: admin.publicKey,
          config: configPda,
          systemProgram: SystemProgram.programId,
        } as any)
        .signers([admin])
        .rpc();
      throw new Error("should fail");
    } catch (e: any) {
      const parsed = (anchor as any).AnchorError?.parse?.(e?.logs);
      if (parsed) {
        expect(parsed.error.errorCode.code).to.eq("AlreadyInitialized");
      } else {
        const msg = e?.error?.errorMessage || e?.message || "";
        expect(msg).to.match(/already/i);
      }
    }
  });
});
