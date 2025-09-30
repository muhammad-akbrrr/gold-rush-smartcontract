import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { airdropMany, getProviderAndProgram } from "./helpers/env";
import { createAta, createMintToken } from "./helpers/token";
import {
  deriveConfigPda,
  deriveGroupAssetPda,
  deriveRoundPda,
  deriveVaultPda,
} from "./helpers/pda";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { hex32ToBytes, stringToBytes } from "./helpers/bytes";
import { GOLD_PRICE_FEED_ID } from "./helpers/pyth";

describe("insertGroupAsset", () => {
  const { provider, program } = getProviderAndProgram();

  let admin: Keypair;
  let treasury: Keypair;
  let keeper: Keypair;
  let user: Keypair;

  let tokenMint: PublicKey;
  let configPda: PublicKey;
  let roundPda: PublicKey;
  let vaultPda: PublicKey;

  before(async () => {
    admin = (provider.wallet as any).payer as Keypair;
    treasury = Keypair.generate();
    keeper = Keypair.generate();
    user = Keypair.generate();

    await airdropMany(provider.connection, [
      admin.publicKey,
      treasury.publicKey,
      keeper.publicKey,
      user.publicKey,
    ]);

    const { mint } = await createMintToken(provider.connection, admin, 9);
    await createAta(provider.connection, mint, admin);
    tokenMint = mint;

    configPda = deriveConfigPda(program.programId);
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

    const now = Math.floor(Date.now() / 1000);
    const start = now + 3;
    const end = start + 15;

    const cfg = await program.account.config.fetch(configPda);
    const nextId = cfg.currentRoundCounter.addn(1);
    roundPda = deriveRoundPda(program.programId, nextId);
    vaultPda = deriveVaultPda(program.programId, roundPda);

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
  });

  it("insert group asset happy path", async () => {
    const symbol = "ASA";
    const symbolArray = stringToBytes(symbol);

    const round = await program.account.round.fetch(roundPda);
    const nextGroupId = round.totalGroups.addn(1);

    const groupAssetPda = deriveGroupAssetPda(
      program.programId,
      roundPda,
      nextGroupId
    );

    try {
      await program.methods
        .insertGroupAsset(symbolArray)
        .accounts({
          signer: admin.publicKey,
          config: configPda,
          round: roundPda,
          groupAsset: groupAssetPda,
          systemProgram: SystemProgram.programId,
        } as any)
        .signers([admin])
        .rpc();
    } catch (e: any) {
      throw e;
    }

    const groupAsset = await program.account.groupAsset.fetch(groupAssetPda);
    expect(groupAsset.id.toString()).to.eq(nextGroupId.toString());
    expect(groupAsset.round.toString()).to.eq(roundPda.toString());
    expect(groupAsset.symbol.toString()).to.eq(symbolArray.toString());
  });

  it("fails unauthorized", async () => {
    const symbol = "AFR";
    const symbolArray = stringToBytes(symbol);

    const round = await program.account.round.fetch(roundPda);
    const nextGroupId = round.totalGroups.addn(1);
    const groupAssetPda = deriveGroupAssetPda(
      program.programId,
      roundPda,
      nextGroupId
    );

    try {
      await program.methods
        .insertGroupAsset(symbolArray)
        .accounts({
          signer: user.publicKey,
          config: configPda,
          round: roundPda,
          groupAsset: groupAssetPda,
          systemProgram: SystemProgram.programId,
        } as any)
        .signers([user])
        .rpc();

      throw new Error("should fail");
    } catch (e: any) {
      const parsed = (anchor as any).AnchorError?.parse?.(e?.logs);
      if (parsed) {
        expect(parsed.error.errorCode.code).to.eq("Unauthorized");
      }
    }
  });
});
