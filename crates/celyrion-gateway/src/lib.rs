//! Gateway: the authenticated edge of the engine. Every inbound request is
//! attributed to an [`AuthContext`], appended to a [`ReplayLog`] so it can be
//! re-driven deterministically, and then handed to the runtime.

#![forbid(unsafe_code)]

use celyrion_protocol::Message;
use celyrion_types::{Error, Result};

/// Who a request is acting on behalf of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthContext {
    /// Authenticated principal.
    pub principal: String,
    /// Scopes granted to the principal.
    pub scopes: Vec<String>,
}

impl AuthContext {
    /// Whether the context holds `scope`.
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }
}

/// Bounded, append-only log of accepted requests for later replay.
#[derive(Debug)]
pub struct ReplayLog {
    entries: Vec<(AuthContext, Message)>,
    capacity: usize,
}

impl ReplayLog {
    /// Creates a log holding at most `capacity` entries.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity,
        }
    }

    /// Appends an entry, failing when the log is full.
    pub fn append(&mut self, ctx: AuthContext, message: Message) -> Result<()> {
        if self.entries.len() >= self.capacity {
            return Err(Error::CapacityExceeded(format!(
                "replay log of {}",
                self.capacity
            )));
        }
        self.entries.push((ctx, message));
        Ok(())
    }

    /// Iterates over the recorded entries in order.
    pub fn iter(&self) -> impl Iterator<Item = &(AuthContext, Message)> {
        self.entries.iter()
    }
}

/// Handles authenticated requests.
pub trait Gateway: Send {
    /// Processes one request and produces a response message.
    fn handle(
        &mut self,
        ctx: &AuthContext,
        request: Message,
    ) -> impl Future<Output = Result<Message>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;
    use celyrion_protocol::{MessageKind, ProtocolVersion};
    use celyrion_types::{Digest, VersionedId};

    #[test]
    fn replay_log_is_bounded() {
        let ctx = AuthContext {
            principal: "p".into(),
            scopes: vec!["infer".into()],
        };
        assert!(ctx.has_scope("infer"));
        let msg = Message {
            version: ProtocolVersion::CURRENT,
            id: VersionedId::new("req", 1),
            kind: MessageKind::Request,
            digest: Digest::ZERO,
            payload: Vec::new(),
        };
        let mut log = ReplayLog::new(1);
        log.append(ctx.clone(), msg.clone()).unwrap();
        assert!(log.append(ctx, msg).is_err());
        assert_eq!(log.iter().count(), 1);
    }
}
