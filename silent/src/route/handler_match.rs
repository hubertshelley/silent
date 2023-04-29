use crate::{Request, Route};

pub(crate) enum Matched {
    Matched(Route),
    Unmatched,
}

pub(crate) trait Match {
    fn handler_match(&self, req: &Request, path: &str) -> Matched;
}
