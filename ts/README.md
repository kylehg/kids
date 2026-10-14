# kids (TypeScript)

TypeScript version of [KIDs](../README.md), run with [Bun](https://bun.sh).

```ts
import { kid, kid16, kid36, kid62, make_kid, ALPHABET36 } from "kids";

kid();        // "aiorufljxihtlhxntcifnwnb"        a-z, same as kid26
kid("usr_");  // "usr_aiorufljxihtlhxntcifnwnb"
kid16();      // "01a06e1b1b2a2f59d9ba61602246"    0-9a-f
kid36();      // "0mtnembgowal2xk45pvwnz"          0-9a-z
kid62();      // "0VUHkNcwATdiFLsxQdu"             0-9A-Za-z

const short = make_kid(ALPHABET36, 6, 10, { epoch: Date.UTC(2024, 0, 1), precision: "sec" });
short(); // 6 base-36 seconds last ~69 years from 2024
```

- `make_kid(alphabet, time_length, random_length, options?)` returns a
  `(prefix?) => string`. The options are `epoch` (ms since the Unix epoch or
  a `Date`, default 0) and `precision` (`"ms"` or `"sec"`). It throws a
  `RangeError` for an invalid epoch, and the function it returns throws when
  the current time is before the epoch or does not fit.
- `encode_time(ms, alphabet, length)` and `random_chars(alphabet, length)`
  are the pieces `make_kid` is built from.
- The alphabets are `ALPHABET16`, `ALPHABET26`, `ALPHABET36`, `ALPHABET62`,
  and `ALT16`.

## CLI

```sh
bun cli.ts 62 -p usr_   # usr_0VUHkNcwATdiFLsxQdu
```

To install it as `kid` on your path, run `bun link` in this directory. See
the [top-level README](../README.md#cli) for the options.

## Develop

```sh
bun install
bun test
bunx tsc --noEmit
```
