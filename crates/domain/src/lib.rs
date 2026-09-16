pub mod agent_session;
pub mod cap_monitor;
pub mod caps;
pub mod claude_hook;
pub mod error;
pub mod focus;
pub mod monitor;
pub mod new_focus;
pub mod notification;
pub mod parse;
pub mod pig_rect;
pub mod settings;
pub mod slug;
pub mod timer;
pub mod timer_ticker;

pub use agent_session::AgentSession;
pub use cap_monitor::{CapTransition, OverCapMonitor};
pub use caps::{cap_state, CapState};
pub use error::DomainError;
pub use focus::{Focus, FocusId, Task, TaskText};
pub use monitor::MonitorInfo;
pub use new_focus::NewFocus;
pub use notification::{
    all_sources, FocusesOverCapSource, NotificationSettings, NotificationSource,
    TaskTimerExpiredSource, TasksOverCapSource, TimerExpiredSource,
};
pub use parse::{parse_focus_md, ParseError};
pub use pig_rect::{PigRect, RectUpdater};
pub use settings::{AgentsConfig, Caps, DisplayConfig, Settings, Widget};
pub use slug::slugify;
pub use timer::{timer_remaining_secs, FocusTimer, TimerOwner, TimerPreset, TimerStatus};
pub use timer_ticker::{tick, TimerTransition, TimerTransitionTarget};
