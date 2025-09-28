export function symbolToBytes(symbol: string) {
  const symbolBytes = new TextEncoder().encode(symbol);
  const symbolArray = new Array(8).fill(0);
  for (let i = 0; i < symbolBytes.length; i++) {
    symbolArray[i] = symbolBytes[i];
  }
  return symbolArray;
}
