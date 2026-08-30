//! Bounded Server-Sent Events parsing used by all streaming endpoints.
//!
//! Rainy streams use ordinary SSE framing (`data:` lines separated by a blank
//! line).  Keeping the parser here avoids endpoint-specific buffering rules
//! and makes the safety limit apply equally to Chat, Responses, and Messages.

use crate::error::{RainyError, Result};
use futures::{Stream, StreamExt, stream};
use std::collections::VecDeque;
use std::fmt::Display;
use std::pin::Pin;

/// Maximum size of one logical SSE event, including its fields.
pub(crate) const MAX_SSE_EVENT_BYTES: usize = 8 * 1024 * 1024;

/// Maximum size retained while waiting for a line terminator.
const MAX_SSE_LINE_BYTES: usize = MAX_SSE_EVENT_BYTES;

/// Parsed SSE event.  A `[DONE]` data frame is represented explicitly so
/// endpoint adapters cannot accidentally try to deserialize it as JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SseEvent {
    /// Optional native SSE event name.
    pub(crate) event: Option<String>,
    /// The concatenated `data:` field contents.
    pub(crate) data: String,
    /// Whether the event is the protocol terminal marker.
    pub(crate) done: bool,
}

#[derive(Debug, Default)]
pub(crate) struct SseParser {
    line: Vec<u8>,
    event_name: Option<String>,
    data_lines: Vec<String>,
    event_bytes: usize,
    skip_lf_after_cr: bool,
    done_seen: bool,
}

impl SseParser {
    /// Adds a network chunk and returns every complete event found in it.
    pub(crate) fn push(&mut self, chunk: &[u8]) -> Result<Vec<SseEvent>> {
        let mut events = Vec::new();

        if self.done_seen {
            return Ok(events);
        }

        for &byte in chunk {
            if self.done_seen {
                break;
            }
            if self.skip_lf_after_cr {
                self.skip_lf_after_cr = false;
                if byte == b'\n' {
                    continue;
                }
            }

            match byte {
                b'\n' => self.finish_line(&mut events)?,
                b'\r' => {
                    self.finish_line(&mut events)?;
                    self.skip_lf_after_cr = true;
                }
                _ => {
                    self.line.push(byte);
                    self.event_bytes = self.event_bytes.saturating_add(1);
                    self.ensure_size()?;
                }
            }
        }

        Ok(events)
    }

    /// Flushes a final unterminated line and event, if present.
    pub(crate) fn finish(&mut self) -> Result<Vec<SseEvent>> {
        let mut events = Vec::new();
        if self.done_seen {
            return Ok(events);
        }
        if !self.line.is_empty() {
            self.finish_line(&mut events)?;
        }
        self.dispatch(&mut events);
        Ok(events)
    }

    fn ensure_size(&self) -> Result<()> {
        if self.event_bytes > MAX_SSE_EVENT_BYTES || self.line.len() > MAX_SSE_LINE_BYTES {
            return Err(RainyError::PayloadTooLarge {
                message: "SSE event exceeds the configured safety limit".to_string(),
                max_bytes: MAX_SSE_EVENT_BYTES,
            });
        }
        Ok(())
    }

    fn finish_line(&mut self, events: &mut Vec<SseEvent>) -> Result<()> {
        self.event_bytes = self.event_bytes.saturating_add(1);
        self.ensure_size()?;

        let line = std::mem::take(&mut self.line);
        if line.is_empty() {
            self.dispatch(events);
            return Ok(());
        }

        // SSE comments are heartbeats and do not contribute to the event.
        if line[0] == b':' {
            return Ok(());
        }

        let separator = line.iter().position(|byte| *byte == b':');
        let (field, value) = match separator {
            Some(index) => (&line[..index], &line[index + 1..]),
            None => (line.as_slice(), &[][..]),
        };
        let value = value.strip_prefix(b" ").unwrap_or(value);
        let field = std::str::from_utf8(field).map_err(|_error| RainyError::Serialization {
            message: "SSE field is not valid UTF-8".to_string(),
            source_error: None,
        })?;
        let value = std::str::from_utf8(value).map_err(|_error| RainyError::Serialization {
            message: "SSE field value is not valid UTF-8".to_string(),
            source_error: None,
        })?;

        match field {
            "event" => self.event_name = Some(value.to_string()),
            "data" => {
                self.data_lines.push(value.to_string());
                self.ensure_size()?;
            }
            // `id` and `retry` are valid SSE fields but have no meaning for
            // one-shot Rainy inference streams.
            "id" | "retry" => {}
            _ => {}
        }

        Ok(())
    }

