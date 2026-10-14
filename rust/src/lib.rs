//! KIDs (KyleIDs): short, sortable IDs made of a timestamp part and a random
//! part.
//!
//! Each KID is: `[prefix] + time part + random part`.
//!
//! The time part is the ms since the Unix epoch written in the alphabet's
//! base, padded on the left with the first character. IDs made later sort
//! after IDs made earlier as plain strings, because every alphabet is in
//! ASCII order.
//!
//! Lengths are chosen so that the time part lasts more than 1000 years from
//! now and the random part holds at least 64 bits of randomness:
//!
//! | base | time | lasts until | random | bits |
//! | ---- | ---- | ----------- | ------ | ---- |
//! | 16   | 12   | year 10889  | 16     | 64.0 |
//! | 26   | 10   | year 6443   | 14     | 65.8 |
//! | 36   | 9    | year 5188   | 13     | 67.2 |
//! | 62   | 8    | year 8888   | 11     | 65.5 |
//!
//! ```
//! let id = kids::kid("usr_");
//! assert_eq!(id.len(), 4 + 24);
//! ```

use std::fmt;
use std::sync::LazyLock;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::seq::IndexedRandom;

pub const ALPHABET16: &str = "0123456789abcdef";
pub const ALPHABET26: &str = "abcdefghijklmnopqrstuvwxyz";
pub const ALPHABET36: &str = "0123456789abcdefghijklmnopqrstuvwxyz";
pub const ALPHABET62: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
/// Base 16 in letters only, `k-z`. Same lengths as [`ALPHABET16`].
pub const ALT16: &str = "klmnopqrstuvwxyz";

/// Unit of the time part.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Precision {
    #[default]
    Millisecond,
    Second,
}

/// Options for a [`Generator`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    /// Time zero for the time part. Default [`UNIX_EPOCH`]. A later epoch
    /// makes the time part last longer. Making an ID before the epoch is an
    /// error.
    pub epoch: SystemTime,
    /// Unit of the time part. Default [`Precision::Millisecond`].
    pub precision: Precision,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            epoch: UNIX_EPOCH,
            precision: Precision::Millisecond,
        }
    }
}

/// Why a KID could not be made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The number does not fit in the time part.
    DoesNotFit { n: u64, length: usize, base: usize },
    /// The current time is before the epoch, given in ms since the Unix epoch.
    BeforeEpoch { epoch_ms: i64 },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DoesNotFit { n, length, base } => {
                write!(f, "{n} does not fit in {length} base-{base} chars")
            }
            Self::BeforeEpoch { epoch_ms } => {
                write!(f, "current time is before the epoch {epoch_ms}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Write `n` in the base of `alphabet`, padded on the left with the first
/// character to `length`. Returns an error if `n` does not fit.
pub fn encode_time(n: u64, alphabet: &str, length: usize) -> Result<String, Error> {
    encode(n, &alphabet.chars().collect::<Vec<_>>(), length)
}

fn encode(n: u64, digits: &[char], length: usize) -> Result<String, Error> {
    let base = digits.len() as u64;
    let mut out = Vec::with_capacity(length);
    let mut m = n;
    while m > 0 {
        out.push(digits[(m % base) as usize]);
        m /= base;
    }
    if out.len() > length {
        return Err(Error::DoesNotFit {
            n,
            length,
            base: digits.len(),
        });
    }
    out.resize(length, digits[0]);
    Ok(out.iter().rev().collect())
}

/// Return `length` characters picked uniformly from `alphabet`.
pub fn random_chars(alphabet: &str, length: usize) -> String {
    random(&alphabet.chars().collect::<Vec<_>>(), length)
}

fn random(digits: &[char], length: usize) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| *digits.choose(&mut rng).expect("alphabet is empty"))
        .collect()
}

fn unix_ms(t: SystemTime) -> i64 {
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as i64,
        Err(e) => -(e.duration().as_millis() as i64),
    }
}

/// Makes KIDs from one alphabet with fixed part lengths.
#[derive(Clone, Debug)]
pub struct Generator {
    digits: Vec<char>,
    time_length: usize,
    random_length: usize,
    epoch_ms: i64,
    divisor: i64,
}

