# kids (Rust)

Rust version of [KIDs](../README.md). Needs Rust 1.99 (edition 2024); the
pinned toolchain in `rust-toolchain.toml` is installed by rustup on first
use.

```rust
use std::time::{Duration, UNIX_EPOCH};

use kids::{ALPHABET36, Generator, Options, Precision, kid, kid16, kid36, kid62};

kid("");       // "aiorufljxihtlhxntcifnwnb"        a-z, same as kid26
kid("usr_");   // "usr_aiorufljxihtlhxntcifnwnb"
kid16("");     // "01a06e1b1b2a2f59d9ba61602246"    0-9a-f
kid36("");     // "0mtnembgowal2xk45pvwnz"          0-9a-z
kid62("");     // "0VUHkNcwATdiFLsxQdu"             0-9A-Za-z

let short = Generator::new(ALPHABET36, 6, 10, Options {
    epoch: UNIX_EPOCH + Duration::from_millis(1_704_067_200_000), // 2024-01-01
    precision: Precision::Second,
});
let id = short.generate("")?; // 6 base-36 seconds last ~69 years from 2024
```

- `Generator::new(alphabet, time_length, random_length, options)` builds a
  generator. `Options::default()` is the Unix epoch and millisecond
  precision. `generate(prefix)` returns a `kids::Error` when the current
  time is before the epoch or does not fit. The preset functions (`kid`,
  `kid16`, ...) panic only if the clock is outside their range.
- `encode_time(n, alphabet, length)` and `random_chars(alphabet, length)`
  are the pieces a `Generator` is built from. Random characters come from
  `rand::rng()`, a cryptographically secure generator.
- The alphabets are `ALPHABET16`, `ALPHABET26`, `ALPHABET36`, `ALPHABET62`,
  and `ALT16`.

To use it from another crate, add to `Cargo.toml`:

```toml
kids = { git = "https://github.com/kylehg/kids" }
```

## CLI

```sh
cargo run -- 62 -p usr_   # usr_0VUHkNcwATdiFLsxQdu
```

To install it as `kid` on your path:

```sh
cargo install --git https://github.com/kylehg/kids kids
```

See the [top-level README](../README.md#cli) for the options. The help text
is clap's, and an `--epoch` date with no offset is read as UTC.

## Develop

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```
