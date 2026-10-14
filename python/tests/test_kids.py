import math
import re
import subprocess
import sys
import time
from datetime import UTC, datetime

import pytest

from kids import (
    ALPHABET16,
    ALPHABET26,
    ALPHABET36,
    ALPHABET62,
    ALT16,
    KidFn,
    encode_time,
    kid,
    kid16,
    kid26,
    kid36,
    kid62,
    make_kid,
    random_chars,
)

MS_PER_YEAR = 365.2425 * 86_400 * 1000

VARIANTS = [
    pytest.param(kid16, ALPHABET16, 12, 16, id="kid16"),
    pytest.param(kid26, ALPHABET26, 10, 14, id="kid26"),
    pytest.param(kid36, ALPHABET36, 9, 13, id="kid36"),
    pytest.param(kid62, ALPHABET62, 8, 11, id="kid62"),
]


def now_ms() -> int:
    return time.time_ns() // 1_000_000


def decode_time(s: str, alphabet: str) -> int:
    n = 0
    for c in s:
        n = n * len(alphabet) + alphabet.index(c)
    return n


def test_kid_is_kid26() -> None:
    assert kid is kid26


class TestEncodeTime:
    def test_writes_a_number_in_the_alphabets_base_left_padded(self) -> None:
        assert encode_time(0, ALPHABET16, 4) == "0000"
        assert encode_time(255, ALPHABET16, 4) == "00ff"
        assert encode_time(26, ALPHABET26, 3) == "aba"
        assert encode_time(61, ALPHABET62, 2) == "0z"

    def test_raises_when_the_number_does_not_fit(self) -> None:
        with pytest.raises(ValueError):
            encode_time(256, ALPHABET16, 2)


class TestRandomChars:
    def test_returns_the_requested_length_from_the_alphabet(self) -> None:
        assert re.fullmatch(r"[xyz]{50}", random_chars("xyz", 50))

    def test_returns_an_empty_string_for_length_0(self) -> None:
        assert random_chars(ALPHABET16, 0) == ""


@pytest.mark.parametrize(("fn", "alphabet", "time_length", "random_length"), VARIANTS)
class TestVariants:
    def test_alphabet_is_in_ascii_order(
        self, fn: KidFn, alphabet: str, time_length: int, random_length: int
    ) -> None:
        assert "".join(sorted(alphabet)) == alphabet

    def test_time_part_lasts_more_than_1000_years(
        self, fn: KidFn, alphabet: str, time_length: int, random_length: int
    ) -> None:
        assert len(alphabet) ** time_length > now_ms() + 1000 * MS_PER_YEAR

    def test_random_part_has_at_least_64_bits(
        self, fn: KidFn, alphabet: str, time_length: int, random_length: int
    ) -> None:
        assert random_length * math.log2(len(alphabet)) >= 64

    def test_has_the_expected_length_and_only_alphabet_characters(
        self, fn: KidFn, alphabet: str, time_length: int, random_length: int
    ) -> None:
        id = fn()
        assert len(id) == time_length + random_length
        assert set(id) <= set(alphabet)

    def test_prepends_the_prefix(
        self, fn: KidFn, alphabet: str, time_length: int, random_length: int
    ) -> None:
        id = fn("user_")
        assert id.startswith("user_")
        assert len(id) == 5 + time_length + random_length

    def test_encodes_the_current_time_in_the_time_part(
        self, fn: KidFn, alphabet: str, time_length: int, random_length: int
    ) -> None:
        before = now_ms()
        id = fn()
        after = now_ms()
        assert before <= decode_time(id[:time_length], alphabet) <= after

    def test_sorts_by_creation_time(
        self, fn: KidFn, alphabet: str, time_length: int, random_length: int
    ) -> None:
        a = fn()
        time.sleep(0.002)
        b = fn()
        assert a < b

    def test_random_part_differs_between_calls(
        self, fn: KidFn, alphabet: str, time_length: int, random_length: int
    ) -> None:
        assert len({fn() for _ in range(1000)}) == 1000

    def test_random_part_uses_every_character(
        self, fn: KidFn, alphabet: str, time_length: int, random_length: int
    ) -> None:
        seen: set[str] = set()
        for _ in range(500):
            seen.update(fn()[time_length:])
        assert len(seen) == len(alphabet)


