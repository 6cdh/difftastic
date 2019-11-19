use crate::diffs::Change;
use regex::Regex;
use std::cmp::{max, min};
use std::collections::HashSet;

#[cfg(test)]
use pretty_assertions::assert_eq;

// TODO: Move to a separate file, this isn't line related.
/// A range in a string, relative to the string start.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AbsoluteRange {
    pub start: usize, // inclusive
    pub end: usize, // exclusive
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineNumber {
    pub number: usize,
}

impl LineNumber {
    pub fn from(number: usize) -> LineNumber {
        LineNumber { number: number }
    }
}

/// A position in a single line of a string.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct LinePosition {
    /// Both zero-indexed.
    pub line: LineNumber,
    column: usize,
}

/// A range within a single line of a string.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct LineRange {
    /// All zero-indexed.
    pub line: LineNumber,
    pub start: usize,
    pub end: usize,
}

/// A struct for efficiently converting absolute string positions to
/// line-relative positions.
#[derive(Debug)]
pub struct NewlinePositions {
    /// A vector of the start positions of all the lines in `s`.
    positions: Vec<usize>,
}

impl NewlinePositions {
    pub fn from(s: &str) -> NewlinePositions {
        let newline_re = Regex::new("\n").unwrap();
        let newlines: Vec<_> = newline_re.find_iter(s).map(|mat| mat.end()).collect();

        let mut positions = Vec::with_capacity(newlines.len() + 1);
        positions.push(0);
        positions.extend(&newlines);

        NewlinePositions {
            positions: positions,
        }
    }

    pub fn from_offset(self: &NewlinePositions, offset: usize) -> LinePosition {
        for line_num in (0..self.positions.len()).rev() {
            if offset >= self.positions[line_num as usize] {
                return LinePosition {
                    line: LineNumber::from(line_num as usize),
                    column: offset - self.positions[line_num as usize],
                };
            }
        }

        LinePosition {
            line: LineNumber::from(0),
            column: offset,
        }
    }

    // Given a range within a string, split it into ranges where each
    // range is on a single line.
    fn split_line_boundaries(
        self: &NewlinePositions,
        start: LinePosition,
        end: LinePosition,
    ) -> Vec<LineRange> {
        let mut ranges = vec![];

        if start.line == end.line {
            ranges.push(LineRange {
                line: start.line,
                start: start.column,
                end: end.column,
            });
            return ranges;
        } else {
            let first_line_end_pos = self.positions[start.line.number + 1] - 1;
            let first_line_length = first_line_end_pos - self.positions[start.line.number];
            ranges.push(LineRange {
                line: start.line,
                start: start.column,
                end: first_line_length,
            });
        }

        for line_num in (start.line.number + 1)..end.line.number {
            let line_end_pos = self.positions[line_num + 1] - 1;
            let line_length = line_end_pos - self.positions[line_num];
            ranges.push(LineRange {
                line: LineNumber::from(line_num),
                start: 0,
                end: line_length,
            });
        }

        // Last line, up to end.
        ranges.push(LineRange {
            line: end.line,
            start: 0,
            end: end.column,
        });

        ranges
    }

    /// Convert absolute string ranges to line-relative ranges. If the
    /// absolute range crosses a newline, split it into multiple
    /// line-relative ranges.
    pub fn from_ranges(self: &NewlinePositions, ranges: &[AbsoluteRange]) -> Vec<LineRange> {
        let mut rel_positions = vec![];
        for range in ranges {
            let start_pos = self.from_offset(range.start);
            let end_pos = self.from_offset(range.end);

            rel_positions.extend(self.split_line_boundaries(start_pos, end_pos));
        }

        rel_positions
    }
}

#[test]
fn from_offset_newline_boundary() {
    let newline_positions = NewlinePositions::from("abc\nbar");
    let position = newline_positions.from_offset(4);
    assert_eq!(position, LinePosition { line: LineNumber::from(1), column: 0});
}

#[test]
fn from_ranges_first_line() {
    let newline_positions = NewlinePositions::from("foo");
    let relative_ranges = newline_positions.from_ranges(&vec![AbsoluteRange { start: 1, end: 3 }]);
    assert_eq!(
        relative_ranges,
        vec![LineRange {
            line: LineNumber::from(0),
            start: 1,
            end: 3
        }]
    );
}

