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

describe("startRound", () => {
  const { provider, program } = getProviderAndProgram();

  let admin: Keypair;
  let treasury: Keypair;
  let keeper: Keypair;

  let tokenMint: PublicKey;
  let configPda: PublicKey;

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
  });

  it("happy path single asset", async () => {
    const now = Math.floor(Date.now() / 1000);
    const start = now + 3;
    const end = start + 15;

    const cfg = await program.account.config.fetch(configPda);
    const nextRoundId = cfg.currentRoundCounter.addn(1);
    const roundPda = deriveRoundPda(program.programId, nextRoundId);
    const vaultPda = deriveVaultPda(program.programId, roundPda);

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

    const SOL_PRICE_FEED_ID =
      "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d";
    const priceFeedAccount = pythSolanaReceiver.getPriceFeedAccountAddress(
      0,
      SOL_PRICE_FEED_ID
    );

    const maxWaitMs = 20000;
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
    expect(round.id.toString()).to.eq(nextRoundId.toString());
    expect(round.status).to.deep.equal({ active: {} });
    expect(round.startPrice?.toNumber?.() ?? 0).to.greaterThan(0);
  });
  it("happy path group battle", async () => {});
  it("fails before start time");
});
