# Difftastic

A language-aware word-level diff.

```
$ cargo run before.js after.js
```

## Other Diff Techniques

There are a bunch of other ways of diffing text files. I summarise
them here, along with example invocations.

### Myers' diff algorithm

This is the default diff algorithm in GNU diff and git diff. It finds
the longest common subsequence (LCS) and is used on a line-by-line basis.

There's a
[great introduction
here](https://blog.jcoglan.com/2017/02/12/the-myers-diff-algorithm-part-1/)
and the original paper is [An O(ND) Difference Algorithm and Its
Variations, Myers
1986](http://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.4.6927).

```
# Modern diff supports colour, but see also
# https://www.colordiff.org/
$ diff --color=always -u sample_files/css_before.css sample_files/css_after.css
```

Note that GNU diff [originally used the Hunt-McIlroy
algorithm](http://fabiensanglard.net/git_code_review/diff.php)).

### Patience Diff

Myer's diff has a problem with sliders:

```diff
 if (!$smtp_server) {
+       $smtp_server = $repo->config('sendemail.smtpserver');
+}
+if (!$smtp_server) {
        foreach (qw( /usr/sbin/sendmail /usr/lib/sendmail )) {
                if (-x $_) {
                        $smtp_server = $_;
```

Instead of:

```diff
+if (!$smtp_server) {
+       $smtp_server = $repo->config('sendemail.smtpserver');
+}
 if (!$smtp_server) {
        foreach (qw( /usr/sbin/sendmail /usr/lib/sendmail )) {
                if (-x $_) {
```

Git has a `--indent-heuristic` that [was added to reduce the
likelihood of making a bad
choice](https://github.com/git/git/commit/433860f3d0beb0c6f205290bd16cda413148f098). There's
a [corpus of test files](https://github.com/mhagger/diff-slider-tools)
where the ideal diff has been chosen by a human, to test different
heuristics.

The [patience diff
algorithm](https://bramcohen.livejournal.com/73318.html) is an LCS
algorithm that aims to do a better job with sliders. It produces great
results by [doing more 
work](https://stackoverflow.com/questions/40133534/is-gits-implementation-of-the-patience-diff-algorithm-correct/40159510#40159510).

```
# Original behaviour
$ git diff --no-indent-heuristic --no-index sample_files/css_before.css sample_files/css_after.css
# As of git 2.11, this heuristic is enabled by default.
$ git diff --indent-heuristic --no-index sample_files/css_before.css sample_files/css_after.css
# Patience algorithm does a better a job in this example.
$ git diff --patience --no-index sample_files/css_before.css sample_files/css_after.css
```

### Histogram Diff

Git 1.7.7+ also has a [histogram
algorithm](https://stackoverflow.com/a/32367597/509706), which aims to
produce better results than Myers' algorithm but without [the slowdown
of the patience algorithm](https://github.com/git/git/commit/85551232b56e763ecfcc7222e0858bac4e962c80).

```
# Inferior to patience on this example file.
$ git diff --histogram --no-index sample_files/css_before.css sample_files/css_after.css
```

### Side-by-side Diff

```
$ diff -y --color=always sample_files/css_before.css sample_files/css_after.css
```

### Tree Diff

Jane Street's [patdiff](https://github.com/janestreet/patdiff)
implements a tree diff, using [an A* algorithm](https://thume.ca/2017/06/17/tree-diffing/).
