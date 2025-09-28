export function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

export async function waitFor<T>(
  fn: () => Promise<T>,
  { timeoutMs = 15000, intervalMs = 500 } = {}
) {
  const end = Date.now() + timeoutMs;
  let lastErr: any;
  while (Date.now() < end) {
    try {
      return await fn();
    } catch (e) {
      lastErr = e;
      await sleep(intervalMs);
    }
  }
  throw lastErr ?? new Error("waitFor timeout");
}
