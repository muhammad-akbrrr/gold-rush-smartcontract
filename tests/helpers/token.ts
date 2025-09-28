import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import {
  createMint,
  createAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";

export async function createMintAndAta(
  connection: Connection,
  payer: Keypair,
  owner: PublicKey,
  decimals = 9
) {
  const mintAuthority = payer; // reuse
  const mint = await createMint(
    connection,
    mintAuthority,
    mintAuthority.publicKey,
    null,
    decimals
  );
  const ata = await createAssociatedTokenAccount(
    connection,
    payer,
    mint,
    owner
  );
  return { mint, ata, mintAuthority };
}

export async function mintAmount(
  connection: Connection,
  mintAuthority: Keypair,
  mint: PublicKey,
  destination: PublicKey,
  amount: number
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
