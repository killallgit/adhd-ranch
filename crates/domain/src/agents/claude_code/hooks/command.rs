use crate::agents::hooks::HookAction;

/// A ceiling on damage rather than a budget. The client gives up on its own long
/// before this; the timeout only matters if it never gets to run, and a hook that
/// hangs holds up somebody's turn.
pub(super) const HOOK_TIMEOUT_SECS: u64 = 5;

/// What the ranch asks the agent to run, and where that lands.
///
/// Both paths are settled by the app at install time and written verbatim into the
/// agent's settings file, so this is the whole contract: the client needs no
/// configuration of its own and reads nothing to find us.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HookCommand<'a> {
    pub client_bin: &'a str,
    pub socket_path: &'a str,
}

impl HookCommand<'_> {
    pub fn for_action(&self, action: HookAction) -> String {
        format!(
            "{} {} {}",
            shell_quote(self.client_bin),
            shell_quote(self.socket_path),
            action.verb()
        )
    }
}

fn shell_quote(raw: &str) -> String {
    format!("'{}'", raw.replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMAND: HookCommand<'static> = HookCommand {
        client_bin: "/Applications/adhd-ranch.app/Contents/MacOS/adhd-ranch-hook",
        socket_path: "/Users/ryan/.adhd-ranch/agent-hooks.sock",
    };

    #[test]
    fn the_command_names_the_client_the_socket_and_the_action() {
        assert_eq!(
            COMMAND.for_action(HookAction::Working),
            "'/Applications/adhd-ranch.app/Contents/MacOS/adhd-ranch-hook' '/Users/ryan/.adhd-ranch/agent-hooks.sock' working"
        );
    }

    #[test]
    fn a_path_with_a_quote_cannot_break_out() {
        let command = HookCommand {
            client_bin: "/tmp/it's here/hook",
            socket_path: "/tmp/s.sock",
        };

        assert!(command
            .for_action(HookAction::Idle)
            .starts_with(r"'/tmp/it'\''s here/hook'"));
    }

    #[test]
    fn every_action_produces_a_distinct_command() {
        let commands: Vec<_> = [
            HookAction::Start,
            HookAction::Working,
            HookAction::Idle,
            HookAction::End,
        ]
        .iter()
        .map(|a| COMMAND.for_action(*a))
        .collect();

        let mut unique = commands.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), commands.len());
    }
}
