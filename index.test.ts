import { describe, expect, test } from "bun:test";
import { kid, kid16, kid26, kid36, kid62 } from "./index.ts";

const MS_PER_YEAR = 365.2425 * 86_400 * 1000;

const variants = [
  { name: "kid16", fn: kid16, alphabet: "0123456789abcdef", time_length: 12, random_length: 16 },
  { name: "kid26", fn: kid26, alphabet: "abcdefghijklmnopqrstuvwxyz", time_length: 10, random_length: 14 },
  { name: "kid36", fn: kid36, alphabet: "0123456789abcdefghijklmnopqrstuvwxyz", time_length: 9, random_length: 13 },
  {
    name: "kid62",
    fn: kid62,
    alphabet: "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
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
