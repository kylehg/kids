// Each KID is: [prefix] + time part + random part.
//
// The time part is the ms since the Unix epoch written in the alphabet's
// base, padded on the left with the first letter. IDs made later sort after
// IDs made earlier as plain strings, because every alphabet is in ASCII order.
//
// Lengths are chosen so that the time part lasts more than 1000 years from
// now and the random part holds at least 64 bits of randomness:
//
//   base  time  lasts until  random  bits
//   16    12    year 10889   16      64.0
//   26    10    year 6443    14      65.8
//   36    9     year 5188    13      67.2
//   62    8     year 8888    11      65.5

type KidFn = (prefix?: string) => string;

function make_kid(alphabet: string, time_length: number, random_length: number): KidFn {
  const base = alphabet.length;
  // Bytes at or above this are thrown away so every character has the same
  // chance of being picked.
  const max_unbiased_byte = 256 - (256 % base);
  const buf = new Uint8Array(random_length * 2);

  function encode_time(ms: number): string {
    let n = ms;
    let out = "";
    while (n > 0) {
      out = alphabet[n % base] + out;
      n = Math.floor(n / base);
    }
    return out.padStart(time_length, alphabet[0]);
  }

  function random_chars(): string {
    let out = "";
    while (out.length < random_length) {
      crypto.getRandomValues(buf);
      for (const byte of buf) {
        if (byte >= max_unbiased_byte) continue;
        out += alphabet[byte % base];
        if (out.length === random_length) break;
      }
    }
    return out;
  }

  return (prefix = "") => prefix + encode_time(Date.now()) + random_chars();
}

/** Hex KID: 12 time chars + 16 random chars, `0-9a-f`. */
export const kid16: KidFn = make_kid("0123456789abcdef", 12, 16);

/** Letters-only KID: 10 time chars + 14 random chars, `a-z`. */
export const kid26: KidFn = make_kid("abcdefghijklmnopqrstuvwxyz", 10, 14);

/** Lowercase alphanumeric KID: 9 time chars + 13 random chars, `0-9a-z`. */
export const kid36: KidFn = make_kid("0123456789abcdefghijklmnopqrstuvwxyz", 9, 13);

/** Mixed-case alphanumeric KID: 8 time chars + 11 random chars, `0-9A-Za-z`. */
export const kid62: KidFn = make_kid(
  "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
  8,
  11,
);

/** The default KID (KyleID). Same as {@link kid26}. */
export const kid: KidFn = kid26;
