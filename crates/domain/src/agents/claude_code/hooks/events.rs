use crate::agents::hooks::HookAction;

/// One Claude Code hook event and what the ranch reads it as.
///
/// There is deliberately no matcher field. A matcher narrows an event to certain
/// tools, which is only useful for the per-tool events this table refuses to
/// register; leaving it out makes the mistake unrepresentable rather than merely
/// discouraged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionHook {
    pub event: &'static str,
    pub action: HookAction,
}

/// The smallest set that answers "is this session working right now?".
///
/// No `PreToolUse`/`PostToolUse`: they fire on every tool call in every session on
/// the machine, and they report a state the session is already in — a turn is
/// `Working` from `UserPromptSubmit` until it ends, whatever it does in between.
///
/// No `SubagentStart`/`SubagentStop` for the same reason: the parent turn has not
/// ended while a subagent runs. They become useful when a subagent gets its own
/// animal, and that is the point at which to add them.
pub const SESSION_HOOKS: [SessionHook; 5] = [
    SessionHook {
        event: "SessionStart",
        action: HookAction::Start,
    },
    SessionHook {
        event: "UserPromptSubmit",
        action: HookAction::Working,
    },
    SessionHook {
        event: "Stop",
        action: HookAction::Idle,
    },
    // A turn cut short by an API error ends on `StopFailure` and never reaches
    // `Stop`. Without this row the session stays `Working` for as long as it lives.
    SessionHook {
        event: "StopFailure",
        action: HookAction::Idle,
    },
    SessionHook {
        event: "SessionEnd",
        action: HookAction::End,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_hook_fires_per_tool_call() {
        for hook in SESSION_HOOKS {
            assert!(
                !hook.event.contains("ToolUse"),
                "{} is per-tool",
                hook.event
            );
        }
    }

    #[test]
    fn a_turn_can_both_begin_and_end() {
        let actions: Vec<_> = SESSION_HOOKS.iter().map(|h| h.action).collect();
        assert!(actions.contains(&HookAction::Working));
        assert!(actions.contains(&HookAction::Idle));
    }

    #[test]
    fn an_api_error_still_ends_the_turn() {
        let stop_failure = SESSION_HOOKS
            .iter()
            .find(|h| h.event == "StopFailure")
            .expect("StopFailure must be registered");
        assert_eq!(stop_failure.action, HookAction::Idle);
    }
}
