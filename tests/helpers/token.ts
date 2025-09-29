import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import {
  createMint,
  createAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";

export async function createMintToken(
  connection: Connection,
  payer: Keypair,
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
  return { mint, mintAuthority };
}

export async function createAta(
  connection: Connection,
  mint: PublicKey,
  owner: Keypair
) {
  return await createAssociatedTokenAccount(
    connection,
    owner,
    mint,
    owner.publicKey
  );
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
