#!/usr/bin/env bun
import { parseArgs } from "node:util";
import { ALPHABET16, ALPHABET26, ALPHABET36, ALPHABET62, ALT16, make_kid } from "./index.ts";

type Preset = { alphabet: string; time_length?: number; random_length?: number };

const PRESETS: Record<string, Preset> = {
  "16": { alphabet: ALPHABET16, time_length: 12, random_length: 16 },
  "26": { alphabet: ALPHABET26, time_length: 10, random_length: 14 },
  "36": { alphabet: ALPHABET36, time_length: 9, random_length: 13 },
  "62": { alphabet: ALPHABET62, time_length: 8, random_length: 11 },
  alt16: { alphabet: ALT16, time_length: 12, random_length: 16 },
};

const USAGE = `Usage: kid [alphabet] [options]

Print one KID: [prefix] + time part + random part.

Alphabet:
  16       0-9a-f        (12 time + 16 random chars)
  26       a-z           (10 time + 14 random chars, default)
  36       0-9a-z        (9 time + 13 random chars)
  62       0-9A-Za-z     (8 time + 11 random chars)
  alt16    k-z           (12 time + 16 random chars)
  Any other string of 2 or more distinct characters is used as a custom
  alphabet. You must then give -t and -r.

Options:
  -t, --time <n>      Number of time chars
  -r, --random <n>    Number of random chars
  -p, --prefix <s>    String to put before the ID
  -h, --help          Show this help
`;

function fail(msg: string): never {
  process.stderr.write(`kid: ${msg}\n`);
  process.exit(2);
}

function usage_error(msg: string): never {
  process.stderr.write(`kid: ${msg}\n\n${USAGE}`);
  process.exit(2);
}

function parse_length(name: string, value: string | undefined, fallback: number | undefined): number {
  if (value === undefined) {
    if (fallback === undefined) usage_error(`--${name} is required with a custom alphabet`);
    return fallback;
  }
  const n = Number(value);
  if (!Number.isInteger(n) || n < 0) usage_error(`--${name} must be a whole number, got ${value}`);
  return n;
}

export function main(argv: string[]): string {
  const { values, positionals } = parseArgs({
    args: argv,
    allowPositionals: true,
    options: {
      time: { type: "string", short: "t" },
      random: { type: "string", short: "r" },
      prefix: { type: "string", short: "p", default: "" },
      help: { type: "boolean", short: "h" },
    },
  });

  if (values.help) {
    process.stdout.write(USAGE);
    process.exit(0);
  }
  if (positionals.length > 1) usage_error(`expected at most one alphabet, got ${positionals.length}`);

  const name = positionals[0] ?? "26";
  let preset = PRESETS[name];
  if (preset === undefined) {
    if (new Set(name).size !== name.length || name.length < 2) {
      usage_error(
        `alphabet must be one of ${Object.keys(PRESETS).join(", ")} or a string of distinct chars`,
      );
    }
    preset = { alphabet: name };
  }

  const time_length = parse_length("time", values.time, preset.time_length);
  const random_length = parse_length("random", values.random, preset.random_length);
  if (time_length + random_length === 0) usage_error("the ID would be empty");

  return make_kid(preset.alphabet, time_length, random_length)(values.prefix);
}

if (import.meta.main) {
  try {
    console.log(main(process.argv.slice(2)));
  } catch (e) {
    fail(e instanceof Error ? e.message : String(e));
  }
}
