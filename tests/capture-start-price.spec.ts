import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { airdropMany, getProviderAndProgram } from "./helpers/env";
import { createAta, createMintToken, mintAmount } from "./helpers/token";
import {
  deriveConfigPda,
  deriveGroupAssetPda,
  deriveAssetPda,
  deriveRoundPda,
  deriveVaultPda,
} from "./helpers/pda";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";
import { GOLD_PRICE_FEED_ID } from "./helpers/pyth";
import { hex32ToBytes, stringToBytes } from "./helpers/bytes";

describe("captureStartPrice", () => {
  const { provider, program } = getProviderAndProgram();

  let admin: Keypair;
  let treasury: Keypair;
  let keeper: Keypair;
  let user: Keypair;

  let priceFeedAccount: PublicKey;
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

    // create mint
    const { mint } = await createMintToken(provider.connection, admin, 9);
    tokenMint = mint;
    await createAta(provider.connection, mint, admin);

    // create price feed account
    const pythSolanaReceiver = new PythSolanaReceiver({
      connection: provider.connection,
      wallet: new anchor.Wallet(admin),
    });
    priceFeedAccount = pythSolanaReceiver.getPriceFeedAccountAddress(
      0,
      GOLD_PRICE_FEED_ID
    );

    // initialize config
    configPda = deriveConfigPda(program.programId);
    const feedId = hex32ToBytes(GOLD_PRICE_FEED_ID);
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

    // create round
    const now = Math.floor(Date.now() / 1000);
    const start = now + 3;
    const end = start + 15;
    const cfg = await program.account.config.fetch(configPda);
    const nextRoundId = cfg.currentRoundCounter.addn(1);
    roundPda = deriveRoundPda(program.programId, nextRoundId);
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

    // insert group asset
    const groupAssetPdas = [];
    for (let i = 0; i < 3; i++) {
      const symbol = stringToBytes(`ASA ${i}`);
      const round = await program.account.round.fetch(roundPda);
      const nextGroupId = round.totalGroups.addn(1);
      const groupAssetPda = deriveGroupAssetPda(
        program.programId,
        roundPda,
        nextGroupId
      );
      try {
        await program.methods
          .insertGroupAsset(symbol)
          .accounts({
            signer: admin.publicKey,
            config: configPda,
            round: roundPda,
            groupAsset: groupAssetPda,
            systemProgram: SystemProgram.programId,
          } as any)
          .signers([admin])
          .rpc();

        groupAssetPdas.push(groupAssetPda);
      } catch (e: any) {
        throw e;
      }
    }

    // insert asset
    for (const groupAssetPda of groupAssetPdas) {
      for (let i = 0; i < 9; i++) {
        const ga = await program.account.groupAsset.fetch(groupAssetPda);
        const nextAssetId = ga.totalAssets.addn(1);
        const assetPda = deriveAssetPda(
          program.programId,
          groupAssetPda,
          nextAssetId
        );

        await program.methods
          .insertAsset(stringToBytes(`S${i}`))
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
    }
  });

  it("happy path with multiple assets (paired remaining accounts)", async () => {
    const r = await program.account.round.fetch(roundPda);
    for (let groupId = 1; groupId <= r.totalGroups.toNumber(); groupId++) {
      const groupAssetPda = deriveGroupAssetPda(
        program.programId,
        roundPda,
        new anchor.BN(groupId)
      );
      const g = await program.account.groupAsset.fetch(groupAssetPda);
      let remainingAccounts = [];
      for (let assetId = 1; assetId <= g.totalAssets.toNumber(); assetId++) {
        const assetPda = deriveAssetPda(
          program.programId,
          groupAssetPda,
          new anchor.BN(assetId)
        );
        remainingAccounts.push({
          pubkey: assetPda,
          isSigner: false,
          isWritable: true,
        });
        remainingAccounts.push({
          pubkey: priceFeedAccount,
          isSigner: false,
          isWritable: false,
        });
      }

      try {
        await program.methods
          .captureStartPrice()
          .accounts({
            signer: keeper.publicKey,
            config: configPda,
            round: roundPda,
            groupAsset: groupAssetPda,
          } as any)
          .remainingAccounts(remainingAccounts)
          .signers([keeper])
          .rpc();
      } catch (e: any) {
        throw e;
      }
    }

    const round = await program.account.round.fetch(roundPda);
    for (let groupId = 1; groupId <= round.totalGroups.toNumber(); groupId++) {
      const groupAssetPda = deriveGroupAssetPda(
        program.programId,
        roundPda,
        new anchor.BN(groupId)
      );
      const group = await program.account.groupAsset.fetch(groupAssetPda);
      expect(group.startPriceAt).to.not.be.null;

      for (
        let assetId = 1;
        assetId <= group.totalAssets.toNumber();
        assetId++
      ) {
        const assetPda = deriveAssetPda(
          program.programId,
          groupAssetPda,
          new anchor.BN(assetId)
        );

        const asset = await program.account.asset.fetch(assetPda);
        expect(asset.startPrice).to.not.be.null;
      }
    }
  });

  it("fails invalid remaining accounts length", async () => {
    const r = await program.account.round.fetch(roundPda);
    for (let groupId = 1; groupId <= r.totalGroups.toNumber(); groupId++) {
      const groupAssetPda = deriveGroupAssetPda(
        program.programId,
        roundPda,
        new anchor.BN(groupId)
      );
      try {
        await program.methods
          .captureStartPrice()
          .accounts({
            signer: keeper.publicKey,
            config: configPda,
            round: roundPda,
            groupAsset: groupAssetPda,
          } as any)
          .remainingAccounts([
            {
              pubkey: priceFeedAccount,
              isSigner: false,
              isWritable: false,
            },
          ])
          .signers([keeper])
          .rpc();
      } catch (e: any) {
        const parsed = (anchor as any).AnchorError?.parse?.(e?.logs);
        if (parsed) {
          expect(parsed.error.errorCode.code).to.eq(
            "InvalidRemainingAccountsLength"
          );
        }
      }
    }
  });
});
