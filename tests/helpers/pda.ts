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

export function deriveAssetPda(
  programId: PublicKey,
  groupAsset: PublicKey,
  id: anchor.BN
) {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from("asset"),
      groupAsset.toBuffer(),
      id.toArrayLike(Buffer, "le", 8),
    ],
    programId
  )[0];
}

export function deriveBetPda(
  programId: PublicKey,
  round: PublicKey,
  id: anchor.BN
) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("bet"), round.toBuffer(), id.toArrayLike(Buffer, "le", 8)],
    programId
  )[0];
}
