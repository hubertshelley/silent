use crate::Route;

pub(crate) enum Matched {
    Matched(Route),
    Unmatched,
}

pub(crate) trait Match {
    fn handler_match(&self, path: &str) -> Matched;
}
