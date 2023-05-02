use uuid::Uuid;

#[derive(Debug)]
pub enum PathParam {
    String(String),
    Int(i32),
    UUid(Uuid),
    Path(String),
}

impl From<String> for PathParam {
    fn from(s: String) -> Self {
        PathParam::String(s)
    }
}

impl From<i32> for PathParam {
    fn from(i: i32) -> Self {
        PathParam::Int(i)
    }
}

impl From<Uuid> for PathParam {
    fn from(u: Uuid) -> Self {
        PathParam::UUid(u)
    }
}
