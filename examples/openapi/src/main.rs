use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use silent::{Handler, Request, Response, Result};

fn main() {
    println!("Hello, world!");
}

struct CustomHandler;
#[async_trait::async_trait]
impl Handler for CustomHandler {
    async fn call(&self, _req: Request) -> Result<Response> {
        Ok(Response::empty())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
/// 查 数据返回
pub struct ListData<T> {
    pub list: Vec<T>,
    pub total: u64,
    pub total_pages: u64,
    pub page_num: u64,
}
/// 分页参数
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct PageParams {
    /// 页码
    pub page_num: Option<u64>,
    /// 每页数量
    pub page_size: Option<u64>,
}

/// 数据统一返回格式
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct Res<T> {
    pub code: Option<i32>,
    pub data: Option<T>,
    pub msg: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_data() {
        let schema = schema_for!(PageParams);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }

    #[test]
    fn test_list_data1() {
        let schema = schema_for!(Res<ListData<PageParams>>);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }
}
