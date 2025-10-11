import anchorPkg from "@coral-xyz/anchor";
const anchor = anchorPkg;
const { BN } = anchor;
import { PublicKey } from "@solana/web3.js";
import fs from "fs";
import os from "os";
import {
  createAssociatedTokenAccount,
  createMint,
  mintTo,
} from "@solana/spl-token";

async function deployTesting() {
  console.log("Starting deployment...");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.GoldRush;

  // Prepare wallets
  const admin = getKeypairFromFile("/.config/solana/localnet/1.json");
  const keeper = getKeypairFromFile("/.config/solana/localnet/2.json");
  const treasury = getKeypairFromFile("/.config/solana/localnet/3.json");
  const user = getKeypairFromFile("/.config/solana/localnet/4.json");

  // Get airdrops
  await airdropMany(provider.connection, [
    admin.publicKey,
    keeper.publicKey,
    treasury.publicKey,
    user.publicKey,
  ]);

  // Create mint
  const { mint } = await createMintToken(provider.connection, admin, 9);
  console.log("Created mint:", mint.toString());

  // Create ata
  const adminAta = await createAta(provider.connection, mint, admin);
  const keeperAta = await createAta(provider.connection, mint, keeper);
  const treasuryAta = await createAta(provider.connection, mint, treasury);
  const userAta = await createAta(provider.connection, mint, user);

  // Mint to ata
  await Promise.all([
    mintAmount(provider.connection, admin, mint, adminAta, 100_000_000),
    mintAmount(provider.connection, admin, mint, keeperAta, 100_000_000),
    mintAmount(provider.connection, admin, mint, treasuryAta, 100_000_000),
    mintAmount(provider.connection, admin, mint, userAta, 100_000_000),
  ]);

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

deployTesting().catch(console.error);

function getKeypairFromFile(path) {
  const file = fs.readFileSync(os.homedir() + path, "utf-8");
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

async function createAta(connection, mint, owner) {
  return await createAssociatedTokenAccount(
    connection,
    owner,
    mint,
    owner.publicKey
  );
}

async function mintAmount(
  connection,
  mintAuthority,
  mint,
  destination,
  amount
) {
  await mintTo(
    connection,
    mintAuthority,
    mint,
    destination,
    mintAuthority.publicKey,
    amount,
    [mintAuthority]
  );
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