class TestMakeKidOptions:
    @staticmethod
    def time_of(id: str, alphabet: str, length: int) -> int:
        return decode_time(id[:length], alphabet)

    def test_epoch_shifts_the_time_part(self) -> None:
        epoch = int(datetime(2020, 1, 1, tzinfo=UTC).timestamp() * 1000)
        fn = make_kid(ALPHABET16, 12, 0, epoch=epoch)
        before = now_ms() - epoch
        t = self.time_of(fn(), ALPHABET16, 12)
        assert before <= t <= now_ms() - epoch

    def test_epoch_accepts_a_datetime(self) -> None:
        epoch = datetime(2020, 1, 1, tzinfo=UTC)
        a = make_kid(ALPHABET16, 12, 0, epoch=epoch)()
        b = make_kid(ALPHABET16, 12, 0, epoch=int(epoch.timestamp() * 1000))()
        assert abs(self.time_of(a, ALPHABET16, 12) - self.time_of(b, ALPHABET16, 12)) < 50

    def test_precision_sec_writes_whole_seconds(self) -> None:
        fn = make_kid(ALPHABET16, 8, 0, precision="sec")
        before = now_ms() // 1000
        t = self.time_of(fn(), ALPHABET16, 8)
        assert before <= t <= now_ms() // 1000

    def test_epoch_and_precision_combine(self) -> None:
        epoch = int(datetime(2020, 1, 1, tzinfo=UTC).timestamp() * 1000)
        fn = make_kid(ALPHABET16, 8, 0, epoch=epoch, precision="sec")
        before = (now_ms() - epoch) // 1000
        t = self.time_of(fn(), ALPHABET16, 8)
        assert before <= t <= (now_ms() - epoch) // 1000

    def test_time_length_0_drops_the_time_part(self) -> None:
        fn = make_kid(ALPHABET16, 0, 8, epoch=now_ms() + 1e9)
        assert re.fullmatch(r"x_[0-9a-f]{8}", fn("x_"))

    def test_raises_when_now_is_before_the_epoch(self) -> None:
        fn = make_kid(ALPHABET16, 12, 0, epoch=now_ms() + 1e9)
        with pytest.raises(ValueError):
            fn()

    def test_raises_on_an_invalid_epoch(self) -> None:
        with pytest.raises(ValueError):
            make_kid(ALPHABET16, 12, 0, epoch=math.nan)


def test_alt16_is_base_16_letters_k_through_z_in_ascii_order() -> None:
    assert ALT16 == "klmnopqrstuvwxyz"
    assert "".join(sorted(ALT16)) == ALT16


class TestCli:
    @staticmethod
    def run(*args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, "-m", "kids", *args], capture_output=True, text=True, check=False
        )

    def test_prints_a_kid26_by_default(self) -> None:
        r = self.run()
        assert r.returncode == 0
        assert re.fullmatch(r"[a-z]{24}\n", r.stdout)

    def test_picks_the_alphabet_from_the_first_arg(self) -> None:
        assert re.fullmatch(r"[0-9a-f]{28}\n", self.run("16").stdout)
        assert re.fullmatch(r"[0-9a-z]{22}\n", self.run("36").stdout)
        assert re.fullmatch(r"[0-9A-Za-z]{19}\n", self.run("62").stdout)
        assert re.fullmatch(r"[k-z]{28}\n", self.run("alt16").stdout)

    def test_takes_prefix_and_part_lengths(self) -> None:
        r = self.run("62", "-p", "usr_", "-t", "9", "-r", "3")
        assert re.fullmatch(r"usr_[0-9A-Za-z]{12}\n", r.stdout)
        assert re.fullmatch(r"x[a-z]{10}\n", self.run("--prefix", "x", "--random", "0").stdout)

    def test_t_0_drops_the_time_part(self) -> None:
        assert re.fullmatch(r"[a-z]{14}\n", self.run("-t", "0").stdout)
        assert self.run("-t", "0", "-e", "2999-01-01").returncode == 0

    def test_takes_a_custom_alphabet_with_explicit_lengths(self) -> None:
        assert re.fullmatch(r"[xyz]{34}\n", self.run("xyz", "-t", "30", "-r", "4").stdout)

    def test_takes_epoch_and_precision(self) -> None:
        epoch = int(datetime(2020, 1, 1, tzinfo=UTC).timestamp() * 1000)
        r = self.run("16", "-t", "12", "-r", "0", "-e", "2020-01-01T00:00:00Z")
        t = decode_time(r.stdout.strip(), ALPHABET16)
        assert now_ms() - epoch - 5000 < t <= now_ms() - epoch

        s = self.run("16", "-t", "8", "-r", "0", "--epoch", str(epoch), "--precision", "sec")
        ts = decode_time(s.stdout.strip(), ALPHABET16)
        assert (now_ms() - epoch) // 1000 - 5 < ts <= (now_ms() - epoch) // 1000

    def test_rejects_bad_input(self) -> None:
        assert "--epoch must be" in self.run("-e", "yesterday").stderr
        assert "--precision must be" in self.run("--precision", "ns").stderr
        assert "before the epoch" in self.run("-e", "2999-01-01").stderr
        assert self.run("99").returncode == 2
        assert "--time is required" in self.run("xyz").stderr
        assert "--random must be a whole number" in self.run("-r", "abc").stderr
        assert "does not fit" in self.run("16", "-t", "2").stderr
        assert "empty" in self.run("-t", "0", "-r", "0").stderr

    def test_prints_help(self) -> None:
        r = self.run("--help")
        assert r.returncode == 0
        assert "Usage: kid" in r.stdout
