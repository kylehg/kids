# kids (Go)

Go version of [KIDs](../README.md). Needs Go 1.27.

```go
import kids "github.com/kylehg/kids/go"

kids.Kid("")        // "aiorufljxihtlhxntcifnwnb"        a-z, same as Kid26
kids.Kid("usr_")    // "usr_aiorufljxihtlhxntcifnwnb"
kids.Kid16("")      // "01a06e1b1b2a2f59d9ba61602246"    0-9a-f
kids.Kid36("")      // "0mtnembgowal2xk45pvwnz"          0-9a-z
kids.Kid62("")      // "0VUHkNcwATdiFLsxQdu"             0-9A-Za-z

short := kids.NewGenerator(kids.Alphabet36, 6, 10, kids.Options{
	Epoch:     time.Date(2024, 1, 1, 0, 0, 0, 0, time.UTC),
	Precision: kids.Second,
})
id, err := short.Generate("") // 6 base-36 seconds last ~69 years from 2024
```

- `NewGenerator(alphabet, timeLength, randomLength, opts)` returns a
  `*Generator`. The zero `Options` means the defaults: the Unix epoch and
  `kids.Millisecond` precision. `Generate(prefix)` returns an error when the
  current time is before the epoch or does not fit; `MustGenerate` panics
  instead. The preset functions (`Kid`, `Kid16`, ...) panic only if the
  clock is outside their range.
- `EncodeTime(n, alphabet, length)` and `RandomChars(alphabet, length)` are
  the pieces a `Generator` is built from. Random characters come from
  `crypto/rand`.
- The alphabets are `Alphabet16`, `Alphabet26`, `Alphabet36`, `Alphabet62`,
  and `Alt16`.

To use it from another module:

```sh
go get github.com/kylehg/kids/go
```

## CLI

```sh
go run ./cmd/kid 62 -p usr_   # usr_0VUHkNcwATdiFLsxQdu
```

To install it as `kid` on your path:

```sh
go install github.com/kylehg/kids/go/cmd/kid@latest
```

See the [top-level README](../README.md#cli) for the options. An `--epoch`
date with no offset is read as UTC.

## Develop

```sh
go test ./...
go vet ./...
gofmt -l .
```
