import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { airdropMany, getProviderAndProgram } from "./helpers/env";
import { createMintAndAta } from "./helpers/token";
import {
  deriveAssetPda,
  deriveConfigPda,
  deriveGroupAssetPda,
  deriveRoundPda,
  deriveVaultPda,
} from "./helpers/pda";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { symbolToBytes } from "./helpers/asset";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";

describe("insertAsset", () => {
  const { provider, program } = getProviderAndProgram();

  let admin: Keypair;
  let treasury: Keypair;
  let keeper: Keypair;

  let tokenMint: PublicKey;
  let configPda: PublicKey;
  let roundPda: PublicKey;
  let vaultPda: PublicKey;
  let groupAssetPda: PublicKey;

  let pythSolanaReceiver: PythSolanaReceiver;

  before(async () => {
    admin = (provider.wallet as any).payer as Keypair;
    treasury = Keypair.generate();
    keeper = Keypair.generate();

    pythSolanaReceiver = new PythSolanaReceiver({
      connection: provider.connection,
      wallet: new anchor.Wallet(admin),
    });

    await airdropMany(provider.connection, [
      admin.publicKey,
      treasury.publicKey,
      keeper.publicKey,
    ]);

    const { mint } = await createMintAndAta(
      provider.connection,
      admin,
      admin.publicKey,
      9
    );
    tokenMint = mint;

    configPda = deriveConfigPda(program.programId);
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

    const symbolArray = symbolToBytes("ASA");
    const round = await program.account.round.fetch(roundPda);
    const nextGroupId = round.totalGroups.addn(1);

    groupAssetPda = deriveGroupAssetPda(
      program.programId,
      roundPda,
      nextGroupId
    );
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
  });

  it("insert asset happy path", async () => {
    const symbol = symbolToBytes("2899.HK");
    const ga = await program.account.groupAsset.fetch(groupAssetPda);
    const nextAssetId = ga.totalAssets.addn(1);
    const assetPda = deriveAssetPda(
      program.programId,
      groupAssetPda,
      nextAssetId
    );

    const SOL_PRICE_FEED_ID =
      "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
    const priceFeedAccount = pythSolanaReceiver.getPriceFeedAccountAddress(
      0,
      SOL_PRICE_FEED_ID
    );

    await program.methods
      .insertAsset(symbol)
      .accounts({
        signer: admin.publicKey,
        config: configPda,
        round: roundPda,
        groupAsset: groupAssetPda,
        asset: assetPda,
        feedPriceAccount: priceFeedAccount,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([admin])
      .rpc();

    const asset = await program.account.asset.fetch(assetPda);
    expect(asset.id.toString()).to.eq(nextAssetId.toString());
    expect(asset.group.toString()).to.eq(groupAssetPda.toString());
    expect(asset.round.toString()).to.eq(roundPda.toString());
    expect(asset.symbol.toString()).to.eq(symbol.toString());
    expect(asset.priceFeedAccount.toString()).to.eq(
      priceFeedAccount.toString()
    );

    const updatedGroupAsset = await program.account.groupAsset.fetch(
      groupAssetPda
    );
    expect(updatedGroupAsset.totalAssets.toString()).to.eq(
      nextAssetId.toString()
    );
  });
  it("fails max assets reached", async () => {
    const SOL_PRICE_FEED_ID =
      "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
    const priceFeedAccount = pythSolanaReceiver.getPriceFeedAccountAddress(
      0,
      SOL_PRICE_FEED_ID
    );

    // Happy path already inserted 1 asset. Insert 9 more to reach the max (10)
    for (let i = 0; i < 9; i++) {
      const ga = await program.account.groupAsset.fetch(groupAssetPda);
      const nextAssetId = ga.totalAssets.addn(1);
      const assetPda = deriveAssetPda(
        program.programId,
        groupAssetPda,
        nextAssetId
      );

      await program.methods
        .insertAsset(symbolToBytes(`S${i}`))
        .accounts({
          signer: admin.publicKey,
          config: configPda,
          round: roundPda,
          groupAsset: groupAssetPda,
          asset: assetPda,
          feedPriceAccount: priceFeedAccount,
          systemProgram: SystemProgram.programId,
        } as any)
        .signers([admin])
        .rpc();
    }

    // Next insert should fail with MaxAssetsReached
    try {
      const ga = await program.account.groupAsset.fetch(groupAssetPda);
      const nextAssetId = ga.totalAssets.addn(1);
      const assetPda = deriveAssetPda(
        program.programId,
        groupAssetPda,
        nextAssetId
      );

      await program.methods
        .insertAsset(symbolToBytes("X"))
        .accounts({
          signer: admin.publicKey,
          config: configPda,
          round: roundPda,
          groupAsset: groupAssetPda,
          asset: assetPda,
          feedPriceAccount: priceFeedAccount,
          systemProgram: SystemProgram.programId,
        } as any)
        .signers([admin])
        .rpc();

      throw new Error("should fail");
    } catch (e: any) {
      const parsed = (anchor as any).AnchorError?.parse?.(e?.logs);
      if (parsed) {
        expect(parsed.error.errorCode.code).to.eq("MaxAssetsReached");
      }
    }
  });
});
