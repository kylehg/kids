# kids

KIDs (KyleIDs): short, sortable IDs made of a timestamp part and a random part.

```
aiorufljxihtlhxntcifnwnb        kid26 (the default), a-z
usr_aiorufljxihtlhxntcifnwnb    with a prefix
01a06e1b1b2a2f59d9ba61602246    kid16, 0-9a-f
0mtnembgowal2xk45pvwnz          kid36, 0-9a-z
0VUHkNcwATdiFLsxQdu             kid62, 0-9A-Za-z
```

The same library and `kid` CLI come in four languages:

| Language                 | Directory | Toolchain         |
| ------------------------ | --------- | ----------------- |
| [TypeScript](ts/)        | `ts/`     | Bun               |
| [Python](python/)        | `python/` | Python 3.15, uv   |
| [Go](go/)                | `go/`     | Go 1.27           |
| [Rust](rust/)            | `rust/`   | Rust 1.99, Cargo  |

Each directory's README shows that language's API, how to install it, and
how to run its tests.

## How it works

Each ID is `[prefix] + time + random`. The time part is the ms since the Unix
epoch written in the alphabet's base, so IDs made later sort after IDs made
earlier as plain strings. Lengths are chosen so the time part lasts more than
1000 years and the random part holds at least 64 bits.

| Preset  | Alphabet    | Time chars | Lasts until | Random chars | Bits |
| ------- | ----------- | ---------- | ----------- | ------------ | ---- |
| `kid16` | `0-9a-f`    | 12         | year 10889  | 16           | 64.0 |
| `kid26` | `a-z`       | 10         | year 6443   | 14           | 65.8 |
| `kid36` | `0-9a-z`    | 9          | year 5188   | 13           | 67.2 |
| `kid62` | `0-9A-Za-z` | 8          | year 8888   | 11           | 65.5 |

There is also `ALT16`, base 16 written in the letters `k-z`, for hex-shaped
IDs with no digits.

To roll your own, give an alphabet, a number of time chars, and a number of
random chars. A time length of 0 drops the time part, so the ID is pure
random and does not sort by creation time. The options are:

- epoch: time zero for the time part. Default the Unix epoch. A later epoch
  makes the same number of time chars last longer. Making an ID before the
  epoch is an error.
- precision: ms (default) or sec. With sec the time part counts whole
  seconds, so it lasts 1000 times longer but IDs made within the same second
  do not sort by creation order.

For example, 6 base-36 chars of seconds from 2024 last about 69 years.

## CLI

Every language builds the same `kid` command:

```sh
kid                    # aiorufljxihtlhxntcifnwnb    default kid26
kid 16                 # 01a06e1b1b2a2f59d9ba61602246
kid alt16              # klukrltsmnmwqmtzloovymwrrton
kid 62 -p usr_         # usr_0VUHkNcwATdiFLsxQdu
kid 36 -t 9 -r 6       # 0mtnembgoq3x7zk         custom lengths
kid xyz -t 30 -r 4     # custom alphabet; -t and -r are required
```

The first argument picks the alphabet: `16`, `26` (default), `36`, `62`, or
`alt16`. Any other string of distinct characters is used as a custom alphabet.

| Option               | Meaning                                              |
| -------------------- | ---------------------------------------------------- |
| `-t, --time <n>`     | Number of time chars. 0 drops the time part.         |
| `-r, --random <n>`   | Number of random chars                               |
| `-p, --prefix <s>`   | String to put before the ID                          |
| `-e, --epoch <when>` | Time zero, as an ISO date or ms since the Unix epoch |
| `--precision <u>`    | `ms` (default) or `sec`                              |
| `-h, --help`         | Show help                                            |

Bad arguments exit with status 2.

## Calculator

Live at <https://kids.kyle.fyi>, or open `public/index.html` in a browser. It
is one file with no dependencies. For a given alphabet, part lengths, epoch,
and precision it shows a sample ID, the date the time part lasts until, and
how many IDs it takes for the random part to reach a chosen collision risk
(birthday bound). The configuration is kept in the URL hash so you can share
a link. Deploy with `npx wrangler deploy`.

## Develop

CI runs each language's tests, linters, and formatters on every push and
pull request; see `.github/workflows/ci.yml`. To run them yourself, see each
language's README.

## why

i have weirdly strong biases about ids. if they're db pks i want them to insert
in-order. if they're in a url i want them to be selectable all in one
double-click (no base64 `-`). when spoken it's helpful if they're case
insensitive. i also find the mixed casing (and mixed height of numbers and
letters) aesthetically displeasing. i often want them prefixed but not always. i
want them to be short as possible but no shorter. for core entities that are
frequently viewed/referenced i want them to be shorter than a credit card
number, though care less about ordering/insert speed if they're minted less
often. i vary all these preferences by the specific use-case and my mood.

so i made a little utility to encode my personal preferences and vary them as i
wish.
