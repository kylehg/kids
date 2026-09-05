# kids

KIDs (KyleIDs): short, sortable IDs made of a timestamp part and a random part.

```ts
import { kid, kid16, kid36, kid62 } from "kids";

kid();        // "aiorufljxihtlhxntcifnwnb"        a-z, same as kid26
kid("usr_");  // "usr_aiorufljxihtlhxntcifnwnb"
kid16();      // "01a06e1b1b2a2f59d9ba61602246"    0-9a-f
kid36();      // "0mtnembgowal2xk45pvwnz"          0-9a-z
kid62();      // "0VUHkNcwATdiFLsxQdu"             0-9A-Za-z
```

Each ID is `[prefix] + time + random`. The time part is the ms since the Unix
epoch written in the alphabet's base, so IDs made later sort after IDs made
earlier as plain strings. Lengths are chosen so the time part lasts more than
1000 years and the random part holds at least 64 bits.

| Function | Alphabet    | Time chars | Lasts until | Random chars | Bits |
| -------- | ----------- | ---------- | ----------- | ------------ | ---- |
| `kid16`  | `0-9a-f`    | 12         | year 10889  | 16           | 64.0 |
| `kid26`  | `a-z`       | 10         | year 6443   | 14           | 65.8 |
| `kid36`  | `0-9a-z`    | 9          | year 5188   | 13           | 67.2 |
| `kid62`  | `0-9A-Za-z` | 8          | year 8888   | 11           | 65.5 |

To roll your own, use `make_kid(alphabet, time_length, random_length)`, or the
pieces it is built from: `encode_time(ms, alphabet, length)` and
`random_chars(alphabet, length)`. The alphabets are exported as `ALPHABET16`,
`ALPHABET26`, `ALPHABET36`, and `ALPHABET62`. There is also `ALT16`, base 16
written in the letters `k-z`, for hex-shaped IDs with no digits.

## CLI

```sh
bun cli.ts                    # aiorufljxihtlhxntcifnwnb    default kid26
bun cli.ts 16                 # 01a06e1b1b2a2f59d9ba61602246
bun cli.ts alt16              # klukrltsmnmwqmtzloovymwrrton
bun cli.ts 62 -p usr_         # usr_0VUHkNcwATdiFLsxQdu
bun cli.ts 36 -t 9 -r 6       # 0mtnembgoq3x7zk         custom lengths
bun cli.ts xyz -t 30 -r 4     # custom alphabet; -t and -r are required
```

The first argument picks the alphabet: `16`, `26` (default), `36`, `62`, or
`alt16`. Any other string of distinct characters is used as a custom alphabet.

| Option              | Meaning                    |
| ------------------- | -------------------------- |
| `-t, --time <n>`    | Number of time chars       |
| `-r, --random <n>`  | Number of random chars     |
| `-p, --prefix <s>`  | String to put before the ID |
| `-h, --help`        | Show help                  |

To install it as `kid` on your path, run `bun link` in this directory.

## Develop

```sh
bun test
bunx tsc --noEmit
```
