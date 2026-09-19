use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

// Claude Code files a worktree session under the worktree's own directory, so two
// sessions on one repo look like unrelated projects. Folding the worktree tail back
// into the checkout it branched from is what puts those sessions in the same pen.
const WORKTREE_SEGMENTS: [&str; 2] = [".claude", "worktrees"];

/// The group an agent session belongs to: one pen per repository checkout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct Pen {
    pub id: String,
    pub name: String,
}

pub fn pen_for_cwd(cwd: &Path) -> Option<Pen> {
    let root = worktree_root(cwd).unwrap_or(cwd);
    let name = root.file_name()?.to_str()?;
    if name.is_empty() {
        return None;
    }
    Some(Pen {
        id: root.to_str()?.to_string(),
        name: name.to_string(),
    })
}

fn worktree_root(cwd: &Path) -> Option<&Path> {
    let names: Vec<&str> = cwd
        .components()
        .map(|component| match component {
            Component::Normal(name) => name.to_str().unwrap_or_default(),
            _ => "",
        })
        .collect();

    let marker = names
        .windows(WORKTREE_SEGMENTS.len())
        .position(|window| window == WORKTREE_SEGMENTS)?;

    let mut root = cwd;
    for _ in marker..names.len() {
        root = root.parent()?;
    }
    Some(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn pen(cwd: &str) -> Option<Pen> {
        pen_for_cwd(&PathBuf::from(cwd))
    }

    #[test]
    fn plain_checkout_is_its_own_pen() {
        assert_eq!(
            pen("/Users/me/Code/adhd-ranch"),
            Some(Pen {
                id: "/Users/me/Code/adhd-ranch".into(),
                name: "adhd-ranch".into(),
            })
        );
    }

    #[test]
    fn worktree_shares_the_pen_of_its_checkout() {
        assert_eq!(
            pen("/Users/me/Code/adhd-ranch/.claude/worktrees/animals-7d5c3c"),
            pen("/Users/me/Code/adhd-ranch")
        );
    }

    #[test]
    fn two_worktrees_of_one_checkout_share_a_pen() {
        assert_eq!(
            pen("/Users/me/Code/adhd-ranch/.claude/worktrees/one-aaa"),
            pen("/Users/me/Code/adhd-ranch/.claude/worktrees/two-bbb")
        );
    }

    #[test]
    fn worktrees_of_different_checkouts_get_different_pens() {
        assert_ne!(
            pen("/Users/me/Code/one/.claude/worktrees/shared-name"),
            pen("/Users/me/Code/two/.claude/worktrees/shared-name")
        );
    }

    #[test]
    fn nested_path_below_a_worktree_still_folds_to_the_checkout() {
        assert_eq!(
            pen("/Users/me/Code/adhd-ranch/.claude/worktrees/one-aaa/crates/domain"),
            pen("/Users/me/Code/adhd-ranch")
        );
    }

    #[test]
    fn a_claude_dir_that_is_not_a_worktree_is_left_alone() {
        assert_eq!(
            pen("/Users/me/Code/adhd-ranch/.claude/agents"),
            Some(Pen {
                id: "/Users/me/Code/adhd-ranch/.claude/agents".into(),
                name: "agents".into(),
            })
        );
    }

    #[test]
    fn the_filesystem_root_has_no_pen() {
        assert_eq!(pen("/"), None);
    }
}
