enum PathParam {
    String(String, String),
    Int(String, i32),
    UUid(String, String),
}

impl From<(String, String)> for PathParam {
    fn from((match_path, path): (String, String)) -> Self {
        PathParam::String(name, path)
    }
}