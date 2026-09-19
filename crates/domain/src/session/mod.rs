//! Animals projected from running coding-agent sessions.
//!
//! Nothing here shares state with `focus` or `timer`: a session animal has no timer,
//! no duration and no expiry. The two only meet on the overlay, where they are drawn
//! and moved by the same animation.
//!
//! These types are the ranch's own vocabulary. What an agent actually writes to disk,
//! and how that maps onto these types, belongs in [`crate::agents`].

pub mod activity;
pub mod agent_session;
pub mod pen;

pub use activity::SessionActivity;
pub use agent_session::AgentSession;
pub use pen::{pen_for_cwd, Pen};
