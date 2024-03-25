use llm::models::Llama;
use llm::Model;
use silent::prelude::{info, warn};
use silent::{MiddleWareHandler, MiddlewareResult, Request, Response, Result};
use std::fmt::Debug;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone)]
pub(crate) struct Llm {
    model_path: String,
}

impl Debug for Llm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Llm")
    }
}

impl Llm {
    fn new() -> Self {
        Self {
            model_path:
                "/Users/hubertshelley/Documents/machine_learn/llama-13B-ggml/ggml-model-q4_0.bin"
                    .to_string(),
        }
    }

    pub(crate) fn chat(&self, ask_str: &str, sender: &UnboundedSender<String>) -> Result<String> {
        info!("ask_str: {}, sender: {:#?}", ask_str, sender);
        let llama = llm::load::<Llama>(
            // path to GGML file
            std::path::Path::new(self.model_path.as_str()),
            // llm::ModelParameters
            Default::default(),
            // load progress callback
            llm::load_progress_callback_stdout,
        )
        .unwrap_or_else(|err| panic!("Failed to load model: {err}"));
        warn!("llama");
        let mut session = llama.start_session(Default::default());
        warn!("session");
        let res = session.infer::<std::convert::Infallible>(
            // model to use for text generation
            &llama,
            // randomness provider
            &mut rand::thread_rng(),
            // the prompt to use for text generation, as well as other
            // inference parameters
            &llm::InferenceRequest {
                prompt: ask_str,
                ..Default::default()
            },
            // llm::OutputRequest
            &mut Default::default(),
            // output callback
            |t| {
                info!("t: {}", t.to_string());
                sender.send(t.to_string()).unwrap();
                Ok(())
            },
        );

        match res {
            Ok(result) => Ok(format!("{}", result)),
            Err(err) => Ok(format!("{}", err)),
        }
    }
}

pub(crate) struct LLMMiddleware {
    llm: Llm,
}

impl LLMMiddleware {
    pub(crate) fn new() -> Self {
        Self { llm: Llm::new() }
    }
}

#[async_trait::async_trait]
impl MiddleWareHandler for LLMMiddleware {
    async fn pre_request(
        &self,
        req: &mut Request,
        _res: &mut Response,
    ) -> Result<MiddlewareResult> {
        let llm = self.llm.clone();
        req.extensions_mut().insert(llm);
        Ok(MiddlewareResult::Continue)
    }
}
