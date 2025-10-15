import * as anchor from "@coral-xyz/anchor";
import { SystemProgram, Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { getProviderAndProgram } from "./helpers/env";
import { createAta, createMintToken } from "./helpers/token";
import { deriveConfigPda } from "./helpers/pda";
import { hex32ToBytes } from "./helpers/bytes";
import { GOLD_PRICE_FEED_ID } from "./helpers/pyth";

describe("emergencyPause", () => {
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

    // initialize program
    const feedId = hex32ToBytes(GOLD_PRICE_FEED_ID);
    try {
      await program.methods
        .initialize(
          [keeper.publicKey],
          tokenMint,
          treasury.publicKey,
          feedId,
          new anchor.BN(120),
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
  });

  it("happy path", async () => {
    try {
      await program.methods
        .emergencyPause()
        .accounts({
          signer: admin.publicKey,
          config: configPda,
        } as any)
        .signers([admin])
        .rpc();
    } catch (e: any) {
      throw e;
    }

    const cfg = await program.account.config.fetch(configPda);
    expect(cfg.status).to.deep.equal({ emergencyPaused: {} });
  });

  it("fails if already emergency paused", async () => {
    try {
      await program.methods
        .emergencyPause()
        .accounts({
          signer: admin.publicKey,
          config: configPda,
        } as any)
        .signers([admin])
        .rpc();

      throw new Error("should fail");
    } catch (e: any) {
      const parsed = (anchor as any).AnchorError?.parse?.(e?.logs);
      if (parsed) {
        expect(parsed.error.errorCode.code).to.eq("AlreadyEmergencyPaused");
      }
    }
  });
});
