use schemars::schema::RootSchema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use silent::prelude::{HandlerAppend, HandlerGetter, Route};
use silent::{Handler, HandlerWrapper, Method, Request, Response, Result};
use std::sync::Arc;
use utoipa::openapi::path::Operation;
use utoipa::openapi::{HttpMethod, OpenApi};
use utoipa::{IntoParams, ToSchema};

fn main() {
    let route = Route::new("hello")
        .handler(
            Method::GET,
            Arc::new(OpenAPIHandler::new(Arc::new(HandlerWrapper::new(
                |_req: Request| async move { Ok("hello world") },
            )))),
        )
        .post(|_req| async { Ok("post world") })
        .append(Route::new("world").get(|_req| async { Ok("world") }));
    println!("{}", route.generate_schema("").to_pretty_json().unwrap())
}

trait OpenAPI {
    fn generate_schema(&self, path: &str) -> OpenApi;
    fn get_method(method: &Method) -> Option<HttpMethod> {
        #[allow(clippy::match_ref_pats)]
        match method {
            &Method::GET => Some(HttpMethod::Get),
            &Method::POST => Some(HttpMethod::Post),
            &Method::PUT => Some(HttpMethod::Put),
            &Method::DELETE => Some(HttpMethod::Delete),
            &Method::PATCH => Some(HttpMethod::Patch),
            &_ => None,
        }
    }
}

impl OpenAPI for Route {
    fn generate_schema(&self, path: &str) -> OpenApi {
        let path = format!("{}/{}", path, self.path);
        let mut schema = OpenApi::default();
        for route in self.children.iter() {
            schema.merge(route.generate_schema(path.as_str()))
        }
        for (method, _handler) in self.handler.iter() {
            let response = utoipa::openapi::ResponsesBuilder::new()
                .response(
                    "200",
                    utoipa::openapi::ResponseBuilder::new()
                        .description("Todo item created successfully")
                        .content(
                            "application/json",
                            utoipa::openapi::content::ContentBuilder::new()
                                .schema(Some(
                                    utoipa::openapi::schema::RefBuilder::new()
                                        .ref_location_from_schema_name(
                                            <Data as utoipa::ToSchema>::name(),
                                        ),
                                ))
                                .into(),
                        )
                        .build(),
                )
                .build();

            let query = <PageParams as utoipa::IntoParams>::into_params(|| {
                Some(utoipa::openapi::path::ParameterIn::Query)
            });

            if let Some(method) = Self::get_method(method) {
                let operation = Operation::builder()
                    .deprecated(None)
                    .parameters(Some(query))
                    .responses(response)
                    .build();
                schema
                    .paths
                    .add_path_operation(path.as_str(), vec![method], operation)
            }
        }
        schema
    }
}

#[allow(dead_code)]
struct OpenAPIHandler {
    pub(crate) param_schema: Option<Arc<RootSchema>>,
    pub(crate) req_schema: Option<Arc<RootSchema>>,
    pub(crate) res_schema: Option<Arc<RootSchema>>,
    handler: Arc<dyn Handler>,
}

impl OpenAPIHandler {
    pub fn new(handler: Arc<dyn Handler>) -> Self {
        Self {
            param_schema: None,
            req_schema: None,
            res_schema: None,
            handler,
        }
    }
}

#[async_trait::async_trait]
impl Handler for OpenAPIHandler {
    async fn call(&self, req: Request) -> Result<Response> {
        self.handler.call(req).await
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, ToSchema)]
/// 查 数据返回
pub struct Data {
    pub name: String,
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
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema, IntoParams)]
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
    use schemars::schema_for;

    #[test]
    fn test_list_data() {
        let schema = schema_for!(PageParams);
        println!("{:?}", schema.meta_schema);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }

    #[test]
    fn test_list_data1() {
        let schema = schema_for!(Res<ListData<PageParams>>);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }

    #[test]
    fn test_schema() {}
}
