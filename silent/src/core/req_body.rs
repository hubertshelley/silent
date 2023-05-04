use hyper::body::Incoming;

#[derive(Debug)]
pub enum ReqBody {
    Empty,
    Incoming(Incoming),
}

impl From<Incoming> for ReqBody {
    fn from(incoming: Incoming) -> Self {
        Self::Incoming(incoming)
    }
}

impl From<()> for ReqBody {
    fn from(_: ()) -> Self {
        Self::Empty
    }
}
