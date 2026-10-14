use std::collections::HashSet;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use kids::{
    ALPHABET16, ALPHABET26, ALPHABET36, ALPHABET62, ALT16, Generator, Options, Precision, kid,
    kid16, kid26, kid36, kid62,
};

const MS_PER_YEAR: f64 = 365.2425 * 86_400.0 * 1000.0;

struct Variant {
    fn_: fn(&str) -> String,
    alphabet: &'static str,
    time_length: usize,
    random_length: usize,
}

const VARIANTS: [Variant; 4] = [
    Variant {
        fn_: kid16,
        alphabet: ALPHABET16,
        time_length: 12,
        random_length: 16,
    },
    Variant {
        fn_: kid26,
        alphabet: ALPHABET26,
        time_length: 10,
        random_length: 14,
    },
    Variant {
        fn_: kid36,
        alphabet: ALPHABET36,
        time_length: 9,
        random_length: 13,
    },
    Variant {
        fn_: kid62,
        alphabet: ALPHABET62,
        time_length: 8,
        random_length: 11,
    },
];

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

fn decode_time(s: &str, alphabet: &str) -> i64 {
    s.chars().fold(0, |n, c| {
        n * alphabet.len() as i64 + alphabet.find(c).unwrap() as i64
    })
}

fn sorted(s: &str) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    chars.sort();
    chars.into_iter().collect()
}

#[test]
fn kid_is_kid26() {
    let id = kid("");
    assert_eq!(id.len(), 24);
    assert!(id.chars().all(|c| c.is_ascii_lowercase()));
}

#[test]
fn alphabets_are_in_ascii_order_so_ids_sort_as_strings() {
    for v in &VARIANTS {
        assert_eq!(sorted(v.alphabet), v.alphabet);
    }
}

#[test]
fn time_part_lasts_more_than_1000_years_from_now() {
    for v in &VARIANTS {
        let base = v.alphabet.len() as f64;
        assert!(base.powi(v.time_length as i32) > now_ms() as f64 + 1000.0 * MS_PER_YEAR);
    }
}

#[test]
fn random_part_has_at_least_64_bits() {
    for v in &VARIANTS {
        assert!(v.random_length as f64 * (v.alphabet.len() as f64).log2() >= 64.0);
    }
}

#[test]
fn has_the_expected_length_and_only_alphabet_characters() {
    for v in &VARIANTS {
        let id = (v.fn_)("");
        assert_eq!(id.len(), v.time_length + v.random_length);
        assert!(id.chars().all(|c| v.alphabet.contains(c)), "{id}");
    }
}

#[test]
fn prepends_the_prefix() {
    for v in &VARIANTS {
        let id = (v.fn_)("user_");
        assert!(id.starts_with("user_"));
        assert_eq!(id.len(), 5 + v.time_length + v.random_length);
    }
}

#[test]
fn encodes_the_current_time_in_the_time_part() {
    for v in &VARIANTS {
        let before = now_ms();
        let id = (v.fn_)("");
        let after = now_ms();
        let t = decode_time(&id[..v.time_length], v.alphabet);
        assert!(
            (before..=after).contains(&t),
            "{t} not in {before}..={after}"
        );
    }
}

#[test]
fn sorts_by_creation_time() {
    for v in &VARIANTS {
        let a = (v.fn_)("");
        sleep(Duration::from_millis(2));
        let b = (v.fn_)("");
        assert!(a < b, "{a} >= {b}");
    }
}

#[test]
fn random_part_differs_between_calls() {
    for v in &VARIANTS {
        let ids: HashSet<String> = (0..1000).map(|_| (v.fn_)("")).collect();
        assert_eq!(ids.len(), 1000);
    }
}

#[test]
fn random_part_uses_every_character() {
    for v in &VARIANTS {
        let seen: HashSet<char> = (0..500)
            .flat_map(|_| (v.fn_)("")[v.time_length..].chars().collect::<Vec<_>>())
            .collect();
        assert_eq!(seen.len(), v.alphabet.len());
    }
}

fn time_of(g: &Generator, alphabet: &str, length: usize) -> i64 {
    decode_time(&g.generate("").unwrap()[..length], alphabet)
}

fn jan_2020() -> SystemTime {
    UNIX_EPOCH + Duration::from_millis(1_577_836_800_000)
}

#[test]
fn epoch_shifts_the_time_part() {
    let epoch = jan_2020();
    let since = || now_ms() - 1_577_836_800_000;
    let g = Generator::new(
        ALPHABET16,
        12,
        0,
        Options {
            epoch,
            ..Options::default()
        },
    );
    let before = since();
    let t = time_of(&g, ALPHABET16, 12);
    assert!((before..=since()).contains(&t));
}

#[test]
fn precision_second_writes_whole_seconds() {
    let g = Generator::new(
        ALPHABET16,
        8,
        0,
        Options {
            precision: Precision::Second,
            ..Options::default()
        },
    );
    let before = now_ms() / 1000;
    let t = time_of(&g, ALPHABET16, 8);
    assert!((before..=now_ms() / 1000).contains(&t));
}

#[test]
fn epoch_and_precision_combine() {
    let since = || (now_ms() - 1_577_836_800_000) / 1000;
    let g = Generator::new(
        ALPHABET16,
        8,
        0,
        Options {
            epoch: jan_2020(),
            precision: Precision::Second,
        },
    );
    let before = since();
    let t = time_of(&g, ALPHABET16, 8);
    assert!((before..=since()).contains(&t));
}

#[test]
fn time_length_0_drops_the_time_part() {
    let epoch = SystemTime::now() + Duration::from_secs(3600);
    let g = Generator::new(
        ALPHABET16,
        0,
        8,
        Options {
            epoch,
            ..Options::default()
        },
    );
    let id = g.generate("x_").unwrap();
    assert_eq!(id.len(), 10);
    assert!(id.starts_with("x_"));
    assert!(id[2..].chars().all(|c| ALPHABET16.contains(c)));
}

#[test]
fn errors_when_now_is_before_the_epoch() {
    let epoch = SystemTime::now() + Duration::from_secs(3600);
    let g = Generator::new(
        ALPHABET16,
        12,
        0,
        Options {
            epoch,
            ..Options::default()
        },
    );
    assert!(matches!(
        g.generate(""),
        Err(kids::Error::BeforeEpoch { .. })
    ));
}

#[test]
fn alt16_is_base_16_letters_k_through_z_in_ascii_order() {
    assert_eq!(ALT16, "klmnopqrstuvwxyz");
    assert_eq!(sorted(ALT16), ALT16);
}
