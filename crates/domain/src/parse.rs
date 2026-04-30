use crate::focus::{Focus, FocusId, Task};

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    MissingFrontmatter,
    UnterminatedFrontmatter,
    MissingField(&'static str),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingFrontmatter => f.write_str("missing frontmatter"),
            Self::UnterminatedFrontmatter => f.write_str("unterminated frontmatter"),
            Self::MissingField(field) => write!(f, "missing field: {field}"),
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse_focus_md(input: &str) -> Result<Focus, ParseError> {
    let (frontmatter, body) = split_frontmatter(input)?;
    let mut id: Option<String> = None;
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut created_at: Option<String> = None;

    for line in frontmatter.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let value = unquote(value.trim()).to_string();
        match key {
            "id" => id = Some(value),
            "title" => title = Some(value),
            "description" => description = Some(value),
            "created_at" => created_at = Some(value),
            _ => {}
        }
    }

    let id = id.ok_or(ParseError::MissingField("id"))?;
    let title = title.ok_or(ParseError::MissingField("title"))?;
    let description = description.unwrap_or_default();
    let created_at = created_at.unwrap_or_default();
    let tasks = parse_tasks(body, &id);

    Ok(Focus {
        id: FocusId(id),
        title,
        description,
        created_at,
        tasks,
    })
}

fn split_frontmatter(input: &str) -> Result<(&str, &str), ParseError> {
    let stripped = input
        .strip_prefix("---")
        .ok_or(ParseError::MissingFrontmatter)?;
    let after_open = stripped.strip_prefix('\n').unwrap_or(stripped);
    let close = after_open
        .find("\n---")
        .ok_or(ParseError::UnterminatedFrontmatter)?;
    let frontmatter = &after_open[..close];
    let rest = &after_open[close + "\n---".len()..];
    let body = rest.strip_prefix('\n').unwrap_or(rest);
    Ok((frontmatter, body))
}

fn unquote(value: &str) -> &str {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            return &value[1..value.len() - 1];
        }
    }
    value
}

fn parse_tasks(body: &str, focus_id: &str) -> Vec<Task> {
    body.lines()
        .filter_map(|line| {
            let line = line.trim_start();
            let rest = line
                .strip_prefix("- [ ]")
                .or_else(|| line.strip_prefix("- [x]"))?;
            let text = rest.trim().to_string();
            if text.is_empty() {
                return None;
            }
            Some(text)
        })
        .enumerate()
        .map(|(index, text)| Task {
            id: format!("{focus_id}:{index}"),
            text,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(id: &str, title: &str, description: &str, body: &str) -> String {
        format!(
            "---\nid: {id}\ntitle: {title}\ndescription: {description}\ncreated_at: 2026-04-30T12:00:00Z\n---\n{body}"
        )
    }

    #[test]
    fn parses_frontmatter_and_tasks() {
        let src = fixture(
            "customer-x-bug",
            "Customer X bug",
            "ship the fix",
            "- [ ] add persistence field\n- [ ] update sdk\n",
        );
        let focus = parse_focus_md(&src).expect("parse ok");
        assert_eq!(focus.id, FocusId("customer-x-bug".into()));
        assert_eq!(focus.title, "Customer X bug");
        assert_eq!(focus.description, "ship the fix");
        assert_eq!(focus.tasks.len(), 2);
        assert_eq!(focus.tasks[0].id, "customer-x-bug:0");
        assert_eq!(focus.tasks[0].text, "add persistence field");
        assert_eq!(focus.tasks[1].text, "update sdk");
    }

    #[test]
    fn empty_body_yields_zero_tasks() {
        let src = fixture("a", "A", "", "");
        let focus = parse_focus_md(&src).unwrap();
        assert!(focus.tasks.is_empty());
    }

    #[test]
    fn ignores_non_checkbox_bullets_and_empty_lines() {
        let src = fixture(
            "a",
            "A",
            "",
            "- regular bullet\n\nrandom prose\n- [ ] real task\n",
        );
        let focus = parse_focus_md(&src).unwrap();
        assert_eq!(focus.tasks.len(), 1);
        assert_eq!(focus.tasks[0].text, "real task");
    }

    #[test]
    fn checked_box_still_counts_as_task_for_now() {
        let src = fixture("a", "A", "", "- [x] done one\n- [ ] open one\n");
        let focus = parse_focus_md(&src).unwrap();
        assert_eq!(focus.tasks.len(), 2);
    }

    #[test]
    fn quoted_values_in_frontmatter_unwrap() {
        let src = "---\nid: \"a\"\ntitle: 'B C'\ndescription:\ncreated_at:\n---\n";
        let focus = parse_focus_md(src).unwrap();
        assert_eq!(focus.id, FocusId("a".into()));
        assert_eq!(focus.title, "B C");
        assert_eq!(focus.description, "");
    }

    #[test]
    fn missing_frontmatter_errors() {
        assert_eq!(
            parse_focus_md("just a body\n").unwrap_err(),
            ParseError::MissingFrontmatter
        );
    }

    #[test]
    fn unterminated_frontmatter_errors() {
        assert_eq!(
            parse_focus_md("---\nid: a\ntitle: A\n").unwrap_err(),
            ParseError::UnterminatedFrontmatter
        );
    }

    #[test]
    fn missing_required_id_errors() {
        let src = "---\ntitle: A\n---\n";
        assert_eq!(
            parse_focus_md(src).unwrap_err(),
            ParseError::MissingField("id")
        );
    }

    #[test]
    fn missing_required_title_errors() {
        let src = "---\nid: a\n---\n";
        assert_eq!(
            parse_focus_md(src).unwrap_err(),
            ParseError::MissingField("title")
        );
    }
}
