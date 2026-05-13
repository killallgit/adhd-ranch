#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FocusDocumentError {
    TaskIndexOutOfRange { index: usize },
}

#[derive(Debug)]
pub(crate) struct FocusDocument {
    raw: String,
}

impl FocusDocument {
    pub(crate) fn from_raw(raw: String) -> Self {
        Self { raw }
    }

    pub(crate) fn into_raw(self) -> String {
        self.raw
    }

    pub(crate) fn rename_focus(mut self, title: &str) -> Self {
        let trailing_newline = self.raw.ends_with('\n');
        let mut out = String::with_capacity(self.raw.len());
        let mut in_frontmatter = false;
        let mut closed_frontmatter = false;
        let mut replaced = false;

        for (line_idx, line) in self.raw.lines().enumerate() {
            if line_idx == 0 && line == "---" {
                in_frontmatter = true;
                out.push_str(line);
                out.push('\n');
                continue;
            }
            if in_frontmatter && !closed_frontmatter && line == "---" {
                closed_frontmatter = true;
                out.push_str(line);
                out.push('\n');
                continue;
            }
            if in_frontmatter && !closed_frontmatter && !replaced {
                if let Some((key, _)) = line.split_once(':') {
                    if key.trim() == "title" {
                        out.push_str(&format!("title: {title}"));
                        out.push('\n');
                        replaced = true;
                        continue;
                    }
                }
            }
            out.push_str(line);
            out.push('\n');
        }
        if !trailing_newline {
            out.pop();
        }
        self.raw = out;
        self
    }

    pub(crate) fn append_task(mut self, text: &str) -> Self {
        if !self.raw.ends_with('\n') {
            self.raw.push('\n');
        }
        self.raw.push_str("- [ ] ");
        self.raw.push_str(text);
        self.raw.push('\n');
        self
    }

    pub(crate) fn delete_task(self, index: usize) -> Result<Self, FocusDocumentError> {
        let target = self.task_line_index(index)?;
        Ok(self.rewrite_lines(|line_idx, line| {
            if line_idx == target {
                None
            } else {
                Some(line.to_string())
            }
        }))
    }

    pub(crate) fn update_task(self, index: usize, text: &str) -> Result<Self, FocusDocumentError> {
        self.rewrite_task_line(index, |line| {
            let leading_ws: String = line.chars().take_while(|c| c.is_whitespace()).collect();
            let body = line.trim_start();
            let (marker, _rest) = if let Some(rest) = body.strip_prefix("- [ ]") {
                ("- [ ]", rest)
            } else if let Some(rest) = body.strip_prefix("- [x]") {
                ("- [x]", rest)
            } else {
                return line.to_string();
            };
            format!("{leading_ws}{marker} {text}", text = text.trim())
        })
    }

    pub(crate) fn toggle_task(self, index: usize, done: bool) -> Result<Self, FocusDocumentError> {
        self.rewrite_task_line(index, |line| {
            let leading_ws: String = line.chars().take_while(|c| c.is_whitespace()).collect();
            let body = line.trim_start();
            let new_marker = if done { "- [x]" } else { "- [ ]" };
            let rest = if let Some(rest) = body.strip_prefix("- [ ]") {
                rest
            } else if let Some(rest) = body.strip_prefix("- [x]") {
                rest
            } else {
                return line.to_string();
            };
            format!("{leading_ws}{new_marker}{rest}")
        })
    }

    fn rewrite_task_line<F>(self, index: usize, transform: F) -> Result<Self, FocusDocumentError>
    where
        F: FnOnce(&str) -> String,
    {
        let target = self.task_line_index(index)?;
        let mut transform = Some(transform);
        Ok(self.rewrite_lines(|line_idx, line| {
            if line_idx == target {
                let f = transform.take().unwrap();
                Some(f(line))
            } else {
                Some(line.to_string())
            }
        }))
    }