#[test]
fn from_ranges_split_over_multiple_lines() {
    let newline_positions = NewlinePositions::from("foo\nbar\nbaz\naaaaaaaaaaa");
    let relative_ranges = newline_positions.from_ranges(&vec![AbsoluteRange { start: 5, end: 10 }]);

    assert_eq!(
        relative_ranges,
        vec![
            (LineRange {
                line: LineNumber::from(1),
                start: 1,
                end: 3
            }),
            (LineRange {
                line: LineNumber::from(2),
                start: 0,
                end: 2
            })
        ]
    );
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct MatchedLine {
    pub line: LineNumber,
    pub opposite_line: LineNumber,
}

/// Given a slice of changes, return the unique lines that
/// they land on, plus their corresponding line in the other file.
pub fn relevant_lines(changes: &[Change], s: &str) -> Vec<MatchedLine> {
    let newlines = NewlinePositions::from(s);

    let mut line_nums_seen = HashSet::new();

    let mut result = vec![];
    for change in changes {
        // TODO: refactor to from_range.
        let line_relative_ranges = newlines.from_ranges(&[change.range]);
        for range in line_relative_ranges {
            if line_nums_seen.contains(&range.line) {
                continue;
            }

            line_nums_seen.insert(range.line);
            result.push(MatchedLine {
                line: range.line,
                opposite_line: change.opposite_line,
            });
        }
    }

    result
}

pub fn add_context(
    lines: &[MatchedLine],
    context: usize,
    max_line: LineNumber,
) -> Vec<MatchedLine> {
    let mut result: Vec<MatchedLine> = vec![];

    for matched_line in lines {
        // We know the line number that matches this line. In order to
        // calculate the opposite line number for the context lines,
        // we assume that they line up. Context line -1 should have
        // opposite_line - 1.
        let opposite_offset =
            matched_line.opposite_line.number as isize - matched_line.line.number as isize;

        let line_number = matched_line.line.number;
        let earliest = max(0, line_number as isize - context as isize) as usize;
        let latest = min(line_number + context, max_line.number);

        for i in earliest..latest + 1 {
            let mut is_new = true;
            if let Some(last_line) = result.last() {
                if i <= last_line.line.number {
                    is_new = false;
                }
            }
            if is_new {
                result.push(MatchedLine {
                    line: LineNumber::from(i),
                    opposite_line: LineNumber::from(max(i as isize + opposite_offset, 0) as usize),
                });
            }
        }
    }

    result
}

pub fn max_line(s: &str) -> LineNumber {
    let lines: Vec<_> = s.lines().collect();
    LineNumber::from(lines.len() - 1)
}

#[test]
fn test_add_context() {
    fn matched_line(i: usize) -> MatchedLine {
        MatchedLine {
            line: LineNumber::from(i),
            opposite_line: LineNumber::from(i),
        }
    }

    let start_lines = [matched_line(5), matched_line(12), matched_line(14)];
    let result = add_context(&start_lines, 2, LineNumber::from(20));

    let expected = [
        matched_line(3),
        matched_line(4),
        matched_line(5),
        matched_line(6),
        matched_line(7),
        matched_line(10),
        matched_line(11),
        matched_line(12),
        matched_line(13),
        matched_line(14),
        matched_line(15),
        matched_line(16),
    ];
    assert_eq!(result, expected);
}

#[test]
fn test_add_zero_context() {
    let start_lines = [
        MatchedLine {
            line: LineNumber::from(5),
            opposite_line: LineNumber::from(5),
        },
        MatchedLine {
            line: LineNumber::from(12),
            opposite_line: LineNumber::from(12),
        },
        MatchedLine {
            line: LineNumber::from(14),
            opposite_line: LineNumber::from(14),
        },
    ];
    let result = add_context(&start_lines, 0, LineNumber::from(20));

    assert_eq!(result, start_lines);
}

/// Ensure that every line in `s` has this length. Pad short lines and
/// truncate long lines.
pub fn enforce_length(s: &str, line_length: usize) -> String {
    let mut result = String::with_capacity(s.len());
    for line in s.lines() {
        // TODO: use length in chars not bytes.
        if line.len() > line_length {
            // Truncate.
            result.push_str(&line[0..line_length]);
            result.push_str("\n");
        } else {
            // Pad with spaces.
            result.push_str(&format!("{:width$}\n", line, width = line_length));
        }
    }

    result
}

#[test]
fn enforce_length_short() {
    let result = enforce_length("foo\nbar\n", 5);
    assert_eq!(result, "foo  \nbar  \n");
}

#[test]
fn enforce_length_long() {
    let result = enforce_length("foobar\nbarbaz\n", 3);
    assert_eq!(result, "foo\nbar\n");
}
