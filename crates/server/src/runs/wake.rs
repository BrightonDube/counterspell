//! `RunWaker`: one Postgres listener that tells open event streams when a run has new events (design Path 9).
//!
//! The listener reads `NOTIFY run_events`. The payload is only a run id. A wake-up is a hint: the stream
//! then reads the events from Postgres itself. Nothing in this module carries event data, so no event
//! can reach a client through a notification.

use std::{str::FromStr, time::Duration};

use counterspell_common::ids::RunId;
use sqlx::{postgres::PgListener, PgPool};
use tokio::sync::broadcast;

/// The Postgres channel that the judge notifies after every event it writes.
pub const RUN_EVENTS_CHANNEL: &str = "run_events";

/// How many wake-ups a slow receiver may fall behind before it lags.
const CHANNEL_CAPACITY: usize = 1024;

/// How long the listener waits before it connects again after an error.
const RECONNECT_DELAY: Duration = Duration::from_secs(1);

/// One wake-up for the streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wake {
    /// This run may have new events. Streams check whether it is theirs.
    Run(RunId),
    /// The listener reconnected, so events may have been missed. Every stream should read again.
    All,
}

impl Wake {
    /// Whether this wake-up means a stream for `run` should read its events now.
    #[must_use]
    pub fn concerns(self, run: RunId) -> bool {
        match self {
            Self::Run(id) => id == run,
            Self::All => true,
        }
    }
}

/// Shares wake-ups with every open stream. Cloning gives another handle to the same channel.
#[derive(Clone)]
pub struct RunWaker {
    sender: broadcast::Sender<Wake>,
}

impl RunWaker {
    /// A waker with no listener. Nothing is ever sent, so streams rely on their poll timer alone.
    /// Used as the default state and in tests that check the poll-only path.
    #[must_use]
    pub fn idle() -> Self {
        let (sender, _keep_alive) = broadcast::channel(CHANNEL_CAPACITY);
        Self { sender }
    }

    /// Starts the listener task on `pool` and returns a handle to its wake-ups.
    ///
    /// The task runs until the process exits. It sends [`Wake::All`] after each successful connection.
    #[must_use]
    pub fn start(pool: PgPool) -> Self {
        let (sender, _keep_alive) = broadcast::channel(CHANNEL_CAPACITY);
        tokio::spawn(listen_forever(pool, sender.clone()));
        Self { sender }
    }

    /// Returns a receiver for wake-ups. Each stream subscribes before it reads its first events.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<Wake> {
        self.sender.subscribe()
    }
}

/// Keeps one listener connection open, sends wake-ups, and reconnects after any error.
async fn listen_forever(pool: PgPool, sender: broadcast::Sender<Wake>) {
    loop {
        match PgListener::connect_with(&pool).await {
            Ok(mut listener) => {
                if let Err(error) = listener.listen(RUN_EVENTS_CHANNEL).await {
                    tracing::warn!(%error, "could not listen for run events; retrying");
                } else {
                    // Anything written while the connection was down is found by reading again.
                    let _ = sender.send(Wake::All);
                    relay(&mut listener, &sender).await;
                }
            }
            Err(error) => tracing::warn!(%error, "run event listener could not connect; retrying"),
        }
        tokio::time::sleep(RECONNECT_DELAY).await;
    }
}

/// Forwards each notification's run id. When the connection drops, sqlx reconnects on its own, and this
/// sends [`Wake::All`] so streams read again, because notifications sent during the gap were lost.
async fn relay(listener: &mut PgListener, sender: &broadcast::Sender<Wake>) {
    loop {
        match listener.try_recv().await {
            Ok(Some(notification)) => {
                // A payload that is not a run id is ignored; it cannot carry event data anyway.
                if let Ok(run) = RunId::from_str(notification.payload()) {
                    // No receivers is fine: nobody is streaming right now.
                    let _ = sender.send(Wake::Run(run));
                }
            }
            Ok(None) => {
                tracing::warn!("run event listener lost its connection; sqlx is reconnecting");
                let _ = sender.send(Wake::All);
            }
            Err(error) => {
                tracing::warn!(%error, "run event listener failed; reconnecting");
                return;
            }
        }
    }
}
