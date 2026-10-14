use std::collections::HashSet;
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser, ValueEnum};
use jiff::Timestamp;
use jiff::civil::DateTime;
use jiff::tz::TimeZone;
use kids::{ALPHABET16, ALPHABET26, ALPHABET36, ALPHABET62, ALT16, Generator, Options, Precision};

const PRESETS: [(&str, &str, usize, usize); 5] = [
    ("16", ALPHABET16, 12, 16),
    ("26", ALPHABET26, 10, 14),
    ("36", ALPHABET36, 9, 13),
    ("62", ALPHABET62, 8, 11),
    ("alt16", ALT16, 12, 16),
];

const ALPHABETS_HELP: &str = "\
Alphabets:
  16       0-9a-f        (12 time + 16 random chars)
  26       a-z           (10 time + 14 random chars, default)
  36       0-9a-z        (9 time + 13 random chars)
  62       0-9A-Za-z     (8 time + 11 random chars)
  alt16    k-z           (12 time + 16 random chars)
  Any other string of 2 or more distinct characters is used as a custom
  alphabet. You must then give -t and -r.";

/// Print one KID: [prefix] + time part + random part.
#[derive(Parser)]
#[command(name = "kid", version, after_help = ALPHABETS_HELP, args_override_self = true)]
struct Cli {
    /// 16, 26, 36, 62, alt16, or a custom alphabet
    #[arg(default_value = "26")]
    alphabet: String,

    /// Number of time chars. 0 drops the time part.
    #[arg(short, long, value_name = "N")]
    time: Option<usize>,

    /// Number of random chars
    #[arg(short, long, value_name = "N")]
    random: Option<usize>,

    /// String to put before the ID
    #[arg(short, long, value_name = "S", default_value = "")]
    prefix: String,

    /// Time zero for the time part, as an ISO date or ms since the Unix
    /// epoch. Default: the Unix epoch. A later epoch makes the time part
    /// last longer.
    #[arg(short, long, value_name = "WHEN", value_parser = parse_epoch, allow_hyphen_values = true)]
    epoch: Option<SystemTime>,

    /// Unit of the time part
    #[arg(long, value_name = "U", value_enum, default_value_t = Unit::Ms)]
    precision: Unit,
}

#[derive(Clone, Copy, ValueEnum)]
enum Unit {
    Ms,
    Sec,
}

/// Parse an ISO date or ms since the Unix epoch. A date with no offset is
/// read as UTC.
fn parse_epoch(value: &str) -> Result<SystemTime, String> {
    if let Ok(ms) = value.parse::<i64>() {
        let offset = Duration::from_millis(ms.unsigned_abs());
        return Ok(if ms < 0 {
            UNIX_EPOCH - offset
        } else {
            UNIX_EPOCH + offset
        });
    }
    let timestamp = value.parse::<Timestamp>().or_else(|_| {
        value
            .parse::<DateTime>()
            .and_then(|dt| dt.to_zoned(TimeZone::UTC))
            .map(|z| z.timestamp())
    });
    timestamp
        .map(SystemTime::from)
        .map_err(|_| "must be an ISO date or ms since the Unix epoch".to_string())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let usage_error = |kind, msg: String| Cli::command().error(kind, msg).exit();

    let (alphabet, time_length, random_length) = match PRESETS
        .iter()
        .find(|(name, ..)| *name == cli.alphabet)
    {
        Some(&(_, alphabet, t, r)) => (alphabet, cli.time.unwrap_or(t), cli.random.unwrap_or(r)),
        None => {
            let chars: Vec<char> = cli.alphabet.chars().collect();
            if chars.len() < 2 || chars.iter().collect::<HashSet<_>>().len() != chars.len() {
                let names: Vec<&str> = PRESETS.iter().map(|(name, ..)| *name).collect();
                usage_error(
                    ErrorKind::InvalidValue,
                    format!(
                        "alphabet must be one of {} or a string of distinct chars",
                        names.join(", ")
                    ),
                );
            }
            let required = |name: &str, value: Option<usize>| {
                value.unwrap_or_else(|| {
                    usage_error(
                        ErrorKind::MissingRequiredArgument,
                        format!("--{name} is required with a custom alphabet"),
                    )
                })
            };
            (
                cli.alphabet.as_str(),
                required("time", cli.time),
                required("random", cli.random),
            )
        }
    };
    if time_length + random_length == 0 {
        usage_error(ErrorKind::InvalidValue, "the ID would be empty".to_string());
    }

    let options = Options {
        epoch: cli.epoch.unwrap_or(UNIX_EPOCH),
        precision: match cli.precision {
            Unit::Ms => Precision::Millisecond,
            Unit::Sec => Precision::Second,
        },
    };
    match Generator::new(alphabet, time_length, random_length, options).generate(&cli.prefix) {
        Ok(id) => {
            println!("{id}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("kid: {e}");
            ExitCode::from(2)
        }
    }
}
