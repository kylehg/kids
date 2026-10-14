// Package kids makes KIDs (KyleIDs): short, sortable IDs made of a timestamp
// part and a random part.
//
// Each KID is: [prefix] + time part + random part.
//
// The time part is the ms since the Unix epoch written in the alphabet's
// base, padded on the left with the first character. IDs made later sort
// after IDs made earlier as plain strings, because every alphabet is in
// ASCII order.
//
// Lengths are chosen so that the time part lasts more than 1000 years from
// now and the random part holds at least 64 bits of randomness:
//
//	base  time  lasts until  random  bits
//	16    12    year 10889   16      64.0
//	26    10    year 6443    14      65.8
//	36    9     year 5188    13      67.2
//	62    8     year 8888    11      65.5
package kids

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"strings"
	"time"
)

const (
	Alphabet16 = "0123456789abcdef"
	Alphabet26 = "abcdefghijklmnopqrstuvwxyz"
	Alphabet36 = "0123456789abcdefghijklmnopqrstuvwxyz"
	Alphabet62 = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
	// Alt16 is base 16 in letters only, k-z. Same lengths as Alphabet16.
	Alt16 = "klmnopqrstuvwxyz"
)

// Precision is the unit of the time part.
type Precision int

const (
	Millisecond Precision = iota
	Second
)

// Options configure a Generator. The zero value means the defaults.
type Options struct {
	// Epoch is time zero for the time part. The zero time.Time means the
	// Unix epoch. A later epoch makes the time part last longer. Making an
	// ID before the epoch is an error.
	Epoch time.Time
	// Precision is the unit of the time part. Default Millisecond.
	Precision Precision
}

// EncodeTime writes n in the base of alphabet, padded on the left with the
// first character to length. It returns an error if n does not fit.
func EncodeTime(n uint64, alphabet string, length int) (string, error) {
	digits := []rune(alphabet)
	base := uint64(len(digits))
	out := make([]rune, 0, length)
	for m := n; m > 0; m /= base {
		out = append(out, digits[m%base])
	}
	if len(out) > length {
		return "", fmt.Errorf("%d does not fit in %d base-%d chars", n, length, base)
	}
	for len(out) < length {
		out = append(out, digits[0])
	}
	for i, j := 0, len(out)-1; i < j; i, j = i+1, j-1 {
		out[i], out[j] = out[j], out[i]
	}
	return string(out), nil
}

// RandomChars returns length characters picked uniformly from alphabet.
func RandomChars(alphabet string, length int) string {
	digits := []rune(alphabet)
	base := big.NewInt(int64(len(digits)))
	var b strings.Builder
	for range length {
		i, err := rand.Int(rand.Reader, base)
		if err != nil {
			panic(err) // crypto/rand.Reader does not fail.
		}
		b.WriteRune(digits[i.Int64()])
	}
	return b.String()
}

// A Generator makes KIDs from one alphabet with fixed part lengths.
type Generator struct {
	alphabet     string
	timeLength   int
	randomLength int
	epochMs      int64
	divisor      int64
}

// NewGenerator returns a Generator for alphabet with the given part lengths.
// A timeLength of 0 drops the time part, so the ID is pure random and does
// not sort by creation time.
func NewGenerator(alphabet string, timeLength, randomLength int, opts Options) *Generator {
	var epochMs int64
	if !opts.Epoch.IsZero() {
		epochMs = opts.Epoch.UnixMilli()
	}
	var divisor int64 = 1
	if opts.Precision == Second {
		divisor = 1000
	}
	return &Generator{alphabet, timeLength, randomLength, epochMs, divisor}
}

func (g *Generator) timePart() (string, error) {
	if g.timeLength == 0 {
		return "", nil
	}
	elapsed := time.Now().UnixMilli() - g.epochMs
	if elapsed < 0 {
		return "", fmt.Errorf("current time is before the epoch %d", g.epochMs)
	}
	return EncodeTime(uint64(elapsed/g.divisor), g.alphabet, g.timeLength)
}

// Generate returns a new KID starting with prefix. It returns an error if
// the current time is before the epoch or does not fit in the time part.
func (g *Generator) Generate(prefix string) (string, error) {
	t, err := g.timePart()
	if err != nil {
		return "", err
	}
	return prefix + t + RandomChars(g.alphabet, g.randomLength), nil
}

// MustGenerate is like Generate but panics on error.
func (g *Generator) MustGenerate(prefix string) string {
	id, err := g.Generate(prefix)
	if err != nil {
		panic(err)
	}
	return id
}

var (
	kid16     = NewGenerator(Alphabet16, 12, 16, Options{})
	kid26     = NewGenerator(Alphabet26, 10, 14, Options{})
	kid36     = NewGenerator(Alphabet36, 9, 13, Options{})
	kid62     = NewGenerator(Alphabet62, 8, 11, Options{})
	littleKid = NewGenerator(Alphabet62, 5, 5, Options{Epoch: time.Now(), Precision: Second})
)

// Kid16 returns a hex KID: 12 time chars + 16 random chars, 0-9a-f.
func Kid16(prefix string) string { return kid16.MustGenerate(prefix) }

// Kid26 returns a letters-only KID: 10 time chars + 14 random chars, a-z.
func Kid26(prefix string) string { return kid26.MustGenerate(prefix) }

// Kid36 returns a lowercase alphanumeric KID: 9 time chars + 13 random
// chars, 0-9a-z.
func Kid36(prefix string) string { return kid36.MustGenerate(prefix) }

// Kid62 returns a mixed-case alphanumeric KID: 8 time chars + 11 random
// chars, 0-9A-Za-z.
func Kid62(prefix string) string { return kid62.MustGenerate(prefix) }

// Kid returns the default KID (KyleID). Same as Kid26.
func Kid(prefix string) string { return Kid26(prefix) }

// LittleKid returns the most compact useful KID: 10 base62 chars, lasts 29
// years from now, P(collision) > 1% if minting more than 1000 IDs per
// second.
func LittleKid(prefix string) string { return littleKid.MustGenerate(prefix) }
