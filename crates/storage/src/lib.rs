use adhd_ranch_domain::Focus;

pub trait FocusStore: Send + Sync {
    fn list(&self) -> Vec<Focus>;
}
