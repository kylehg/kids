use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const JAN_2020_MS: i64 = 1_577_836_800_000;

struct Run {
    code: i32,
    out: String,
    err: String,
}

fn run(args: &[&str]) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_kid"))
        .args(args)
        .output()
        .unwrap();
    Run {
        code: output.status.code().unwrap(),
        out: String::from_utf8(output.stdout).unwrap(),
        err: String::from_utf8(output.stderr).unwrap(),
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// Check that `out` is one line of `length` chars from `alphabet` after `prefix`.
fn assert_id(out: &str, prefix: &str, alphabet: &str, length: usize) {
    let id = out.strip_suffix('\n').expect("ends with a newline");
    let rest = id.strip_prefix(prefix).expect("starts with the prefix");
    assert_eq!(rest.chars().count(), length, "{out:?}");
    assert!(rest.chars().all(|c| alphabet.contains(c)), "{out:?}");
}

fn decode_hex(s: &str) -> i64 {
    i64::from_str_radix(s.trim(), 16).unwrap()
}

#[test]
fn prints_a_kid26_by_default() {
    let r = run(&[]);
    assert_eq!(r.code, 0);
    assert_id(&r.out, "", kids::ALPHABET26, 24);
}

#[test]
fn picks_the_alphabet_from_the_first_arg() {
    assert_id(&run(&["16"]).out, "", kids::ALPHABET16, 28);
    assert_id(&run(&["36"]).out, "", kids::ALPHABET36, 22);
    assert_id(&run(&["62"]).out, "", kids::ALPHABET62, 19);
    assert_id(&run(&["alt16"]).out, "", kids::ALT16, 28);
}

#[test]
fn takes_prefix_and_part_lengths() {
    assert_id(
        &run(&["62", "-p", "usr_", "-t", "9", "-r", "3"]).out,
        "usr_",
        kids::ALPHABET62,
        12,
    );
    assert_id(
        &run(&["--prefix", "x", "--random", "0"]).out,
        "x",
        kids::ALPHABET26,
        10,
    );
}

#[test]
fn t_0_drops_the_time_part() {
    assert_id(&run(&["-t", "0"]).out, "", kids::ALPHABET26, 14);
    assert_eq!(run(&["-t", "0", "-e", "2999-01-01"]).code, 0);
}

#[test]
fn takes_a_custom_alphabet_with_explicit_lengths() {
    assert_id(&run(&["xyz", "-t", "30", "-r", "4"]).out, "", "xyz", 34);
}

#[test]
fn takes_epoch_and_precision() {
    let since = || now_ms() - JAN_2020_MS;
    let t = decode_hex(&run(&["16", "-t", "12", "-r", "0", "-e", "2020-01-01T00:00:00Z"]).out);
    assert!(t <= since() && t > since() - 5000, "{t}");

    let epoch = JAN_2020_MS.to_string();
    let r = run(&[
        "16",
        "-t",
        "8",
        "-r",
        "0",
        "--epoch",
        &epoch,
        "--precision",
        "sec",
    ]);
    let ts = decode_hex(&r.out);
    assert!(ts <= since() / 1000 && ts > since() / 1000 - 5, "{ts}");
}

#[test]
fn rejects_bad_input() {
    let cases: &[(&[&str], &str)] = &[
        (&["-e", "yesterday"], "must be an ISO date"),
        (&["--precision", "ns"], "possible values: ms, sec"),
        (&["-e", "2999-01-01"], "before the epoch"),
        (&["99"], "alphabet must be one of 16, 26, 36, 62, alt16"),
        (&["xyz"], "--time is required"),
        (&["-r", "abc"], "invalid value 'abc'"),
        (&["16", "-t", "2"], "does not fit"),
        (&["-t", "0", "-r", "0"], "empty"),
    ];
    for (args, want) in cases {
        let r = run(args);
        assert_eq!(r.code, 2, "kid {args:?}");
        assert!(r.err.contains(want), "kid {args:?}: {}", r.err);
    }
}

#[test]
fn prints_help() {
    let r = run(&["--help"]);
    assert_eq!(r.code, 0);
    assert!(r.out.contains("Usage: kid"));
}
