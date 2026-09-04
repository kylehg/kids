const ALPHABET = "abcdefghijklmnopqrstuvwxyz";
const BASE = 26;
const TIME_LENGTH = 10;
const RANDOM_LENGTH = 14;

// Largest multiple of 26 that fits in a byte. Bytes at or above this are
// thrown away so every letter has the same chance of being picked.
const MAX_UNBIASED_BYTE = 234;

function encodeTime(ms: number): string {
  let n = ms;
  let out = "";
  while (n > 0) {
    out = ALPHABET[n % BASE] + out;
    n = Math.floor(n / BASE);
  }
  return out.padStart(TIME_LENGTH, ALPHABET[0]);
}

function randomLetters(length: number): string {
  let out = "";
  const buf = new Uint8Array(length * 2);
  while (out.length < length) {
    crypto.getRandomValues(buf);
    for (const byte of buf) {
      if (byte >= MAX_UNBIASED_BYTE) continue;
      out += ALPHABET[byte % BASE];
      if (out.length === length) break;
    }
  }
  return out;
}

/**
 * Make a KID (KyleID): a lowercase, letters-only ID.
 *
 * Layout: `[prefix]` + 10 letters of base-26 timestamp (ms since the Unix
 * epoch) + 14 random letters. IDs made later sort after IDs made earlier.
 */
export function kid(prefix = ""): string {
  return prefix + encodeTime(Date.now()) + randomLetters(RANDOM_LENGTH);
}
