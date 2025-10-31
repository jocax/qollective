// The NatsClient in src/nats/client.rs already provides all NATS functionality.
// We'll add a trait implementation for it in the nats module itself.
// This file exists for module structure consistency.

pub use crate::nats::NatsClient as NatsServiceImpl;