    fn dispatch(&mut self, events: &mut Vec<SseEvent>) {
        if self.event_name.is_none() && self.data_lines.is_empty() {
            self.event_bytes = 0;
            return;
        }

        let data = self.data_lines.join("\n");
        let done = data.trim().eq_ignore_ascii_case("[done]");
        events.push(SseEvent {
            event: self.event_name.take(),
            data,
            done,
        });
        self.done_seen |= done;
        self.data_lines.clear();
        self.event_bytes = 0;
    }
}

struct StreamState<S> {
    input: Pin<Box<S>>,
    parser: SseParser,
    pending: VecDeque<Result<SseEvent>>,
    finished: bool,
}

/// Converts a byte stream into one bounded, protocol-aware SSE stream.
pub(crate) fn parse_sse_stream<S, B, E>(
    input: S,
) -> Pin<Box<dyn Stream<Item = Result<SseEvent>> + Send>>
where
    S: Stream<Item = std::result::Result<B, E>> + Send + 'static,
    B: AsRef<[u8]> + Send + 'static,
    E: Display + Send + 'static,
{
    let state = StreamState {
        input: Box::pin(input),
        parser: SseParser::default(),
        pending: VecDeque::new(),
        finished: false,
    };

    let output = stream::unfold(state, |mut state| async move {
        loop {
            if let Some(event) = state.pending.pop_front() {
                return Some((event, state));
            }

            if state.finished {
                return None;
            }

            match state.input.next().await {
                Some(Ok(chunk)) => match state.parser.push(chunk.as_ref()) {
                    Ok(mut events) => {
                        if let Some(index) = events.iter().position(|event| event.done) {
                            events.truncate(index + 1);
                            state.finished = true;
                        }
                        state.pending.extend(events.into_iter().map(Ok));
                    }
                    Err(error) => {
                        state.finished = true;
                        return Some((Err(error), state));
                    }
                },
                Some(Err(_error)) => {
                    state.finished = true;
                    return Some((
                        Err(RainyError::Network {
                            message: "SSE stream failed".to_string(),
                            retryable: false,
                            source_error: None,
                        }),
                        state,
                    ));
                }
                None => {
                    state.finished = true;
                    match state.parser.finish() {
                        Ok(mut events) => {
                            if let Some(index) = events.iter().position(|event| event.done) {
                                events.truncate(index + 1);
                            }
                            state.pending.extend(events.into_iter().map(Ok));
                        }
                        Err(error) => return Some((Err(error), state)),
                    }
                }
            }
        }
    });

    Box::pin(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;

    #[test]
    fn parses_fragmented_crlf_frames_and_done() {
        let mut parser = SseParser::default();
        let mut events = parser.push(b"event: rainy.billing\r\nda").unwrap();
        assert!(events.is_empty());
        events.extend(parser.push(b"ta: {\"charged_credits\":1}\r\n\r\n").unwrap());
        events.extend(parser.push(b"data: [DONE]\n\n").unwrap());

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event.as_deref(), Some("rainy.billing"));
        assert_eq!(events[0].data, r#"{"charged_credits":1}"#);
        assert!(!events[0].done);
        assert!(events[1].done);
    }

    #[test]
    fn joins_multiple_data_lines_and_ignores_comments() {
        let mut parser = SseParser::default();
        let events = parser
            .push(b": heartbeat\ndata: {\"a\":\ndata: 1}\n\n")
            .unwrap();
        assert_eq!(events[0].data, "{\"a\":\n1}");
    }

    #[test]
    fn rejects_oversized_event() {
        let mut parser = SseParser::default();
        let error = parser
            .push(&vec![b'x'; MAX_SSE_EVENT_BYTES + 1])
            .expect_err("event must be bounded");
        assert!(matches!(error, RainyError::PayloadTooLarge { .. }));
    }

    #[tokio::test]
    async fn parses_stream_errors_without_panicking() {
        let input = stream::iter(vec![Ok::<_, std::io::Error>(b"data: {}\n\n".to_vec())]);
        let mut output = parse_sse_stream(input);
        let event = output.next().await.expect("event").expect("valid event");
        assert_eq!(event.data, "{}");
    }

    #[tokio::test]
    async fn stops_after_done_and_discards_trailing_frames() {
        let input = stream::iter(vec![Ok::<_, std::io::Error>(
            b"data: [DONE]\n\ndata: {\"after\":true}\n\n".to_vec(),
        )]);
        let events = parse_sse_stream(input).collect::<Vec<_>>().await;
        assert_eq!(events.len(), 1);
        assert!(events[0].as_ref().expect("valid event").done);
    }
}
