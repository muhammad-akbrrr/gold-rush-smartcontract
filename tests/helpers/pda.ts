import { PublicKey } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";

export function deriveConfigPda(programId: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    programId
  )[0];
}

export function deriveRoundPda(programId: PublicKey, id: anchor.BN) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("round"), id.toArrayLike(Buffer, "le", 8)],
    programId
  )[0];
}

export function deriveVaultPda(programId: PublicKey, round: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), round.toBuffer()],
    programId
  )[0];
}

export function deriveGroupAssetPda(
  programId: PublicKey,
  round: PublicKey,
  id: anchor.BN
) {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("group_asset"),
      round.toBuffer(),
      id.toArrayLike(Buffer, "le", 8),
    ],
    programId
  )[0];
}

export function deriveBetPda(
  programId: PublicKey,
  round: PublicKey,
  id: bigint
) {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("bet"),
      round.toBuffer(),
      Buffer.from(
        new Uint8Array(new BigInt64Array([BigInt.asIntN(64, id)]).buffer)
      ),
    ],
    programId
  )[0];
}
