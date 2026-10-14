"""The `kid` command: print one KID."""

import argparse
import re
import sys
from dataclasses import dataclass
from datetime import UTC, datetime
from typing import NoReturn

from kids import ALPHABET16, ALPHABET26, ALPHABET36, ALPHABET62, ALT16, Precision, make_kid


@dataclass(frozen=True)
class Preset:
    alphabet: str
    time_length: int | None = None
    random_length: int | None = None


PRESETS = {
    "16": Preset(ALPHABET16, 12, 16),
    "26": Preset(ALPHABET26, 10, 14),
    "36": Preset(ALPHABET36, 9, 13),
    "62": Preset(ALPHABET62, 8, 11),
    "alt16": Preset(ALT16, 12, 16),
}

USAGE = """\
Usage: kid [alphabet] [options]

Print one KID: [prefix] + time part + random part.

Alphabet:
  16       0-9a-f        (12 time + 16 random chars)
  26       a-z           (10 time + 14 random chars, default)
  36       0-9a-z        (9 time + 13 random chars)
  62       0-9A-Za-z     (8 time + 11 random chars)
  alt16    k-z           (12 time + 16 random chars)
  Any other string of 2 or more distinct characters is used as a custom
  alphabet. You must then give -t and -r.

Options:
  -t, --time <n>      Number of time chars. 0 drops the time part.
  -r, --random <n>    Number of random chars
  -p, --prefix <s>    String to put before the ID
  -e, --epoch <when>  Time zero for the time part, as an ISO date or ms since
                      the Unix epoch. Default: the Unix epoch. A later epoch
                      makes the time part last longer.
      --precision <u> Unit of the time part: ms (default) or sec
  -h, --help          Show this help
"""


class UsageError(Exception):
    pass


class _Parser(argparse.ArgumentParser):
    def error(self, message: str) -> NoReturn:
        raise UsageError(message)


def _parse_length(name: str, value: str | None, fallback: int | None) -> int:
    if value is None:
        if fallback is None:
            raise UsageError(f"--{name} is required with a custom alphabet")
        return fallback
    if not re.fullmatch(r"\d+", value):
        raise UsageError(f"--{name} must be a whole number, got {value}")
    return int(value)


def _parse_epoch(value: str | None) -> int | datetime:
    if value is None:
        return 0
    if re.fullmatch(r"-?\d+", value):
        return int(value)
    try:
        when = datetime.fromisoformat(value)
    except ValueError:
        raise UsageError(
            f"--epoch must be an ISO date or ms since the Unix epoch, got {value}"
        ) from None
    # Read a date with no offset as UTC, like JavaScript's Date.parse does.
    return when if when.tzinfo is not None else when.replace(tzinfo=UTC)


def _parse_precision(value: str | None) -> Precision:
    if value is None or value == "ms":
        return "ms"
    if value == "sec":
        return "sec"
    raise UsageError(f"--precision must be ms or sec, got {value}")


def run(argv: list[str]) -> str:
    """Parse `argv` and return one KID. Raises UsageError on bad arguments."""
    parser = _Parser(prog="kid", add_help=False)
    parser.add_argument("alphabet", nargs="*")
    parser.add_argument("-t", "--time")
    parser.add_argument("-r", "--random")
    parser.add_argument("-p", "--prefix", default="")
    parser.add_argument("-e", "--epoch")
    parser.add_argument("--precision")
    parser.add_argument("-h", "--help", action="store_true")
    args = parser.parse_intermixed_args(argv)

    if args.help:
        sys.stdout.write(USAGE)
        sys.exit(0)
    if len(args.alphabet) > 1:
        raise UsageError(f"expected at most one alphabet, got {len(args.alphabet)}")

    name: str = args.alphabet[0] if args.alphabet else "26"
    preset = PRESETS.get(name)
    if preset is None:
        if len(set(name)) != len(name) or len(name) < 2:
            raise UsageError(
                f"alphabet must be one of {', '.join(PRESETS)} or a string of distinct chars"
            )
        preset = Preset(name)

    time_length = _parse_length("time", args.time, preset.time_length)
    random_length = _parse_length("random", args.random, preset.random_length)
    if time_length + random_length == 0:
        raise UsageError("the ID would be empty")

    fn = make_kid(
        preset.alphabet,
        time_length,
        random_length,
        epoch=_parse_epoch(args.epoch),
        precision=_parse_precision(args.precision),
    )
    return fn(args.prefix)


def main(argv: list[str] | None = None) -> None:
    try:
        print(run(sys.argv[1:] if argv is None else argv))
    except UsageError as e:
        sys.stderr.write(f"kid: {e}\n\n{USAGE}")
        sys.exit(2)
    except ValueError as e:
        sys.stderr.write(f"kid: {e}\n")
        sys.exit(2)


if __name__ == "__main__":
    main()
