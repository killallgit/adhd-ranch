#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    EmptyTitle,
    EmptyTaskText,
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyTitle => f.write_str("title must not be empty"),
            Self::EmptyTaskText => f.write_str("task text must not be empty"),
        }
    }
}

impl std::error::Error for DomainError {}
