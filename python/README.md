# kids (Python)

Python version of [KIDs](../README.md). Needs Python 3.15 and uses
[uv](https://docs.astral.sh/uv/).

```python
from datetime import UTC, datetime

from kids import ALPHABET36, kid, kid16, kid36, kid62, make_kid

kid()        # "aiorufljxihtlhxntcifnwnb"        a-z, same as kid26
kid("usr_")  # "usr_aiorufljxihtlhxntcifnwnb"
kid16()      # "01a06e1b1b2a2f59d9ba61602246"    0-9a-f
kid36()      # "0mtnembgowal2xk45pvwnz"          0-9a-z
kid62()      # "0VUHkNcwATdiFLsxQdu"             0-9A-Za-z

short = make_kid(ALPHABET36, 6, 10, epoch=datetime(2024, 1, 1, tzinfo=UTC), precision="sec")
short()  # 6 base-36 seconds last ~69 years from 2024
```

- `make_kid(alphabet, time_length, random_length, *, epoch=0, precision="ms")`
  returns a `(prefix="") -> str`. `epoch` is ms since the Unix epoch or a
  `datetime` (a naive one is local time, as with `datetime.timestamp()`).
  It raises `ValueError` for an invalid epoch, and the function it returns
  raises `ValueError` when the current time is before the epoch or does not
  fit.
- `encode_time(ms, alphabet, length)` and `random_chars(alphabet, length)`
  are the pieces `make_kid` is built from. Random characters come from
  `secrets`.
- The alphabets are `ALPHABET16`, `ALPHABET26`, `ALPHABET36`, `ALPHABET62`,
  and `ALT16`.

To use it from another uv project:

```sh
uv add "kids @ git+https://github.com/kylehg/kids#subdirectory=python"
```

## CLI

```sh
uv run kid 62 -p usr_   # usr_0VUHkNcwATdiFLsxQdu
```

To install it as `kid` on your path:

```sh
uv tool install "git+https://github.com/kylehg/kids#subdirectory=python"
```

See the [top-level README](../README.md#cli) for the options. An `--epoch`
date with no offset is read as UTC.

## Develop

```sh
uv sync
uv run pytest
uv run ruff check
uv run ruff format --check
uv run mypy src tests
```
