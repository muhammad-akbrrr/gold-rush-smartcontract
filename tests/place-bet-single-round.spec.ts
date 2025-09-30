import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { airdropMany, getProviderAndProgram } from "./helpers/env";
import { createAta, createMintToken, mintAmount } from "./helpers/token";
import {
  deriveBetPda,
  deriveConfigPda,
  deriveRoundPda,
  deriveVaultPda,
} from "./helpers/pda";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";
import { GOLD_PRICE_FEED_ID, SOL_PRICE_FEED_ID } from "./helpers/pyth";
import { hex32ToBytes } from "./helpers/bytes";

describe("placeBetSingleRound", () => {
  const { provider, program } = getProviderAndProgram();

  let admin: Keypair;
  let treasury: Keypair;
  let keeper: Keypair;
  let user: Keypair;

  let tokenMint: PublicKey;
  let configPda: PublicKey;
  let roundPda: PublicKey;
  let vaultPda: PublicKey;
  let userTokenAccount: PublicKey;

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
    userTokenAccount = await createAta(provider.connection, mint, user);

    await mintAmount(
      provider.connection,
      admin,
      tokenMint,
      userTokenAccount,
      100_000_000
    );

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

    const now = Math.floor(Date.now() / 1000);
    const start = now + 3;
    const end = start + 15;
    const cfg = await program.account.config.fetch(configPda);
    const nextRoundId = cfg.currentRoundCounter.addn(1);
    roundPda = deriveRoundPda(program.programId, nextRoundId);
    vaultPda = deriveVaultPda(program.programId, roundPda);

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

    const priceFeedAccount = pythSolanaReceiver.getPriceFeedAccountAddress(
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
            systemProgram: SystemProgram.programId,
          } as any)
          .remainingAccounts([
            {
              pubkey: priceFeedAccount,
              isSigner: false,
              isWritable: true,
            },
          ])
          .signers([keeper])
          .rpc();
        break;
      } catch (e: any) {
        const parsed = (anchor as any).AnchorError?.parse?.(e?.logs);
        const code = parsed?.error?.errorCode?.code;
        if (code === "RoundNotReady") {
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
  });

  it("happy path up", async () => {
    const amount = new anchor.BN(10_000_000); // 10 GRT
    const direction = { up: {} };
    const r = await program.account.round.fetch(roundPda);
    const nextBetId = r.totalBets.addn(1);
    const betPda = deriveBetPda(program.programId, roundPda, nextBetId);
    try {
      await program.methods
        .placeBet(amount, direction)
        .accounts({
          signer: user.publicKey,
          config: configPda,
          round: roundPda,
          groupAsset: null,
          bet: betPda,
          vault: vaultPda,
          tokenAccount: userTokenAccount,
          mint: tokenMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        } as any)
        .signers([user])
        .rpc();
    } catch (e: any) {
      throw e;
    }
    const round = await program.account.round.fetch(roundPda);
    const bet = await program.account.bet.fetch(betPda);
    expect(round.totalBets.toString()).to.eq(bet.id.toString());
    expect(bet.status).to.deep.equal({ pending: {} });
    expect(bet.amount.toNumber()).to.eq(amount.toNumber());
    expect(bet.direction).to.deep.equal(direction);
  });
  it("happy path down", async () => {
    const amount = new anchor.BN(10_000_000); // 10 GRT
    const direction = { down: {} };
    const r = await program.account.round.fetch(roundPda);
    const nextBetId = r.totalBets.addn(1);
    const betPda = deriveBetPda(program.programId, roundPda, nextBetId);
    try {
      await program.methods
        .placeBet(amount, direction)
        .accounts({
          signer: user.publicKey,
          config: configPda,
          round: roundPda,
          groupAsset: null,
          bet: betPda,
          vault: vaultPda,
          tokenAccount: userTokenAccount,
          mint: tokenMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        } as any)
        .signers([user])
        .rpc();
    } catch (e: any) {
      throw e;
    }
    const round = await program.account.round.fetch(roundPda);
    const bet = await program.account.bet.fetch(betPda);
    expect(round.totalBets.toString()).to.eq(bet.id.toString());
    expect(bet.status).to.deep.equal({ pending: {} });
    expect(bet.amount.toNumber()).to.eq(amount.toNumber());
    expect(bet.direction).to.deep.equal(direction);
  });

  it("happy path percentage", async () => {
    const amount = new anchor.BN(10_000_000); // 10 GRT
    const direction = { percentageChangeBps: { 0: 10 } };
    const r = await program.account.round.fetch(roundPda);
    const nextBetId = r.totalBets.addn(1);
    const betPda = deriveBetPda(program.programId, roundPda, nextBetId);
    try {
      await program.methods
        .placeBet(amount, direction)
        .accounts({
          signer: user.publicKey,
          config: configPda,
          round: roundPda,
          groupAsset: null,
          bet: betPda,
          vault: vaultPda,
          tokenAccount: userTokenAccount,
          mint: tokenMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        } as any)
        .signers([user])
        .rpc();
    } catch (e: any) {
      throw e;
    }
    const round = await program.account.round.fetch(roundPda);
    const bet = await program.account.bet.fetch(betPda);
    expect(round.totalBets.toString()).to.eq(bet.id.toString());
    expect(bet.status).to.deep.equal({ pending: {} });
    expect(bet.amount.toNumber()).to.eq(amount.toNumber());
    expect(bet.direction).to.deep.equal(direction);
  });

  it("fails below min amount", async () => {
    const cfg = await program.account.config.fetch(configPda);
    const amount = cfg.minBetAmount.subn(1);
    const direction = { up: {} };
    const r = await program.account.round.fetch(roundPda);
    const nextBetId = r.totalBets.addn(1);
    const betPda = deriveBetPda(program.programId, roundPda, nextBetId);
    try {
      await program.methods
        .placeBet(amount, direction)
        .accounts({
          signer: user.publicKey,
          config: configPda,
          round: roundPda,
          groupAsset: null,
          bet: betPda,
          vault: vaultPda,
          tokenAccount: userTokenAccount,
          mint: tokenMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        } as any)
        .signers([user])
        .rpc();

      throw new Error("should fail");
    } catch (e: any) {
      const parsed = (anchor as any).AnchorError?.parse?.(e?.logs);
      if (parsed) {
        expect(parsed.error.errorCode.code).to.eq("BetBelowMinimum");
      }
    }
  });
});
