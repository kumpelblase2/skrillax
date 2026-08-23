use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

const SEQUENCE_SPACE: i64 = 1_i64 << 32;
const HALF_SEQUENCE_SPACE: i64 = SEQUENCE_SPACE / 2;
const MAX_BUFFERED_BYTES: usize = 8 * 1024 * 1024;
const MAX_PENDING_SEGMENTS: usize = 4096;
const MAX_FORWARD_GAP: i64 = 16 * 1024 * 1024;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum TcpReassemblyError {
    ForwardGapTooLarge { gap: u64, maximum: u64 },
    BufferLimitExceeded { bytes: usize, maximum: usize },
    SegmentLimitExceeded { segments: usize, maximum: usize },
}

impl Display for TcpReassemblyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ForwardGapTooLarge { gap, maximum } => {
                write!(f, "TCP sequence gap of {gap} bytes exceeds the {maximum}-byte limit")
            },
            Self::BufferLimitExceeded { bytes, maximum } => {
                write!(
                    f,
                    "TCP reassembly needs {bytes} buffered bytes, exceeding the {maximum}-byte limit"
                )
            },
            Self::SegmentLimitExceeded { segments, maximum } => {
                write!(
                    f,
                    "TCP reassembly needs {segments} pending segments, exceeding the {maximum}-segment limit"
                )
            },
        }
    }
}

impl Error for TcpReassemblyError {}

/// Reconstructs one direction of a TCP byte stream from captured segments.
///
/// Only contiguous bytes that have not previously been emitted are returned.
/// Retransmissions are discarded and future segments are retained until gaps
/// are filled. If a capture starts after the handshake, the first observed
/// segment anchors the sequence space; bytes sent before that point cannot be
/// recovered from the capture.
#[derive(Default)]
pub(crate) struct TcpReassembler {
    next_sequence: Option<i64>,
    pending: BTreeMap<i64, Vec<u8>>,
    pending_fin: BTreeSet<i64>,
    buffered_bytes: usize,
    finished: bool,
}

impl TcpReassembler {
    pub(crate) fn is_finished(&self) -> bool {
        self.finished
    }

    pub(crate) fn push(
        &mut self,
        sequence_number: u32,
        syn: bool,
        fin: bool,
        payload: &[u8],
    ) -> Result<Vec<u8>, TcpReassemblyError> {
        if self.finished {
            return Ok(Vec::new());
        }

        let sequence_number = match self.next_sequence {
            Some(checkpoint) => unwrap_sequence_number(sequence_number, checkpoint),
            None => SEQUENCE_SPACE + i64::from(sequence_number),
        };
        let payload_start = sequence_number + i64::from(syn);
        let payload_end = payload_start + payload.len() as i64;
        let next_sequence = *self.next_sequence.get_or_insert(payload_start);

        let (unseen_start, uncovered) = if payload_end > next_sequence {
            let unseen_start = payload_start.max(next_sequence);
            let unseen_offset = (unseen_start - payload_start) as usize;
            let uncovered = self.uncovered_segments(unseen_start, &payload[unseen_offset..]);
            self.validate_pending(unseen_start, &uncovered, next_sequence)?;
            (unseen_start, uncovered)
        } else {
            (next_sequence, Vec::new())
        };
        let added_pending_segments = if unseen_start > next_sequence {
            uncovered.len()
        } else {
            0
        };
        if fin && payload_end >= next_sequence {
            self.validate_fin(payload_end, next_sequence, added_pending_segments)?;
        }

        for (start, payload) in uncovered {
            self.insert_pending(start, payload);
        }
        if fin && payload_end >= next_sequence {
            self.pending_fin.insert(payload_end);
        }
        let emitted = self.drain_contiguous();
        self.consume_contiguous_fin();
        Ok(emitted)
    }

    fn validate_pending(
        &self,
        start: i64,
        uncovered: &[(i64, Vec<u8>)],
        next_sequence: i64,
    ) -> Result<(), TcpReassemblyError> {
        if start <= next_sequence {
            return Ok(());
        }

        let gap = (start - next_sequence) as u64;
        if gap > MAX_FORWARD_GAP as u64 {
            return Err(TcpReassemblyError::ForwardGapTooLarge {
                gap,
                maximum: MAX_FORWARD_GAP as u64,
            });
        }

        let added_bytes = uncovered.iter().map(|(_, payload)| payload.len()).sum::<usize>();
        let buffered_bytes = self.buffered_bytes.saturating_add(added_bytes);
        if buffered_bytes > MAX_BUFFERED_BYTES {
            return Err(TcpReassemblyError::BufferLimitExceeded {
                bytes: buffered_bytes,
                maximum: MAX_BUFFERED_BYTES,
            });
        }

        let segments = self.pending.len() + self.pending_fin.len() + uncovered.len();
        if segments > MAX_PENDING_SEGMENTS {
            return Err(TcpReassemblyError::SegmentLimitExceeded {
                segments,
                maximum: MAX_PENDING_SEGMENTS,
            });
        }

        Ok(())
    }

