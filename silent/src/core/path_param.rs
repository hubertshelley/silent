use crate::SilentError;
use uuid::Uuid;

/// 路由参数
/// 支持类型：
///     String(String)
///     Int(i32),
///     Uuid(Uuid),
///     Path(String),
/// 支持数据转换
#[derive(Debug, Clone, PartialEq)]
pub enum PathParam {
    /// 字符串类型参数
    String(String),
    /// 整型参数
    Int(i32),
    /// Uuid类型参数
    Uuid(Uuid),
    /// 路径参数
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
        PathParam::Uuid(u)
    }
}

impl<'a> TryFrom<&'a PathParam> for i32 {
    type Error = SilentError;

    fn try_from(value: &'a PathParam) -> Result<Self, Self::Error> {
        match value {
            PathParam::Int(value) => Ok(*value),
            _ => Err(SilentError::ParamsNotFound),
        }
    }
}

impl<'a> TryFrom<&'a PathParam> for String {
    type Error = SilentError;

    fn try_from(value: &'a PathParam) -> Result<Self, Self::Error> {
        match value {
            PathParam::String(value) => Ok(value.clone()),
            PathParam::Path(value) => Ok(value.clone()),
            _ => Err(SilentError::ParamsNotFound),
        }
    }
}

impl<'a> TryFrom<&'a PathParam> for Uuid {
    type Error = SilentError;

    fn try_from(value: &'a PathParam) -> Result<Self, Self::Error> {
        match value {
            PathParam::Uuid(value) => Ok(*value),
            _ => Err(SilentError::ParamsNotFound),
        }
    }
}
