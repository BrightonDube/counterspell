//! A small SSE client for the integration tests. It reads the real HTTP response from a socket, the way a
//! browser does, and splits the bytes into SSE frames. It never looks at `NOTIFY`, so a test sees only
//! what the server actually sends.

use std::{net::SocketAddr, time::Duration};

use http_body_util::{BodyExt, Empty};
use hyper::{
    body::{Bytes, Incoming},
    client::conn::http1,
    Request, Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use tokio::{net::TcpStream, time::timeout};

/// One frame of an SSE stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frame {
    /// An event. `id` is the sequence number, `event` the type and `data` the JSON envelope.
    Event {
        /// The `id:` line, if present.
        id: Option<String>,
        /// The `event:` line, if present.
        event: Option<String>,
        /// The `data:` lines joined with newlines.
        data: String,
    },
    /// A comment line such as the heartbeat, without the leading colon.
    Comment(String),
}

impl Frame {
    /// The sequence number of an event frame, or `None` for a comment or a frame with no numeric id.
    #[must_use]
    pub fn seq(&self) -> Option<i64> {
        match self {
            Self::Event { id: Some(id), .. } => id.parse().ok(),
            _ => None,
        }
    }

    /// The event type of an event frame, or `None` for a comment.
    #[must_use]
    pub fn kind(&self) -> Option<&str> {
        match self {
            Self::Event { event, .. } => event.as_deref(),
            Self::Comment(_) => None,
        }
    }

    /// The parsed JSON envelope of an event frame. Panics on a comment, which has no data.
    #[must_use]
    pub fn envelope(&self) -> serde_json::Value {
        match self {
            Self::Event { data, .. } => serde_json::from_str(data).expect("event data is JSON"),
            Self::Comment(text) => panic!("a comment has no envelope: {text}"),
        }
    }
}

/// An open response. Its body is read as frames; dropping it closes the connection, as a browser does.
pub struct SseResponse {
    /// The HTTP status of the response.
    pub status: StatusCode,
    /// The response headers.
    pub headers: hyper::HeaderMap,
    body: Option<Incoming>,
    text: String,
}

impl SseResponse {
    /// Sends `GET path` with extra headers and returns once the status and headers have arrived.
    ///
    /// # Panics
    ///
    /// Panics when the connection or the request fails. Tests call this only against a local server.
    pub async fn open(address: SocketAddr, path: &str, headers: &[(&str, &str)]) -> Self {
        let stream = TcpStream::connect(address)
            .await
            .expect("connect to the test server");
        let (mut sender, connection) = http1::handshake(TokioIo::new(stream))
            .await
            .expect("http/1 handshake");
        tokio::spawn(async move {
            let _ = connection.await;
        });

        let mut builder = Request::builder()
            .method("GET")
            .uri(path)
            .header("host", "localhost");
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        let request = builder.body(Empty::<Bytes>::new()).expect("request");
        let response: Response<Incoming> = sender.send_request(request).await.expect("response");
        Self {
            status: response.status(),
            headers: response.headers().clone(),
            body: Some(response.into_body()),
            text: String::new(),
        }
    }

    /// Reads the next frame, or `None` once the body has ended.
    pub async fn next_frame(&mut self) -> Option<Frame> {
        loop {
            if let Some(end) = self.text.find("\n\n") {
                let block: String = self.text.drain(..end + 2).collect();
                if let Some(frame) = parse_block(&block) {
                    return Some(frame);
                }
                continue;
            }
            let body = self.body.as_mut()?;
            let next = body.frame().await;
            let Some(Ok(frame)) = next else {
                self.body = None;
                return None;
            };
            if let Ok(data) = frame.into_data() {
                self.text.push_str(&String::from_utf8_lossy(&data));
            }
        }
    }

    /// Reads the next frame within `limit`.
    ///
    /// # Panics
    ///
    /// Panics when no frame arrives in time, so a test fails instead of hanging.
    pub async fn next_frame_within(&mut self, limit: Duration) -> Option<Frame> {
        timeout(limit, self.next_frame())
            .await
            .expect("a frame should arrive within the time limit")
    }

    /// Reads the rest of the body as text. Used for JSON error replies.
    pub async fn read_body(mut self) -> String {
        let mut out = std::mem::take(&mut self.text);
        if let Some(mut body) = self.body.take() {
            while let Some(Ok(frame)) = body.frame().await {
                if let Ok(data) = frame.into_data() {
                    out.push_str(&String::from_utf8_lossy(&data));
                }
            }
        }
        out
    }
}

/// Reads one SSE block (the text before a blank line). Returns `None` for a block with nothing in it.
fn parse_block(block: &str) -> Option<Frame> {
    let trimmed = block.trim_end_matches('\n');
    if trimmed.is_empty() {
        return None;
    }
    if let Some(comment) = trimmed.strip_prefix(':') {
        return Some(Frame::Comment(comment.trim_start().to_owned()));
    }

    let mut id = None;
    let mut event = None;
    let mut data: Vec<&str> = Vec::new();
    for line in trimmed.lines() {
        if let Some(value) = line.strip_prefix("id:") {
            id = Some(value.trim_start().to_owned());
        } else if let Some(value) = line.strip_prefix("event:") {
            event = Some(value.trim_start().to_owned());
        } else if let Some(value) = line.strip_prefix("data:") {
            data.push(value.trim_start());
        }
    }
    if id.is_none() && event.is_none() && data.is_empty() {
        return None;
    }
    Some(Frame::Event {
        id,
        event,
        data: data.join("\n"),
    })
}
