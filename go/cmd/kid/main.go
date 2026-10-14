// Command kid prints one KID.
package main

import (
	"errors"
	"flag"
	"fmt"
	"io"
	"maps"
	"os"
	"regexp"
	"slices"
	"strconv"
	"strings"
	"time"

	kids "github.com/kylehg/kids/go"
)

type preset struct {
	alphabet     string
	timeLength   *int
	randomLength *int
}

var presets = map[string]preset{
	"16":    {kids.Alphabet16, new(12), new(16)},
	"26":    {kids.Alphabet26, new(10), new(14)},
	"36":    {kids.Alphabet36, new(9), new(13)},
	"62":    {kids.Alphabet62, new(8), new(11)},
	"alt16": {kids.Alt16, new(12), new(16)},
}

const usage = `Usage: kid [alphabet] [options]

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
`

// usageError is a bad argument. It is printed with the usage text.
type usageError string

func (e usageError) Error() string { return string(e) }

func usageErrorf(format string, args ...any) error {
	return usageError(fmt.Sprintf(format, args...))
}

var (
	wholeNumber = regexp.MustCompile(`^\d+$`)
	integer     = regexp.MustCompile(`^-?\d+$`)
)

func parseLength(name string, value *string, fallback *int) (int, error) {
	if value == nil {
		if fallback == nil {
			return 0, usageErrorf("--%s is required with a custom alphabet", name)
		}
		return *fallback, nil
	}
	n, err := strconv.Atoi(*value)
	if !wholeNumber.MatchString(*value) || err != nil {
		return 0, usageErrorf("--%s must be a whole number, got %s", name, *value)
	}
	return n, nil
}

// epochLayouts are the ISO date forms --epoch takes. A time with no offset
// is read as UTC.
var epochLayouts = []string{
	time.RFC3339Nano,
	"2006-01-02T15:04:05.999999999",
	"2006-01-02T15:04",
	"2006-01-02",
}

func parseEpoch(value *string) (time.Time, error) {
	if value == nil {
		return time.Time{}, nil
	}
	if integer.MatchString(*value) {
		if ms, err := strconv.ParseInt(*value, 10, 64); err == nil {
			return time.UnixMilli(ms), nil
		}
	}
	for _, layout := range epochLayouts {
		if t, err := time.Parse(layout, *value); err == nil {
			return t, nil
		}
	}
	return time.Time{}, usageErrorf("--epoch must be an ISO date or ms since the Unix epoch, got %s", *value)
}

func parsePrecision(value *string) (kids.Precision, error) {
	if value == nil || *value == "ms" {
		return kids.Millisecond, nil
	}
	if *value == "sec" {
		return kids.Second, nil
	}
	return 0, usageErrorf("--precision must be ms or sec, got %s", *value)
}

func distinct(s string) bool {
	seen := map[rune]bool{}
	for _, r := range s {
		if seen[r] {
			return false
		}
		seen[r] = true
	}
	return true
}

// generate parses args and returns one KID.
func generate(args []string) (string, error) {
	fs := flag.NewFlagSet("kid", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	var timeArg, randomArg, epochArg, precisionArg *string
	prefix := ""
	stringFlag := func(dst **string, names ...string) {
		for _, name := range names {
			fs.Func(name, "", func(s string) error { *dst = &s; return nil })
		}
	}
	stringFlag(&timeArg, "t", "time")
	stringFlag(&randomArg, "r", "random")
	stringFlag(&epochArg, "e", "epoch")
	stringFlag(&precisionArg, "precision")
	fs.StringVar(&prefix, "p", "", "")
	fs.StringVar(&prefix, "prefix", "", "")

	// The flag package stops at the first positional arg, so keep parsing
	// after each one to allow flags on either side of the alphabet.
	var positionals []string
	for {
		if err := fs.Parse(args); err != nil {
			if errors.Is(err, flag.ErrHelp) {
				return "", err
			}
			return "", usageError(err.Error())
		}
		args = fs.Args()
		if len(args) == 0 {
			break
		}
		positionals = append(positionals, args[0])
		args = args[1:]
	}
	if len(positionals) > 1 {
		return "", usageErrorf("expected at most one alphabet, got %d", len(positionals))
	}

	name := "26"
	if len(positionals) == 1 {
		name = positionals[0]
	}
	p, ok := presets[name]
	if !ok {
		if !distinct(name) || len([]rune(name)) < 2 {
			names := slices.Sorted(maps.Keys(presets))
			return "", usageErrorf("alphabet must be one of %s or a string of distinct chars", strings.Join(names, ", "))
		}
		p = preset{alphabet: name}
	}

	timeLength, err := parseLength("time", timeArg, p.timeLength)
	if err != nil {
		return "", err
	}
	randomLength, err := parseLength("random", randomArg, p.randomLength)
	if err != nil {
		return "", err
	}
	if timeLength+randomLength == 0 {
		return "", usageError("the ID would be empty")
	}
	epoch, err := parseEpoch(epochArg)
	if err != nil {
		return "", err
	}
	precision, err := parsePrecision(precisionArg)
	if err != nil {
		return "", err
	}

	g := kids.NewGenerator(p.alphabet, timeLength, randomLength, kids.Options{Epoch: epoch, Precision: precision})
	return g.Generate(prefix)
}

func run(args []string, stdout, stderr io.Writer) int {
	id, err := generate(args)
	var ue usageError
	switch {
	case errors.Is(err, flag.ErrHelp):
		fmt.Fprint(stdout, usage)
		return 0
	case errors.As(err, &ue):
		fmt.Fprintf(stderr, "kid: %s\n\n%s", err, usage)
		return 2
	case err != nil:
		fmt.Fprintf(stderr, "kid: %s\n", err)
		return 2
	}
	fmt.Fprintln(stdout, id)
	return 0
}

func main() {
	os.Exit(run(os.Args[1:], os.Stdout, os.Stderr))
}
