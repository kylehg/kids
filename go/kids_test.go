package kids_test

import (
	"math"
	"regexp"
	"slices"
	"strings"
	"testing"
	"time"

	kids "github.com/kylehg/kids/go"
)

const msPerYear = 365.2425 * 86_400 * 1000

var variants = []struct {
	name         string
	fn           func(string) string
	alphabet     string
	timeLength   int
	randomLength int
}{
	{"Kid16", kids.Kid16, kids.Alphabet16, 12, 16},
	{"Kid26", kids.Kid26, kids.Alphabet26, 10, 14},
	{"Kid36", kids.Kid36, kids.Alphabet36, 9, 13},
	{"Kid62", kids.Kid62, kids.Alphabet62, 8, 11},
}

func decodeTime(s, alphabet string) int64 {
	var n int64
	for _, c := range s {
		n = n*int64(len(alphabet)) + int64(strings.IndexRune(alphabet, c))
	}
	return n
}

func sorted(s string) string {
	b := []byte(s)
	slices.Sort(b)
	return string(b)
}

func TestKidIsKid26(t *testing.T) {
	id := kids.Kid("")
	if !regexp.MustCompile(`^[a-z]{24}$`).MatchString(id) {
		t.Errorf("Kid() = %q, want a Kid26", id)
	}
}

func TestEncodeTime(t *testing.T) {
	tests := []struct {
		n        uint64
		alphabet string
		length   int
		want     string
	}{
		{0, kids.Alphabet16, 4, "0000"},
		{255, kids.Alphabet16, 4, "00ff"},
		{26, kids.Alphabet26, 3, "aba"},
		{61, kids.Alphabet62, 2, "0z"},
	}
	for _, tt := range tests {
		got, err := kids.EncodeTime(tt.n, tt.alphabet, tt.length)
		if err != nil || got != tt.want {
			t.Errorf("EncodeTime(%d, %q, %d) = %q, %v; want %q", tt.n, tt.alphabet, tt.length, got, err, tt.want)
		}
	}
}

func TestEncodeTimeDoesNotFit(t *testing.T) {
	if _, err := kids.EncodeTime(256, kids.Alphabet16, 2); err == nil {
		t.Error("EncodeTime(256, hex, 2) did not return an error")
	}
}

func TestRandomChars(t *testing.T) {
	if s := kids.RandomChars("xyz", 50); !regexp.MustCompile(`^[xyz]{50}$`).MatchString(s) {
		t.Errorf("RandomChars(xyz, 50) = %q", s)
	}
	if s := kids.RandomChars(kids.Alphabet16, 0); s != "" {
		t.Errorf("RandomChars(hex, 0) = %q, want empty", s)
	}
}

