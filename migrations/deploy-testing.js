import anchorPkg from "@coral-xyz/anchor";
const anchor = anchorPkg;
const { BN } = anchor;
import { PublicKey, Transaction } from "@solana/web3.js";
import fs from "fs";
import os from "os";
import {
  getOrCreateAssociatedTokenAccount,
  createMint,
  createMintToInstruction,
} from "@solana/spl-token";

async function deployTesting() {
  console.log("Starting deployment...");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.GoldRush;

  // Prepare wallets
  const admin = getKeypairFromFile("wallets/1.json");
  const keeper = getKeypairFromFile("wallets/1.json"); // Same as admin for simplicity
  const treasury = getKeypairFromFile("wallets/1.json"); // Same as admin for simplicity
  const user = getKeypairFromFile("wallets/2.json");

  // if localnet, do airdrop
  if (provider.connection.rpcEndpoint.includes("127.0.0.1")) {
    await airdropMany(provider.connection, [
      admin.publicKey,
      keeper.publicKey,
      treasury.publicKey,
      user.publicKey,
    ]);
    return;
  }

  // Create mint
  const { mint } = await createMintToken(provider.connection, admin, 9);
  console.log("Created mint:", mint.toString());

  // Create ata
  const adminAta = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    admin, // payer
    mint,
    admin.publicKey // owner
  );
  const keeperAta = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    keeper, // payer
    mint,
    keeper.publicKey // owner
  );
  const treasuryAta = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    treasury, // payer
    mint,
    treasury.publicKey // owner
  );
  const userAta = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    user, // payer
    mint,
    user.publicKey // owner
  );

  // Mint to ATAs in batch
  await mintToAllATAsBatch(provider.connection, admin, mint, [
    { ata: adminAta.address, amount: 100_000_000 },
    { ata: keeperAta.address, amount: 100_000_000 },
    { ata: treasuryAta.address, amount: 100_000_000 },
    { ata: userAta.address, amount: 100_000_000 },
  ]);

  // Initialize config
  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    program.programId
  );
  const goldPriceFeedId = hex32ToBytes(
    "0x765d2ba906dbc32ca17cc11f5310a89e9ee1f6420508c63861f2f8ba4ee34bb2"
  );
  const maxPriceUpdateAgeSecs = new BN(6000); // 100 minutes
  const feeSingleAssetBps = 2_000; // 2%
  const feeGroupBattleBps = 2_500; // 2.5%
  const minBetAmount = new BN(1_000_000); // 1 Token
  const betCutoffWindowSecs = new BN(10); // 10 seconds
  const minTimeFactorBps = new BN(1_000); // 1%
  const maxTimeFactorBps = new BN(2_000); // 2%
  const defaultDirectionFactorBps = new BN(1_000); // 1%
  const keepers = [keeper.publicKey];

  try {
    const tx = await program.methods
      .initialize(
        keepers,
        mint,
        treasury.publicKey,
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
        signer: admin.publicKey,
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

async function mintToAllATAsBatch(
  connection,
  mintAuthority,
  mint,
  destinations
) {
  console.log("Minting to all ATAs in batch...");

  const transaction = new Transaction();

  for (const dest of destinations) {
    const instruction = createMintToInstruction(
      mint, // mint
      dest.ata, // destination
      mintAuthority.publicKey, // authority
      dest.amount // amount
    );
    transaction.add(instruction);
  }

  const signature = await connection.sendTransaction(transaction, [
    mintAuthority,
  ]);
  await connection.confirmTransaction(signature);
  console.log("All minting completed in batch:", signature);
}

function getKeypairFromFile(path) {
  const file = fs.readFileSync(path, "utf-8");
  return anchor.web3.Keypair.fromSecretKey(Uint8Array.from(JSON.parse(file)));
}

async function airdropMany(connection, pubs, lamports = 100_000_000_000) {
  await Promise.all(
    pubs.map(async (pk) =>
      connection.confirmTransaction(
        await connection.requestAirdrop(pk, lamports)
      )
    )
  );
}

async function createMintToken(connection, payer, decimals = 9) {
  const mintAuthority = payer;
  const mint = await createMint(
    connection,
    mintAuthority,
    mintAuthority.publicKey,
    null,
    decimals
  );
  return { mint, mintAuthority };
}

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

deployTesting().catch(console.error);