    fn validate_fin(
        &self,
        fin_sequence: i64,
        next_sequence: i64,
        added_pending_segments: usize,
    ) -> Result<(), TcpReassemblyError> {
        if fin_sequence <= next_sequence || self.pending_fin.contains(&fin_sequence) {
            return Ok(());
        }

        let gap = (fin_sequence - next_sequence) as u64;
        if gap > MAX_FORWARD_GAP as u64 {
            return Err(TcpReassemblyError::ForwardGapTooLarge {
                gap,
                maximum: MAX_FORWARD_GAP as u64,
            });
        }

        let segments = self.pending.len() + self.pending_fin.len() + added_pending_segments + 1;
        if segments > MAX_PENDING_SEGMENTS {
            return Err(TcpReassemblyError::SegmentLimitExceeded {
                segments,
                maximum: MAX_PENDING_SEGMENTS,
            });
        }

        Ok(())
    }

    fn uncovered_segments(&self, start: i64, payload: &[u8]) -> Vec<(i64, Vec<u8>)> {
        let end = start + payload.len() as i64;
        let mut cursor = start;
        let mut uncovered = Vec::new();

        for (&existing_start, existing) in self.pending.range(..end) {
            let existing_end = existing_start + existing.len() as i64;
            if existing_end <= cursor {
                continue;
            }

            if existing_start > cursor {
                let uncovered_end = existing_start.min(end);
                let start_offset = (cursor - start) as usize;
                let end_offset = (uncovered_end - start) as usize;
                uncovered.push((cursor, payload[start_offset..end_offset].to_vec()));
            }

            cursor = cursor.max(existing_end);
            if cursor >= end {
                break;
            }
        }

        if cursor < end {
            let start_offset = (cursor - start) as usize;
            uncovered.push((cursor, payload[start_offset..].to_vec()));
        }

        uncovered
    }

    fn insert_pending(&mut self, start: i64, payload: Vec<u8>) {
        self.buffered_bytes += payload.len();
        let replaced = self.pending.insert(start, payload);
        debug_assert!(replaced.is_none(), "uncovered TCP ranges must not overlap");
    }

    fn consume_contiguous_fin(&mut self) {
        let mut next_sequence = self.next_sequence.expect("sequence was initialized");

        while let Some(&fin_sequence) = self.pending_fin.first() {
            if fin_sequence > next_sequence {
                break;
            }

            self.pending_fin.remove(&fin_sequence);
            if fin_sequence == next_sequence {
                next_sequence += 1;
                self.finished = true;
            }
        }

        self.next_sequence = Some(next_sequence);
        if self.finished {
            self.pending.clear();
            self.pending_fin.clear();
            self.buffered_bytes = 0;
        }
    }

    fn drain_contiguous(&mut self) -> Vec<u8> {
        let mut emitted = Vec::new();
        let mut next_sequence = self.next_sequence.expect("sequence was initialized");

        while let Some((&start, payload)) = self.pending.first_key_value() {
            let terminal_fin = self.pending_fin.first().copied();
            if start > next_sequence || terminal_fin.is_some_and(|fin| next_sequence >= fin) {
                break;
            }

            let payload_len = payload.len();
            let payload_end = start + payload_len as i64;
            let emitted_end = terminal_fin.map_or(payload_end, |fin| payload_end.min(fin));
            let payload = self.pending.remove(&start).expect("entry was just observed");
            self.buffered_bytes -= payload_len;

            if emitted_end <= next_sequence {
                continue;
            }

            let unseen_offset = (next_sequence - start) as usize;
            let emitted_end_offset = (emitted_end - start) as usize;
            emitted.extend_from_slice(&payload[unseen_offset..emitted_end_offset]);
            next_sequence = emitted_end;
        }

        self.next_sequence = Some(next_sequence);
        emitted
    }
}