func TestVariants(t *testing.T) {
	for _, v := range variants {
		t.Run(v.name, func(t *testing.T) {
			base := float64(len(v.alphabet))
			total := v.timeLength + v.randomLength

			t.Run("alphabet is in ASCII order so IDs sort as strings", func(t *testing.T) {
				if sorted(v.alphabet) != v.alphabet {
					t.Errorf("%q is not sorted", v.alphabet)
				}
			})

			t.Run("time part lasts more than 1000 years from now", func(t *testing.T) {
				if math.Pow(base, float64(v.timeLength)) <= float64(time.Now().UnixMilli())+1000*msPerYear {
					t.Error("time part is too short")
				}
			})

			t.Run("random part has at least 64 bits", func(t *testing.T) {
				if float64(v.randomLength)*math.Log2(base) < 64 {
					t.Error("random part is too short")
				}
			})

			t.Run("has the expected length and only alphabet characters", func(t *testing.T) {
				id := v.fn("")
				if len(id) != total {
					t.Errorf("len(%q) = %d, want %d", id, len(id), total)
				}
				for _, c := range id {
					if !strings.ContainsRune(v.alphabet, c) {
						t.Errorf("%q has %q, not in the alphabet", id, c)
					}
				}
			})

			t.Run("prepends the prefix", func(t *testing.T) {
				id := v.fn("user_")
				if !strings.HasPrefix(id, "user_") || len(id) != 5+total {
					t.Errorf("got %q", id)
				}
			})

			t.Run("encodes the current time in the time part", func(t *testing.T) {
				before := time.Now().UnixMilli()
				id := v.fn("")
				after := time.Now().UnixMilli()
				if ts := decodeTime(id[:v.timeLength], v.alphabet); ts < before || ts > after {
					t.Errorf("time part %d not in [%d, %d]", ts, before, after)
				}
			})

			t.Run("sorts by creation time", func(t *testing.T) {
				a := v.fn("")
				time.Sleep(2 * time.Millisecond)
				b := v.fn("")
				if a >= b {
					t.Errorf("%q >= %q", a, b)
				}
			})

			t.Run("random part differs between calls", func(t *testing.T) {
				ids := map[string]bool{}
				for range 1000 {
					ids[v.fn("")] = true
				}
				if len(ids) != 1000 {
					t.Errorf("got %d distinct IDs, want 1000", len(ids))
				}
			})

			t.Run("random part uses every character", func(t *testing.T) {
				seen := map[rune]bool{}
				for range 500 {
					for _, c := range v.fn("")[v.timeLength:] {
						seen[c] = true
					}
				}
				if len(seen) != len(v.alphabet) {
					t.Errorf("saw %d characters, want %d", len(seen), len(v.alphabet))
				}
			})
		})
	}
}

func timeOf(t *testing.T, g *kids.Generator, alphabet string, length int) int64 {
	t.Helper()
	id, err := g.Generate("")
	if err != nil {
		t.Fatal(err)
	}
	return decodeTime(id[:length], alphabet)
}

func TestOptions(t *testing.T) {
	epoch := time.Date(2020, 1, 1, 0, 0, 0, 0, time.UTC)
	since := func() int64 { return time.Now().UnixMilli() - epoch.UnixMilli() }

	t.Run("epoch shifts the time part", func(t *testing.T) {
		g := kids.NewGenerator(kids.Alphabet16, 12, 0, kids.Options{Epoch: epoch})
		before := since()
		if ts := timeOf(t, g, kids.Alphabet16, 12); ts < before || ts > since() {
			t.Errorf("time part %d not since 2020", ts)
		}
	})

	t.Run("precision Second writes whole seconds", func(t *testing.T) {
		g := kids.NewGenerator(kids.Alphabet16, 8, 0, kids.Options{Precision: kids.Second})
		before := time.Now().Unix()
		if ts := timeOf(t, g, kids.Alphabet16, 8); ts < before || ts > time.Now().Unix() {
			t.Errorf("time part %d is not the current second", ts)
		}
	})

	t.Run("epoch and precision combine", func(t *testing.T) {
		g := kids.NewGenerator(kids.Alphabet16, 8, 0, kids.Options{Epoch: epoch, Precision: kids.Second})
		before := since() / 1000
		if ts := timeOf(t, g, kids.Alphabet16, 8); ts < before || ts > since()/1000 {
			t.Errorf("time part %d is not seconds since 2020", ts)
		}
	})

	t.Run("time length 0 drops the time part", func(t *testing.T) {
		g := kids.NewGenerator(kids.Alphabet16, 0, 8, kids.Options{Epoch: time.Now().Add(time.Hour)})
		id, err := g.Generate("x_")
		if err != nil || !regexp.MustCompile(`^x_[0-9a-f]{8}$`).MatchString(id) {
			t.Errorf("got %q, %v", id, err)
		}
	})

	t.Run("errors when now is before the epoch", func(t *testing.T) {
		g := kids.NewGenerator(kids.Alphabet16, 12, 0, kids.Options{Epoch: time.Now().Add(time.Hour)})
		if _, err := g.Generate(""); err == nil {
			t.Error("Generate did not return an error")
		}
	})
}

func TestAlt16(t *testing.T) {
	if kids.Alt16 != "klmnopqrstuvwxyz" || sorted(kids.Alt16) != kids.Alt16 {
		t.Errorf("Alt16 = %q", kids.Alt16)
	}
}
