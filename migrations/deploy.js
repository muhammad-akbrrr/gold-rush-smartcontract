// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

async function deploy() {
  console.log("Starting deployment...");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.GoldRush;
  const admin = provider.wallet.publicKey;
  const keepers = [admin];
  const treasury = admin;

  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    program.programId
  );

  const tokenMint = new PublicKey(
    "EJwZgeZrdC8TXTQbQBoL6bfuAnFUUy1PVCMB4DYPzVaS"
  );
  const goldPriceFeedId = hex32ToBytes(
    "0x765d2ba906dbc32ca17cc11f5310a89e9ee1f6420508c63861f2f8ba4ee34bb2"
  );
  const maxPriceUpdateAgeSecs = new anchor.BN(120); // 2 minutes
  const feeSingleAssetBps = 2_000; // 2%
  const feeGroupBattleBps = 2_500; // 2.5%
  const minBetAmount = new anchor.BN(10_000_000); // 10 Token
  const betCutoffWindowSecs = new anchor.BN(10); // 10 seconds
  const minTimeFactorBps = new anchor.BN(1_000); // 1%
  const maxTimeFactorBps = new anchor.BN(2_000); // 2%
  const defaultDirectionFactorBps = new anchor.BN(1_000); // 1%

  try {
    await program.methods
      .initialize(
        keepers,
        tokenMint,
        treasury,
        goldPriceFeedId,
        maxPriceUpdateAgeSecs,
        feeSingleAssetBps,
        feeGroupBattleBps,
        minBetAmount,
        betCutoffWindowSecs,
        minTimeFactorBps,
        maxTimeFactorBps,
        defaultDirectionFactorBps
      )
      .accounts({
        signer: admin,
        config: configPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    console.log("Initialize successful:", tx);
    console.log("Config PDA:", configPda.toString());
  } catch (err) {
    if (err.message.includes("already in use")) {
      console.log("Config already initialized - deployment complete");
    } else {
      console.error("Initialize failed:", err);
      throw err;
    }
  }
}

deploy().catch(console.error);

function hex32ToBytes(hex) {
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex;
  if (clean.length % 2 !== 0) throw new Error("Hex length must be even");
  const bytes = [];
  for (let i = 0; i < clean.length; i += 2) {
    bytes.push(parseInt(clean.slice(i, i + 2), 16));
  }

  if (bytes.length !== 32) throw new Error("Must be exactly 32 bytes");
  return bytes;
}
