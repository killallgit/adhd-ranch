pub fn slugify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_dash = true;
    for ch in input.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            out.push(lower);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        "focus".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowercases_and_hyphenates() {
        assert_eq!(slugify("Customer X bug"), "customer-x-bug");
    }

    #[test]
    fn collapses_consecutive_separators() {
        assert_eq!(slugify("Hello,  World!!"), "hello-world");
    }

    #[test]
    fn falls_back_to_focus_for_empty_input() {
        assert_eq!(slugify("   "), "focus");
        assert_eq!(slugify("!!!"), "focus");
    }

    #[test]
    fn drops_leading_and_trailing_separators() {
        assert_eq!(slugify("--abc--"), "abc");
    }

    #[test]
    fn preserves_digits() {
        assert_eq!(slugify("Sprint 42"), "sprint-42");
    }
}
