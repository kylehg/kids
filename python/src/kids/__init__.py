"""KIDs (KyleIDs): short, sortable IDs made of a timestamp part and a random part.

Each KID is: [prefix] + time part + random part.

The time part is the ms since the Unix epoch written in the alphabet's
base, padded on the left with the first character. IDs made later sort
after IDs made earlier as plain strings, because every alphabet is in
ASCII order.

Lengths are chosen so that the time part lasts more than 1000 years from
now and the random part holds at least 64 bits of randomness:

  base  time  lasts until  random  bits
  16    12    year 10889   16      64.0
  26    10    year 6443    14      65.8
  36    9     year 5188    13      67.2
  62    8     year 8888    11      65.5
"""

import math
import secrets
import time
from datetime import UTC, datetime, timedelta
from typing import Literal, Protocol

__all__ = [
    "ALPHABET16",
    "ALPHABET26",
    "ALPHABET36",
    "ALPHABET62",
    "ALT16",
    "KidFn",
    "Precision",
    "encode_time",
    "kid",
    "kid16",
    "kid26",
    "kid36",
    "kid62",
    "little_kid",
    "make_kid",
    "random_chars",
]

ALPHABET16 = "0123456789abcdef"
ALPHABET26 = "abcdefghijklmnopqrstuvwxyz"
ALPHABET36 = "0123456789abcdefghijklmnopqrstuvwxyz"
ALPHABET62 = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
ALT16 = "klmnopqrstuvwxyz"
"""Base 16 in letters only, `k-z`. Same lengths as `ALPHABET16`."""

type Precision = Literal["ms", "sec"]


class KidFn(Protocol):
    def __call__(self, prefix: str = "") -> str: ...


def _now_ms() -> int:
    return time.time_ns() // 1_000_000


def _datetime_ms(when: datetime) -> int:
    # A naive datetime is local time, as with datetime.timestamp().
    return (when.astimezone(UTC) - datetime(1970, 1, 1, tzinfo=UTC)) // timedelta(milliseconds=1)


def encode_time(ms: float, alphabet: str, length: int) -> str:
    """Write `ms` in the base of `alphabet`, padded on the left with the first
    character to `length`. Raises ValueError if `ms` does not fit.
    """
    base = len(alphabet)
    n = math.floor(ms)
    out: list[str] = []
    while n > 0:
        n, digit = divmod(n, base)
        out.append(alphabet[digit])
    if len(out) > length:
        raise ValueError(f"{ms} does not fit in {length} base-{base} chars")
    return "".join(reversed(out)).rjust(length, alphabet[0])


def random_chars(alphabet: str, length: int) -> str:
    """Return `length` characters picked uniformly from `alphabet`."""
    return "".join(secrets.choice(alphabet) for _ in range(length))


def make_kid(
    alphabet: str,
    time_length: int,
    random_length: int,
    *,
    epoch: float | datetime = 0,
    precision: Precision = "ms",
) -> KidFn:
    """Build a KID function for `alphabet` with the given part lengths.

    A `time_length` of 0 drops the time part, so the ID is pure random and
    does not sort by creation time.

    `epoch` is time zero for the time part, as ms since the Unix epoch or a
    datetime. Default 0 (the Unix epoch). A later epoch makes the time part
    last longer. Making an ID before the epoch raises ValueError.

    `precision` is the unit of the time part: "ms" (default) or "sec".
    """
    epoch_ms = _datetime_ms(epoch) if isinstance(epoch, datetime) else epoch
    if not math.isfinite(epoch_ms):
        raise ValueError(f"invalid epoch: {epoch}")
    divisor = 1000 if precision == "sec" else 1

    def time_part() -> str:
        if time_length == 0:
            return ""
        elapsed = _now_ms() - epoch_ms
        if elapsed < 0:
            raise ValueError(f"current time is before the epoch {epoch_ms}")
        return encode_time(elapsed // divisor, alphabet, time_length)

    def kid_fn(prefix: str = "") -> str:
        return prefix + time_part() + random_chars(alphabet, random_length)

    return kid_fn


kid16 = make_kid(ALPHABET16, 12, 16)
"""Hex KID: 12 time chars + 16 random chars, `0-9a-f`."""

kid26 = make_kid(ALPHABET26, 10, 14)
"""Letters-only KID: 10 time chars + 14 random chars, `a-z`."""

kid36 = make_kid(ALPHABET36, 9, 13)
"""Lowercase alphanumeric KID: 9 time chars + 13 random chars, `0-9a-z`."""

kid62 = make_kid(ALPHABET62, 8, 11)
"""Mixed-case alphanumeric KID: 8 time chars + 11 random chars, `0-9A-Za-z`."""

kid = kid26
"""The default KID (KyleID). Same as `kid26`."""

little_kid = make_kid(ALPHABET62, 5, 5, epoch=_now_ms(), precision="sec")
"""Most compact useful KID: 10 base62 chars, lasts 29 years from now,
P(collision) > 1% if minting more than 1000 IDs per second.
"""
