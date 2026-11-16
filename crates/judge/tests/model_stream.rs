//! Model tests for the bounded line reader and the stderr buffer (requirements R6.1 to
//! R6.4, property P5). No Docker, no clock, no I/O.

use counterspell_judge::sandbox::stream::{LineReader, Overflow, StderrBuffer};
use proptest::prelude::*;

const STDERR_CAP: usize = 64 * 1024;

/// Feeds `chunks` to a reader and returns the lines seen and whether it overflowed.
fn run_reader(cap: usize, chunks: &[Vec<u8>]) -> (Vec<Vec<u8>>, bool) {
    let mut reader = LineReader::new(cap);
    let mut lines = Vec::new();
    for chunk in chunks {
        match reader.push(chunk) {
            Ok(mut found) => lines.append(&mut found),
            Err(Overflow) => return (lines, true),
        }
        assert!(
            reader.held() <= cap,
            "held {} bytes with cap {cap}",
            reader.held()
        );
    }
    (lines, false)
}

/// Splits `stream` at the given cut points (clamped to the stream length), in order.
fn split_at(stream: &[u8], cuts: &[usize]) -> Vec<Vec<u8>> {
    let mut points: Vec<usize> = cuts.iter().map(|c| c % (stream.len() + 1)).collect();
    points.sort_unstable();
    let mut chunks = Vec::new();
    let mut start = 0;
    for point in points {
        chunks.push(stream[start..point].to_vec());
        start = point;
    }
    chunks.push(stream[start..].to_vec());
    chunks
}

#[test]
fn a_line_exactly_at_the_cap_is_accepted() {
    let mut reader = LineReader::new(4);
    assert_eq!(reader.push(b"abcd\n").unwrap(), vec![b"abcd".to_vec()]);
}

#[test]
fn one_byte_over_the_cap_overflows() {
    let mut reader = LineReader::new(4);
    assert_eq!(reader.push(b"abcde\n"), Err(Overflow));
}

#[test]
fn an_unterminated_line_overflows_before_its_newline_arrives() {
    let mut reader = LineReader::new(4);
    assert_eq!(reader.push(b"abcde"), Err(Overflow));
}

#[test]
fn a_line_split_across_chunks_counts_its_whole_length() {
    let mut reader = LineReader::new(4);
    assert_eq!(reader.push(b"ab").unwrap(), Vec::<Vec<u8>>::new());
    assert_eq!(reader.push(b"cde\n"), Err(Overflow));
}

#[test]
fn once_overflowed_the_reader_stays_overflowed() {
    let mut reader = LineReader::new(2);
    assert_eq!(reader.push(b"abc"), Err(Overflow));
    assert_eq!(reader.push(b"\n"), Err(Overflow));
}

#[test]
fn blank_lines_are_returned_as_empty_lines() {
    let mut reader = LineReader::new(8);
    assert_eq!(
        reader.push(b"\n\n").unwrap(),
        vec![Vec::<u8>::new(), Vec::<u8>::new()]
    );
}

#[test]
fn the_reader_holds_at_most_the_cap_of_an_unfinished_line() {
    let mut reader = LineReader::new(4);
    reader.push(b"ab").unwrap();
    assert_eq!(reader.held(), 2);
    reader.push(b"c\n").unwrap();
    assert_eq!(reader.held(), 0);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// P5: any chunking gives the same overflow decision, and the same lines when
    /// nothing overflows. The reader never holds more than the cap.
    #[test]
    fn stdout_lines_do_not_depend_on_chunking(
        stream in prop::collection::vec(prop_oneof![Just(b'a'), Just(b'b'), Just(b'\n')], 0..300),
        cap in 1usize..=12,
        cuts in prop::collection::vec(any::<usize>(), 0..8),
    ) {
        let whole = run_reader(cap, std::slice::from_ref(&stream));
        let chunked = run_reader(cap, &split_at(&stream, &cuts));
        prop_assert_eq!(whole.1, chunked.1, "overflow decision must not depend on chunking");
        if !whole.1 {
            prop_assert_eq!(whole.0, chunked.0);
        }
    }
}