fn unwrap_sequence_number(sequence_number: u32, checkpoint: i64) -> i64 {
    let generation = checkpoint.div_euclid(SEQUENCE_SPACE);
    let mut candidate = generation * SEQUENCE_SPACE + i64::from(sequence_number);
    let distance = candidate - checkpoint;

    if distance > HALF_SEQUENCE_SPACE {
        candidate -= SEQUENCE_SPACE;
    } else if distance < -HALF_SEQUENCE_SPACE {
        candidate += SEQUENCE_SPACE;
    }

    candidate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_retransmission_is_emitted_once() {
        let mut stream = TcpReassembler::default();

        assert_eq!(stream.push(100, false, false, b"packet").unwrap(), b"packet");
        assert!(stream.push(100, false, false, b"packet").unwrap().is_empty());
    }

    #[test]
    fn overlapping_retransmission_emits_only_its_unseen_suffix() {
        let mut stream = TcpReassembler::default();

        assert_eq!(stream.push(100, false, false, b"packet").unwrap(), b"packet");
        assert_eq!(stream.push(103, false, false, b"ket bytes").unwrap(), b" bytes");
    }

    #[test]
    fn out_of_order_data_is_held_until_the_gap_is_filled() {
        let mut stream = TcpReassembler::default();

        assert!(stream.push(99, true, false, b"").unwrap().is_empty());
        assert!(stream.push(106, false, false, b"world").unwrap().is_empty());
        assert_eq!(stream.push(100, false, false, b"hello ").unwrap(), b"hello world");
    }

    #[test]
    fn first_observed_bytes_win_when_pending_segments_conflict() {
        let mut stream = TcpReassembler::default();

        stream.push(99, true, false, b"").unwrap();
        stream.push(105, false, false, b"FIRST").unwrap();
        stream.push(103, false, false, b"abXXXXX").unwrap();

        assert_eq!(stream.push(100, false, false, b"012").unwrap(), b"012abFIRST");
    }

    #[test]
    fn sequence_numbers_wrap_without_reemitting_data() {
        let mut stream = TcpReassembler::default();

        assert!(stream.push(u32::MAX - 1, true, false, b"").unwrap().is_empty());
        assert_eq!(stream.push(u32::MAX, false, false, b"ab").unwrap(), b"ab");
        assert!(stream.push(u32::MAX, false, false, b"ab").unwrap().is_empty());
        assert_eq!(stream.push(1, false, false, b"c").unwrap(), b"c");
    }

    #[test]
    fn implausibly_large_forward_gap_is_rejected() {
        let mut stream = TcpReassembler::default();
        stream.push(99, true, false, b"").unwrap();

        assert_eq!(
            stream
                .push(100 + MAX_FORWARD_GAP as u32 + 1, false, false, b"future")
                .unwrap_err(),
            TcpReassemblyError::ForwardGapTooLarge {
                gap: MAX_FORWARD_GAP as u64 + 1,
                maximum: MAX_FORWARD_GAP as u64,
            }
        );
    }

    #[test]
    fn out_of_order_fin_consumes_sequence_space_after_the_gap_closes() {
        let mut stream = TcpReassembler::default();

        stream.push(99, true, false, b"").unwrap();
        stream.push(105, false, true, b"").unwrap();
        assert!(!stream.is_finished());
        assert_eq!(stream.push(100, false, false, b"hello").unwrap(), b"hello");
        assert!(stream.is_finished());
        assert!(stream.push(106, false, false, b"!").unwrap().is_empty());
    }

    #[test]
    fn pending_fin_is_a_terminal_boundary_for_buffered_data() {
        let mut stream = TcpReassembler::default();

        stream.push(99, true, false, b"").unwrap();
        stream.push(105, false, true, b"").unwrap();
        stream.push(103, false, false, b"XYZW").unwrap();

        assert_eq!(stream.push(100, false, false, b"abc").unwrap(), b"abcXY");
        assert!(stream.is_finished());
    }

    #[test]
    fn pending_fin_markers_are_bounded() {
        let mut stream = TcpReassembler::default();
        stream.push(99, true, false, b"").unwrap();

        for offset in 1..=MAX_PENDING_SEGMENTS as u32 {
            stream.push(100 + offset, false, true, b"").unwrap();
        }

        assert_eq!(
            stream
                .push(101 + MAX_PENDING_SEGMENTS as u32, false, true, b"")
                .unwrap_err(),
            TcpReassemblyError::SegmentLimitExceeded {
                segments: MAX_PENDING_SEGMENTS + 1,
                maximum: MAX_PENDING_SEGMENTS,
            }
        );
    }
}
