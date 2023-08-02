use crate::{MiddleWareHandler, Request, Response, Result, SilentError, StatusCode};
use async_trait::async_trait;
use serde::Serialize;
use std::sync::Arc;
use tera::{Context, Tera};

#[derive(Debug, Clone)]
pub struct TemplateResponse<T>
where
    T: Serialize,
{
    template: String,
    data: T,
}

impl<T: Serialize> From<(String, T)> for TemplateResponse<T> {
    fn from((template, data): (String, T)) -> Self {
        TemplateResponse { template, data }
    }
}

impl<T: Serialize + Send + Sync + 'static> From<TemplateResponse<T>> for Response {
    fn from(value: TemplateResponse<T>) -> Self {
        let mut res = Response::empty();
        res.extensions.insert(value);
        res
    }
}

pub struct TemplateMiddleware {
    pub template: Arc<Tera>,
}

impl TemplateMiddleware {
    pub fn new(template_path: &str) -> Self {
        println!("template middleware new");
        let template = Arc::new(Tera::new(template_path).expect("Failed to load templates"));
        TemplateMiddleware { template }
    }
}

#[async_trait]
impl MiddleWareHandler for TemplateMiddleware {
    async fn pre_request(&self, _req: &mut Request, _res: &mut Response) -> Result<()> {
        println!("template middleware pre request");
        Ok(())
    }
    async fn after_response(&self, res: &mut Response) -> Result<()> {
        println!("template middleware after response");
        let template = res.extensions.get::<TemplateResponse<()>>().unwrap();
        res.set_body(
            self.template
                .render(
                    &template.template,
                    &{
                        #[allow(clippy::no_effect)]
                        template.data;
                        Context::from_serialize(())
                    }
                    .map_err(|e| {
                        SilentError::business_error(
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Failed to render template: {}", e),
                        )
                    })?,
                )
                .map_err(|e| {
                    SilentError::business_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to render template: {}", e),
                    )
                })?
                .into(),
        );
        Ok(())
    }
}