impl Generator {
    /// A generator for `alphabet` with the given part lengths. A
    /// `time_length` of 0 drops the time part, so the ID is pure random and
    /// does not sort by creation time.
    pub fn new(alphabet: &str, time_length: usize, random_length: usize, options: Options) -> Self {
        Self {
            digits: alphabet.chars().collect(),
            time_length,
            random_length,
            epoch_ms: unix_ms(options.epoch),
            divisor: match options.precision {
                Precision::Millisecond => 1,
                Precision::Second => 1000,
            },
        }
    }

    fn time_part(&self) -> Result<String, Error> {
        if self.time_length == 0 {
            return Ok(String::new());
        }
        let elapsed = unix_ms(SystemTime::now()) - self.epoch_ms;
        if elapsed < 0 {
            return Err(Error::BeforeEpoch {
                epoch_ms: self.epoch_ms,
            });
        }
        encode(
            (elapsed / self.divisor) as u64,
            &self.digits,
            self.time_length,
        )
    }

    /// A new KID starting with `prefix`. Returns an error if the current
    /// time is before the epoch or does not fit in the time part.
    pub fn generate(&self, prefix: &str) -> Result<String, Error> {
        let time = self.time_part()?;
        Ok(format!(
            "{prefix}{time}{}",
            random(&self.digits, self.random_length)
        ))
    }
}

static KID16: LazyLock<Generator> =
    LazyLock::new(|| Generator::new(ALPHABET16, 12, 16, Options::default()));
static KID26: LazyLock<Generator> =
    LazyLock::new(|| Generator::new(ALPHABET26, 10, 14, Options::default()));
static KID36: LazyLock<Generator> =
    LazyLock::new(|| Generator::new(ALPHABET36, 9, 13, Options::default()));
static KID62: LazyLock<Generator> =
    LazyLock::new(|| Generator::new(ALPHABET62, 8, 11, Options::default()));
static LITTLE_KID: LazyLock<Generator> = LazyLock::new(|| {
    Generator::new(
        ALPHABET62,
        5,
        5,
        Options {
            epoch: SystemTime::now(),
            precision: Precision::Second,
        },
    )
});

fn preset(g: &Generator, prefix: &str) -> String {
    g.generate(prefix)
        .expect("the clock is outside the preset's range")
}

/// Hex KID: 12 time chars + 16 random chars, `0-9a-f`.
pub fn kid16(prefix: &str) -> String {
    preset(&KID16, prefix)
}

/// Letters-only KID: 10 time chars + 14 random chars, `a-z`.
pub fn kid26(prefix: &str) -> String {
    preset(&KID26, prefix)
}

/// Lowercase alphanumeric KID: 9 time chars + 13 random chars, `0-9a-z`.
pub fn kid36(prefix: &str) -> String {
    preset(&KID36, prefix)
}

/// Mixed-case alphanumeric KID: 8 time chars + 11 random chars, `0-9A-Za-z`.
pub fn kid62(prefix: &str) -> String {
    preset(&KID62, prefix)
}

/// The default KID (KyleID). Same as [`kid26`].
pub fn kid(prefix: &str) -> String {
    kid26(prefix)
}

/// Most compact useful KID: 10 base62 chars, lasts 29 years from now,
/// P(collision) > 1% if minting more than 1000 IDs per second.
pub fn little_kid(prefix: &str) -> String {
    preset(&LITTLE_KID, prefix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_time_writes_a_number_in_the_alphabets_base_left_padded() {
        assert_eq!(encode_time(0, ALPHABET16, 4).unwrap(), "0000");
        assert_eq!(encode_time(255, ALPHABET16, 4).unwrap(), "00ff");
        assert_eq!(encode_time(26, ALPHABET26, 3).unwrap(), "aba");
        assert_eq!(encode_time(61, ALPHABET62, 2).unwrap(), "0z");
    }

    #[test]
    fn encode_time_errors_when_the_number_does_not_fit() {
        assert_eq!(
            encode_time(256, ALPHABET16, 2),
            Err(Error::DoesNotFit {
                n: 256,
                length: 2,
                base: 16
            })
        );
    }

    #[test]
    fn random_chars_returns_the_requested_length_from_the_alphabet() {
        let s = random_chars("xyz", 50);
        assert_eq!(s.len(), 50);
        assert!(s.chars().all(|c| "xyz".contains(c)));
        assert_eq!(random_chars(ALPHABET16, 0), "");
    }
}