    fn rewrite_lines<F>(self, mut transform: F) -> Self
    where
        F: FnMut(usize, &str) -> Option<String>,
    {
        let trailing_newline = self.raw.ends_with('\n');
        let mut out = String::with_capacity(self.raw.len());
        for (line_idx, line) in self.raw.lines().enumerate() {
            if let Some(next_line) = transform(line_idx, line) {
                out.push_str(&next_line);
                out.push('\n');
            }
        }
        if !trailing_newline {
            out.pop();
        }
        Self { raw: out }
    }

    fn task_line_index(&self, index: usize) -> Result<usize, FocusDocumentError> {
        task_line_indices(&self.raw)
            .get(index)
            .copied()
            .ok_or(FocusDocumentError::TaskIndexOutOfRange { index })
    }
}

fn task_line_indices(raw: &str) -> Vec<usize> {
    raw.lines()
        .enumerate()
        .filter_map(|(line_idx, line)| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") {
                Some(line_idx)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document(raw: &str) -> FocusDocument {
        FocusDocument::from_raw(raw.to_string())
    }

    fn focus_md(tasks: &[&str]) -> String {
        let mut s = "---\nid: a\ntitle: A\ndescription:\ncreated_at: 2026-04-30T12:00:00Z\n---\n"
            .to_string();
        for task in tasks {
            s.push_str(task);
            s.push('\n');
        }
        s
    }

    #[test]
    fn rename_focus_updates_title_in_frontmatter_only() {
        let raw = format!("{}notes title: keep\n", focus_md(&["- [ ] one"]));

        let next = document(&raw).rename_focus("New Title").into_raw();

        assert!(next.contains("title: New Title\n"));
        assert!(next.contains("notes title: keep\n"));
    }

    #[test]
    fn rename_focus_preserves_missing_title_behavior() {
        let raw = "---\nid: a\ndescription:\n---\n- [ ] one\n";

        let next = document(raw).rename_focus("New Title").into_raw();

        assert_eq!(next, raw);
    }

    #[test]
    fn append_task_appends_checkbox_bullet() {
        let raw = focus_md(&["- [ ] existing"]);

        let next = document(&raw).append_task("new task").into_raw();

        assert!(next.contains("- [ ] existing\n"));
        assert!(next.ends_with("- [ ] new task\n"));
    }

    #[test]
    fn append_task_preserves_no_trailing_newline_behavior() {
        let raw = "---\nid: a\n---";

        let next = document(raw).append_task("new task").into_raw();

        assert_eq!(next, "---\nid: a\n---\n- [ ] new task\n");
    }

    #[test]
    fn delete_task_removes_indexed_checkbox() {
        let raw = focus_md(&["- [ ] one", "- [x] two", "- [ ] three"]);

        let next = document(&raw).delete_task(1).unwrap().into_raw();

        assert!(next.contains("- [ ] one\n"));
        assert!(!next.contains("- [x] two\n"));
        assert!(next.contains("- [ ] three\n"));
    }

    #[test]
    fn delete_task_preserves_missing_trailing_newline() {
        let raw = "- [ ] one\n- [ ] two";

        let next = document(raw).delete_task(0).unwrap().into_raw();

        assert_eq!(next, "- [ ] two");
    }

    #[test]
    fn delete_task_returns_index_error() {
        let err = document("- [ ] one\n").delete_task(3).unwrap_err();

        assert_eq!(err, FocusDocumentError::TaskIndexOutOfRange { index: 3 });
    }

    #[test]
    fn update_task_preserves_marker_and_indentation() {
        let raw = "  - [x] old\n- [ ] keep\n";

        let next = document(raw).update_task(0, " new ").unwrap().into_raw();

        assert_eq!(next, "  - [x] new\n- [ ] keep\n");
    }

    #[test]
    fn toggle_task_updates_marker_and_preserves_text() {
        let raw = "  - [ ] keep spacing\n";

        let next = document(raw).toggle_task(0, true).unwrap().into_raw();

        assert_eq!(next, "  - [x] keep spacing\n");
    }
}
