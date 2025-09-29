import * as anchor from "@coral-xyz/anchor";
import { SystemProgram, PublicKey, Keypair } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { getProviderAndProgram } from "./helpers/env";
import { createAta, createMintToken } from "./helpers/token";
import { deriveConfigPda, deriveVaultPda } from "./helpers/pda";

describe("createRound", () => {
  const { provider, program } = getProviderAndProgram();

  let admin: Keypair;
  let treasury: Keypair;
  let keeper: Keypair;

  let tokenMint: PublicKey;
  let configPda: PublicKey;

  before(async () => {
    admin = (provider.wallet as any).payer as Keypair;
    treasury = Keypair.generate();
    keeper = Keypair.generate();

    const { mint } = await createMintToken(provider.connection, admin, 9);
    await createAta(provider.connection, mint, admin);
    tokenMint = mint;

    configPda = deriveConfigPda(program.programId);

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
  });

  it("group battle happy path", async () => {
    const now = Math.floor(Date.now() / 1000);
    const start = now + 3;
    const end = start + 15;

    const cfg = await program.account.config.fetch(configPda);
    const nextId = cfg.currentRoundCounter.addn(1);
    const roundPda = PublicKey.findProgramAddressSync(
      [Buffer.from("round"), nextId.toArrayLike(Buffer, "le", 8)],
      program.programId
    )[0];
    const vaultPda = deriveVaultPda(program.programId, roundPda);

    try {
      await program.methods
        .createRound(
          { groupBattle: {} },
          new anchor.BN(start),
          new anchor.BN(end)
        )
        .accounts({
          signer: admin.publicKey,
          config: configPda,
          round: roundPda,
          vault: vaultPda,
          mint: tokenMint,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        } as any)
        .signers([admin])
        .rpc();
    } catch (e: any) {
      throw e;
    }

    const round = await program.account.round.fetch(roundPda);
    expect(round.id.toString()).to.eq(nextId.toString());
    expect(round.marketType).to.deep.equal({ groupBattle: {} });
    expect(round.status).to.deep.equal({ scheduled: {} });
    expect(round.vault.toString()).to.eq(vaultPda.toString());
  });

  it("single asset happy path", async () => {
    const now = Math.floor(Date.now() / 1000);
    const start = now + 3;
    const end = start + 15;

    const cfg = await program.account.config.fetch(configPda);
    const nextId = cfg.currentRoundCounter.addn(1);
    const roundPda = PublicKey.findProgramAddressSync(
      [Buffer.from("round"), nextId.toArrayLike(Buffer, "le", 8)],
      program.programId
    )[0];
    const vaultPda = deriveVaultPda(program.programId, roundPda);

    try {
      await program.methods
        .createRound(
          { singleAsset: {} },
          new anchor.BN(start),
          new anchor.BN(end)
        )
        .accounts({
          signer: admin.publicKey,
          config: configPda,
          round: roundPda,
          vault: vaultPda,
          mint: tokenMint,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        } as any)
        .signers([admin])
        .rpc();
    } catch (e: any) {
      throw e;
    }

    const round = await program.account.round.fetch(roundPda);
    expect(round.id.toString()).to.eq(nextId.toString());
    expect(round.marketType).to.deep.equal({ singleAsset: {} });
    expect(round.status).to.deep.equal({ scheduled: {} });
    expect(round.vault.toString()).to.eq(vaultPda.toString());
  });

  it("fails invalid timestamps", async () => {
    const now = Math.floor(Date.now() / 1000);
    const start = now - 3;
    const end = start + 15;

    const cfg = await program.account.config.fetch(configPda);
    const nextId = cfg.currentRoundCounter.addn(1);
    const roundPda = PublicKey.findProgramAddressSync(
      [Buffer.from("round"), nextId.toArrayLike(Buffer, "le", 8)],
      program.programId
    )[0];
    const vaultPda = deriveVaultPda(program.programId, roundPda);

    try {
      await program.methods
        .createRound(
          { groupBattle: {} },
          new anchor.BN(start),
          new anchor.BN(end)
        )
        .accounts({
          signer: admin.publicKey,
          config: configPda,
          round: roundPda,
          vault: vaultPda,
          mint: tokenMint,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        } as any)
        .signers([admin])
        .rpc();

      throw new Error("should fail");
    } catch (e: any) {
      const parsed = (anchor as any).AnchorError?.parse?.(e?.logs);
      if (parsed) {
        expect(parsed.error.errorCode.code).to.eq("InvalidTimestamps");
      }
    }
  });
});
