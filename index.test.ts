import { describe, expect, test } from "bun:test";
import { kid } from "./index.ts";

const ALPHABET = "abcdefghijklmnopqrstuvwxyz";

function decodeTime(s: string): number {
  let n = 0;
  for (const c of s) n = n * 26 + ALPHABET.indexOf(c);
  return n;
}

describe("kid", () => {
  test("is 24 lowercase letters with no prefix", () => {
    expect(kid()).toMatch(/^[a-z]{24}$/);
  });

  test("prepends the prefix", () => {
    const id = kid("user_");
    expect(id).toMatch(/^user_[a-z]{24}$/);
  });

  test("encodes the current time in the first 10 letters", () => {
    const before = Date.now();
    const id = kid();
    const after = Date.now();
    const t = decodeTime(id.slice(0, 10));
    expect(t).toBeGreaterThanOrEqual(before);
    expect(t).toBeLessThanOrEqual(after);
  });

  test("sorts by creation time", async () => {
    const a = kid();
    await Bun.sleep(2);
    const b = kid();
    expect(a < b).toBe(true);
  });

  test("random part differs between calls", () => {
    const ids = new Set(Array.from({ length: 1000 }, () => kid()));
    expect(ids.size).toBe(1000);
  });

  test("random part uses every letter", () => {
    const seen = new Set<string>();
    for (let i = 0; i < 200; i++) {
      for (const c of kid().slice(10)) seen.add(c);
    }
    expect(seen.size).toBe(26);
  });
});