#[test]
fn a_call_budget_overflows_even_when_every_line_is_short() {
    // Ten bytes of budget, spread over short lines. The eleventh byte overflows.
    let mut reader = LineReader::with_budget(10, 10);
    assert_eq!(reader.push(b"a\na\na\na\na\n").unwrap().len(), 5);
    assert_eq!(reader.push(b"a\n"), Err(Overflow));
}

#[test]
fn short_stderr_is_kept_without_a_marker() {
    let mut stderr = StderrBuffer::new();
    stderr.push(b"hello\n");
    assert!(!stderr.is_truncated());
    assert_eq!(stderr.rendered(), b"hello\n");
}

#[test]
fn stderr_exactly_at_the_cap_is_not_marked_truncated() {
    let mut stderr = StderrBuffer::new();
    stderr.push(&vec![b'x'; STDERR_CAP]);
    assert!(!stderr.is_truncated());
}

#[test]
fn stderr_past_the_cap_is_truncated_with_a_marker() {
    let mut stderr = StderrBuffer::new();
    stderr.push(&vec![b'x'; STDERR_CAP + 5000]);
    assert!(stderr.is_truncated());
    let rendered = stderr.rendered();
    assert!(rendered.starts_with(&vec![b'x'; STDERR_CAP]));
    assert!(rendered.ends_with(b"[stderr truncated]\n"));
    assert_eq!(rendered.len(), STDERR_CAP + 20);
}

#[test]
fn the_last_complete_line_survives_a_flood() {
    let mut stderr = StderrBuffer::new();
    stderr.push(&vec![b'x'; 1024 * 1024]);
    stderr.push(b"\nValueError: bad input\n");
    assert_eq!(
        stderr.last_line(),
        Some(b"ValueError: bad input".as_slice())
    );
}

#[test]
fn the_last_line_is_capped_at_four_kib() {
    let mut stderr = StderrBuffer::new();
    stderr.push(&vec![b'y'; 10 * 1024]);
    stderr.push(b"\n");
    assert_eq!(stderr.last_line().unwrap().len(), 4096);
}

#[test]
fn an_unterminated_final_line_is_not_a_complete_line() {
    let mut stderr = StderrBuffer::new();
    stderr.push(b"ValueError: bad\npartial");
    assert_eq!(stderr.last_line(), Some(b"ValueError: bad".as_slice()));
}

#[test]
fn the_truncation_point_does_not_depend_on_chunk_size() {
    let mut stream = vec![b'x'; 70_000];
    stream.extend_from_slice(b"\nlast line\n");

    let mut whole = StderrBuffer::new();
    whole.push(&stream);

    let mut chunked = StderrBuffer::new();
    for chunk in stream.chunks(1000) {
        chunked.push(chunk);
    }

    assert!(whole.is_truncated());
    assert_eq!(whole.rendered(), chunked.rendered());
    assert_eq!(chunked.last_line(), Some(b"last line".as_slice()));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    /// P5 for stderr: any chunking stores the same text, marks truncation the same way
    /// and keeps the same last line. The stored size stays within the cap plus the marker.
    #[test]
    fn stderr_result_does_not_depend_on_chunking(
        stream in prop::collection::vec(prop_oneof![Just(b'a'), Just(b'\n'), Just(b'z')], 0..400),
        cuts in prop::collection::vec(any::<usize>(), 0..8),
    ) {
        let mut whole = StderrBuffer::new();
        whole.push(&stream);

        let mut chunked = StderrBuffer::new();
        for chunk in split_at(&stream, &cuts) {
            chunked.push(&chunk);
        }

        prop_assert_eq!(whole.rendered(), chunked.rendered());
        prop_assert_eq!(whole.is_truncated(), chunked.is_truncated());
        prop_assert_eq!(whole.last_line(), chunked.last_line());
        prop_assert!(chunked.rendered().len() <= STDERR_CAP + 64);
    }
}
