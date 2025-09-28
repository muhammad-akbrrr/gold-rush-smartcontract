import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { GoldRush } from "../../target/types/gold_rush";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";

export function getProviderAndProgram() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.goldRush as Program<GoldRush>;
  return { provider, program };
}

export async function airdropMany(
  connection: Connection,
  pubs: PublicKey[],
  lamports: number
) {
  await Promise.all(
    pubs.map(async (pk) =>
      connection.confirmTransaction(
        await connection.requestAirdrop(pk, lamports)
      )
    )
  );
}
