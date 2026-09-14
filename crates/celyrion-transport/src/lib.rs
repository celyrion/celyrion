//! Transport: moves [`celyrion_protocol::Message`]s and state between
//! components, whether they share a process, a host, or a network.

#![forbid(unsafe_code)]

use celyrion_protocol::Message;
use celyrion_types::{Digest, Result, VersionedId};
use tokio::sync::mpsc;

/// Address of a transport endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Endpoint(pub String);

/// A request to move a state bundle from one endpoint to another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateMovement {
    /// Bundle being moved.
    pub bundle: Digest,
    /// Where it currently lives.
    pub from: Endpoint,
    /// Where it should end up.
    pub to: Endpoint,
    /// Schema of the bundle, for validation on arrival.
    pub schema: VersionedId,
}

/// Bidirectional message channel.
pub trait Transport: Send {
    /// Sends a message to `to`.
    fn send(&mut self, to: &Endpoint, message: Message) -> impl Future<Output = Result<()>> + Send;

    /// Receives the next inbound message, or `None` when the transport closes.
    fn recv(&mut self) -> impl Future<Output = Option<(Endpoint, Message)>> + Send;
}

/// In-process transport that loops every send back to its own receive side.
#[derive(Debug)]
pub struct Loopback {
    tx: mpsc::Sender<(Endpoint, Message)>,
    rx: mpsc::Receiver<(Endpoint, Message)>,
}

impl Loopback {
    /// Creates a loopback transport with the given buffer size.
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity.max(1));
        Self { tx, rx }
    }
}

impl Transport for Loopback {
    async fn send(&mut self, to: &Endpoint, message: Message) -> Result<()> {
        self.tx
            .send((to.clone(), message))
            .await
            .map_err(|e| celyrion_types::Error::Other(Box::new(e)))
    }

    async fn recv(&mut self) -> Option<(Endpoint, Message)> {
        self.rx.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use celyrion_protocol::{MessageKind, ProtocolVersion};

    #[tokio::test]
    async fn loopback_round_trips() {
        let mut transport = Loopback::new(1);
        let message = Message {
            version: ProtocolVersion::CURRENT,
            id: VersionedId::new("ping", 1),
            kind: MessageKind::Event,
            digest: Digest::ZERO,
            payload: vec![1],
        };
        transport
            .send(&Endpoint("self".into()), message.clone())
            .await
            .unwrap();
        let (to, received) = transport.recv().await.unwrap();
        assert_eq!(to, Endpoint("self".into()));
        assert_eq!(received, message);
    }
}
