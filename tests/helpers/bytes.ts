export function stringToBytes(symbol: string) {
  const symbolBytes = new TextEncoder().encode(symbol);
  const symbolArray = new Array(8).fill(0);
  for (let i = 0; i < symbolBytes.length; i++) {
    symbolArray[i] = symbolBytes[i];
  }
  return symbolArray;
}

export function hexToBytes(hex: string): number[] {
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex;
  if (clean.length % 2 !== 0) throw new Error("Hex length must be even");
  const bytes: number[] = [];
  for (let i = 0; i < clean.length; i += 2) {
    bytes.push(parseInt(clean.slice(i, i + 2), 16));
  }
  return bytes;
}

// opsional: enforce 32 byte (FeedId)
export function hex32ToBytes(hex: string): number[] {
  const bytes = hexToBytes(hex);
  if (bytes.length !== 32) throw new Error("Must be exactly 32 bytes");
  return bytes;
}
