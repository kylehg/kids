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
`ALPHABET26`, `ALPHABET36`, and `ALPHABET62`.

## Develop

```sh
bun test
bunx tsc --noEmit
```
