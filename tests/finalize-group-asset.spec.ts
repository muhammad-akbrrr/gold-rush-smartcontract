import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { airdropMany, getProviderAndProgram } from "./helpers/env";
import { createAta, createMintToken, mintAmount } from "./helpers/token";
import {
  deriveBetPda,
  deriveConfigPda,
  deriveGroupAssetPda,
  deriveAssetPda,
  deriveRoundPda,
  deriveVaultPda,
} from "./helpers/pda";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { expect } from "chai";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";
import { GOLD_PRICE_FEED_ID, SOL_PRICE_FEED_ID } from "./helpers/pyth";
import { hex32ToBytes, stringToBytes } from "./helpers/bytes";

describe("finalizeGroupAsset", () => {
  const { provider, program } = getProviderAndProgram();

  let admin: Keypair;
  let treasury: Keypair;
  let keeper: Keypair;
  let user: Keypair;

  let userTokenAccount: PublicKey;
  let priceFeedAccount: PublicKey;
  let tokenMint: PublicKey;
  let configPda: PublicKey;
  let roundPda: PublicKey;
  let vaultPda: PublicKey;
  let treasuryTokenAccount: PublicKey;
  let betPda: PublicKey;

  let pythSolanaReceiver: PythSolanaReceiver;

  before(async () => {
    admin = (provider.wallet as any).payer as Keypair;
    treasury = Keypair.generate();
    keeper = Keypair.generate();
    user = Keypair.generate();

    pythSolanaReceiver = new PythSolanaReceiver({
      connection: provider.connection,
      wallet: new anchor.Wallet(admin),
    });

    await airdropMany(provider.connection, [
      admin.publicKey,
      treasury.publicKey,
      keeper.publicKey,
      user.publicKey,
    ]);

    const { mint } = await createMintToken(provider.connection, admin, 9);
    tokenMint = mint;
    await createAta(provider.connection, mint, admin);
    treasuryTokenAccount = await createAta(provider.connection, mint, treasury);
    userTokenAccount = await createAta(provider.connection, mint, user);

    await mintAmount(
      provider.connection,
      admin,
      tokenMint,
      userTokenAccount,
      100_000_000
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

    // capture start price

    // start round
    priceFeedAccount = pythSolanaReceiver.getPriceFeedAccountAddress(
      0,
      SOL_PRICE_FEED_ID
    );
    const maxWaitMs = 20000;
    const pollIntervalMs = 500;
    const startWait = Date.now();
    const maxPythErrorIterations = 10;
    let pythErrorIterations = 0;
    while (true) {
      try {
        await program.methods
          .startRound()
          .accounts({
            signer: keeper.publicKey,
            config: configPda,
            round: roundPda,
            priceUpdate: priceFeedAccount,
            systemProgram: SystemProgram.programId,
          } as any)
          .signers([keeper])
          .rpc();
        break;
      } catch (e: any) {
        const parsed = (anchor as any).AnchorError?.parse?.(e?.logs);
        const code = parsed?.error?.errorCode?.code;
        if (code === "RoundNotReadyForStart") {
          if (Date.now() - startWait > maxWaitMs) {
            throw new Error("Timed out waiting for round to be ready");
          }
          await new Promise((r) => setTimeout(r, pollIntervalMs));
          continue;
        } else if (code === "PythError") {
          pythErrorIterations++;
          if (pythErrorIterations >= maxPythErrorIterations) {
            throw new Error("Timed out waiting for pyth error");
          }
          await new Promise((r) => setTimeout(r, pollIntervalMs));
          continue;
        }
        throw e;
      }
    }

    // place bet
    const amount = new anchor.BN(10_000_000); // 10 GRT
    const direction = { up: {} };
    const r = await program.account.round.fetch(roundPda);
    const nextBetId = r.totalBets.addn(1);
    betPda = deriveBetPda(program.programId, roundPda, nextBetId);
    await program.methods
      .placeBet(amount, direction)
      .accounts({
        signer: user.publicKey,
        config: configPda,
        round: roundPda,
        groupAsset: groupAsset1Pda,
        bet: betPda,
        vault: vaultPda,
        tokenAccount: userTokenAccount,
        mint: tokenMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      } as any)
      .signers([user])
      .rpc();
  });

  it("fails before end time");

  it("happy path");

  it("fails group asset already finalized");
});
