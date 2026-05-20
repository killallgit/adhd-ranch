use std::path::Path;

use adhd_ranch_commands::{Commands, CreateFocusInput};

const EXAMPLE_MARKER: &str = ".example-focus-seeded";
const EXAMPLE_TITLE: &str = "Welcome to ADHD Ranch";

pub fn ensure_example_focus(
    commands: &Commands,
    focuses_root: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let marker = focuses_root.join(EXAMPLE_MARKER);
    let focuses = commands.list_focuses()?;
    if !focuses.is_empty() {
        mark_seeded(&marker)?;
        return Ok(());
    }
    if marker.exists() {
        return Ok(());
    }

    let created = commands.create_focus(CreateFocusInput {
        title: EXAMPLE_TITLE.to_string(),
        description: "Example focus created on first launch so the ranch has one animal to show."
            .to_string(),
        timer_preset: None,
    })?;
    commands.append_task(&created.id, "Click the animal to open this task card")?;
    commands.append_task(
        &created.id,
        "Use + New Focus from the tray to add your own work",
    )?;
    mark_seeded(&marker)?;
    Ok(())
}

fn mark_seeded(marker: &Path) -> std::io::Result<()> {
    std::fs::write(marker, b"1\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use adhd_ranch_commands::Commands;
    use adhd_ranch_domain::Settings;
    use adhd_ranch_storage::{JsonlDecisionLog, JsonlProposalQueue, MarkdownFocusStore};
    use std::sync::Arc;
    use tempfile::TempDir;

    fn commands(root: &Path) -> Commands {
        let focuses_root = root.join("focuses");
        std::fs::create_dir_all(&focuses_root).unwrap();
        Commands::new(
            Arc::new(MarkdownFocusStore::new(focuses_root)),
            Arc::new(JsonlProposalQueue::new(root.join("proposals.jsonl"))),
            Arc::new(JsonlDecisionLog::new(root.join("decisions.jsonl"))),
            Arc::new(|| "2026-05-20T00:00:00Z".to_string()),
            Arc::new(|| 1_779_235_200),
            Arc::new(|| "example-id".to_string()),
            Settings::default(),
        )
    }

    #[test]
    fn creates_example_focus_once_when_store_is_empty() {
        let dir = TempDir::new().unwrap();
        let commands = commands(dir.path());
        let focuses_root = dir.path().join("focuses");

        ensure_example_focus(&commands, &focuses_root).unwrap();
        ensure_example_focus(&commands, &focuses_root).unwrap();

        let focuses = commands.list_focuses().unwrap();
        assert_eq!(focuses.len(), 1);
        assert_eq!(focuses[0].title, EXAMPLE_TITLE);
        assert_eq!(focuses[0].tasks.len(), 2);
        assert!(focuses_root.join(EXAMPLE_MARKER).is_file());
    }

    #[test]
    fn does_not_recreate_example_after_user_clears_all_focuses() {
        let dir = TempDir::new().unwrap();
        let commands = commands(dir.path());
        let focuses_root = dir.path().join("focuses");

        ensure_example_focus(&commands, &focuses_root).unwrap();
        let focus_id = commands.list_focuses().unwrap()[0].id.0.clone();
        commands.delete_focus(&focus_id).unwrap();
        ensure_example_focus(&commands, &focuses_root).unwrap();

        assert!(commands.list_focuses().unwrap().is_empty());
    }
}
