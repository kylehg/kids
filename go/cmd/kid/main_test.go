package main

import (
	"bytes"
	"regexp"
	"strconv"
	"strings"
	"testing"
	"time"

	kids "github.com/kylehg/kids/go"
)

type result struct {
	code     int
	out, err string
}

func runKid(args ...string) result {
	var out, err bytes.Buffer
	code := run(args, &out, &err)
	return result{code, out.String(), err.String()}
}

func decodeTime(s, alphabet string) int64 {
	var n int64
	for _, c := range s {
		n = n*int64(len(alphabet)) + int64(strings.IndexRune(alphabet, c))
	}
	return n
}

func TestOutput(t *testing.T) {
	tests := []struct {
		args []string
		want string
	}{
		{nil, `^[a-z]{24}\n$`},
		{[]string{"16"}, `^[0-9a-f]{28}\n$`},
		{[]string{"36"}, `^[0-9a-z]{22}\n$`},
		{[]string{"62"}, `^[0-9A-Za-z]{19}\n$`},
		{[]string{"alt16"}, `^[k-z]{28}\n$`},
		{[]string{"62", "-p", "usr_", "-t", "9", "-r", "3"}, `^usr_[0-9A-Za-z]{12}\n$`},
		{[]string{"--prefix", "x", "--random", "0"}, `^x[a-z]{10}\n$`},
		{[]string{"-t", "0"}, `^[a-z]{14}\n$`},
		{[]string{"-t", "0", "-e", "2999-01-01"}, `^[a-z]{14}\n$`},
		{[]string{"xyz", "-t", "30", "-r", "4"}, `^[xyz]{34}\n$`},
	}
	for _, tt := range tests {
		r := runKid(tt.args...)
		if r.code != 0 || !regexp.MustCompile(tt.want).MatchString(r.out) {
			t.Errorf("kid %v = %d, %q, %q; want output matching %s", tt.args, r.code, r.out, r.err, tt.want)
		}
	}
}

func TestEpochAndPrecision(t *testing.T) {
	epoch := time.Date(2020, 1, 1, 0, 0, 0, 0, time.UTC).UnixMilli()
	since := func() int64 { return time.Now().UnixMilli() - epoch }

	r := runKid("16", "-t", "12", "-r", "0", "-e", "2020-01-01T00:00:00Z")
	if ts := decodeTime(strings.TrimSpace(r.out), kids.Alphabet16); ts > since() || ts <= since()-5000 {
		t.Errorf("-e 2020-01-01: time part %d is not ms since 2020", ts)
	}

	r = runKid("16", "-t", "8", "-r", "0", "--epoch", strconv.FormatInt(epoch, 10), "--precision", "sec")
	if ts := decodeTime(strings.TrimSpace(r.out), kids.Alphabet16); ts > since()/1000 || ts <= since()/1000-5 {
		t.Errorf("--precision sec: time part %d is not seconds since 2020", ts)
	}
}

func TestRejectsBadInput(t *testing.T) {
	tests := []struct {
		args []string
		want string
	}{
		{[]string{"-e", "yesterday"}, "--epoch must be"},
		{[]string{"--precision", "ns"}, "--precision must be"},
		{[]string{"-e", "2999-01-01"}, "before the epoch"},
		{[]string{"99"}, "alphabet must be one of 16, 26, 36, 62, alt16"},
		{[]string{"xyz"}, "--time is required"},
		{[]string{"-r", "abc"}, "--random must be a whole number"},
		{[]string{"16", "-t", "2"}, "does not fit"},
		{[]string{"-t", "0", "-r", "0"}, "empty"},
		{[]string{"16", "26"}, "at most one alphabet"},
		{[]string{"--nope"}, "not defined"},
	}
	for _, tt := range tests {
		r := runKid(tt.args...)
		if r.code != 2 || !strings.Contains(r.err, tt.want) {
			t.Errorf("kid %v = %d, %q; want exit 2 and %q", tt.args, r.code, r.err, tt.want)
		}
	}
}

func TestHelp(t *testing.T) {
	for _, arg := range []string{"-h", "--help"} {
		r := runKid(arg)
		if r.code != 0 || !strings.Contains(r.out, "Usage: kid") {
			t.Errorf("kid %s = %d, %q", arg, r.code, r.out)
		}
	}
}
