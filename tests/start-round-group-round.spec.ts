import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { airdropMany, getProviderAndProgram } from "./helpers/env";
import { createAta, createMintToken } from "./helpers/token";
import { deriveConfigPda, deriveRoundPda, deriveVaultPda } from "./helpers/pda";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import { PythSolanaReceiver } from "@pythnetwork/pyth-solana-receiver";
import { GOLD_PRICE_FEED_ID, SOL_PRICE_FEED_ID } from "./helpers/pyth";
import { hex32ToBytes } from "./helpers/bytes";

describe("startRoundGroupRound", () => {
  const { provider, program } = getProviderAndProgram();

  let admin: Keypair;
  let treasury: Keypair;
  let keeper: Keypair;

  let tokenMint: PublicKey;
  let configPda: PublicKey;
  let roundPda: PublicKey;
  let priceFeedAccount: PublicKey;

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

    const { mint } = await createMintToken(provider.connection, admin, 9);
    await createAta(provider.connection, mint, admin);
    tokenMint = mint;

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
    const vaultPda = deriveVaultPda(program.programId, roundPda);
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

    priceFeedAccount = pythSolanaReceiver.getPriceFeedAccountAddress(
      0,
      SOL_PRICE_FEED_ID
    );
  });

  it("fails before start time", async () => {
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
    } catch (e: any) {
      const parsed = (anchor as any).AnchorError?.parse?.(e?.logs);
      if (parsed) {
        expect(parsed.error.errorCode.code).to.eq("RoundNotReady");
      }
    }
  });

  it("happy path", async () => {
    const maxWaitMs = 20_000;
    const pollIntervalMs = 500;
    const startWait = Date.now();
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
        }
        throw e;
      }
    }

    const round = await program.account.round.fetch(roundPda);
    expect(round.status).to.deep.equal({ active: {} });
    expect(round.startPrice?.toNumber?.() ?? 0).to.greaterThan(0);
  });

  it("fails unauthorized keeper", async () => {
    let unauthorizedSigner = Keypair.generate();
    await airdropMany(provider.connection, [unauthorizedSigner.publicKey]);

    try {
      await program.methods
        .startRound()
        .accounts({
          signer: unauthorizedSigner.publicKey,
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
        .signers([unauthorizedSigner])
        .rpc();
    } catch (e: any) {
      const parsed = (anchor as any).AnchorError?.parse?.(e?.logs);
      if (parsed) {
        expect(parsed.error.errorCode.code).to.eq("UnauthorizedKeeper");
      }
    }
  });
});
