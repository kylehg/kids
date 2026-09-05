import { describe, expect, test } from "bun:test";
import {
  ALPHABET16,
  ALPHABET26,
  ALPHABET36,
  ALPHABET62,
  ALT16,
  encode_time,
  kid,
  kid16,
  kid26,
  kid36,
  kid62,
  random_chars,
} from "./index.ts";

const MS_PER_YEAR = 365.2425 * 86_400 * 1000;

const variants = [
  { name: "kid16", fn: kid16, alphabet: ALPHABET16, time_length: 12, random_length: 16 },
  { name: "kid26", fn: kid26, alphabet: ALPHABET26, time_length: 10, random_length: 14 },
  { name: "kid36", fn: kid36, alphabet: ALPHABET36, time_length: 9, random_length: 13 },
  {
    name: "kid62",
    fn: kid62,
    alphabet: ALPHABET62,
    time_length: 8,
    random_length: 11,
  },
];

function decode_time(s: string, alphabet: string): number {
  let n = 0;
  for (const c of s) n = n * alphabet.length + alphabet.indexOf(c);
  return n;
}

test("kid is kid26", () => {
  expect(kid).toBe(kid26);
});

describe("encode_time", () => {
  test("writes a number in the alphabet's base, left-padded", () => {
    expect(encode_time(0, ALPHABET16, 4)).toBe("0000");
    expect(encode_time(255, ALPHABET16, 4)).toBe("00ff");
    expect(encode_time(26, ALPHABET26, 3)).toBe("aba");
    expect(encode_time(61, ALPHABET62, 2)).toBe("0z");
  });

  test("throws when the number does not fit", () => {
    expect(() => encode_time(256, ALPHABET16, 2)).toThrow(RangeError);
  });
});

describe("random_chars", () => {
  test("returns the requested length from the alphabet", () => {
    const s = random_chars("xyz", 50);
    expect(s).toHaveLength(50);
    expect(s).toMatch(/^[xyz]{50}$/);
  });

  test("returns an empty string for length 0", () => {
    expect(random_chars(ALPHABET16, 0)).toBe("");
  });
});

describe.each(variants)("$name", ({ fn, alphabet, time_length, random_length }) => {
  const base = alphabet.length;
  const total = time_length + random_length;

  test("alphabet is in ASCII order so IDs sort as strings", () => {
    expect([...alphabet].sort().join("")).toBe(alphabet);
  });

  test("time part lasts more than 1000 years from now", () => {
    expect(base ** time_length).toBeGreaterThan(Date.now() + 1000 * MS_PER_YEAR);
  });

  test("random part has at least 64 bits", () => {
    expect(random_length * Math.log2(base)).toBeGreaterThanOrEqual(64);
  });

  test("has the expected length and only alphabet characters", () => {
    const id = fn();
    expect(id).toHaveLength(total);
    for (const c of id) expect(alphabet).toContain(c);
  });

  test("prepends the prefix", () => {
    const id = fn("user_");
    expect(id.startsWith("user_")).toBe(true);
    expect(id).toHaveLength(5 + total);
  });

  test("encodes the current time in the time part", () => {
    const before = Date.now();
    const id = fn();
    const after = Date.now();
    const t = decode_time(id.slice(0, time_length), alphabet);
    expect(t).toBeGreaterThanOrEqual(before);
    expect(t).toBeLessThanOrEqual(after);
  });

  test("sorts by creation time", async () => {
    const a = fn();
    await Bun.sleep(2);
    const b = fn();
    expect(a < b).toBe(true);
  });

  test("random part differs between calls", () => {
    const ids = new Set(Array.from({ length: 1000 }, () => fn()));
    expect(ids.size).toBe(1000);
  });

  test("random part uses every character", () => {
    const seen = new Set<string>();
    for (let i = 0; i < 500; i++) {
      for (const c of fn().slice(time_length)) seen.add(c);
    }
    expect(seen.size).toBe(base);
  });
});

describe("ALT16", () => {
  test("is base 16, letters k through z, in ASCII order", () => {
    expect(ALT16).toHaveLength(16);
    expect(ALT16).toBe("klmnopqrstuvwxyz");
    expect([...ALT16].sort().join("")).toBe(ALT16);
  });
});

describe("cli", () => {
  function run(...args: string[]) {
    const r = Bun.spawnSync(["bun", "cli.ts", ...args], { cwd: import.meta.dir });
    return { code: r.exitCode, out: r.stdout.toString(), err: r.stderr.toString() };
  }

  test("prints a kid26 by default", () => {
    const r = run();
    expect(r.code).toBe(0);
    expect(r.out).toMatch(/^[a-z]{24}\n$/);
  });

  test("picks the alphabet from the first arg", () => {
    expect(run("16").out).toMatch(/^[0-9a-f]{28}\n$/);
    expect(run("36").out).toMatch(/^[0-9a-z]{22}\n$/);
    expect(run("62").out).toMatch(/^[0-9A-Za-z]{19}\n$/);
    expect(run("alt16").out).toMatch(/^[k-z]{28}\n$/);
  });

  test("takes prefix and part lengths", () => {
    expect(run("62", "-p", "usr_", "-t", "9", "-r", "3").out).toMatch(/^usr_[0-9A-Za-z]{12}\n$/);
    expect(run("--prefix", "x", "--random", "0").out).toMatch(/^x[a-z]{10}\n$/);
  });

  test("takes a custom alphabet with explicit lengths", () => {
    expect(run("xyz", "-t", "30", "-r", "4").out).toMatch(/^[xyz]{34}\n$/);
  });

  test("rejects bad input", () => {
    expect(run("99").code).toBe(2);
    expect(run("xyz").err).toContain("--time is required");
    expect(run("-r", "abc").err).toContain("--random must be a whole number");
    expect(run("16", "-t", "2").err).toContain("does not fit");
    expect(run("-t", "0", "-r", "0").err).toContain("empty");
  });

  test("prints help", () => {
    const r = run("--help");
    expect(r.code).toBe(0);
    expect(r.out).toContain("Usage: kid");
  });
});
