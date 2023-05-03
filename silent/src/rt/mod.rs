#[derive(Default, Debug, Clone)]
/// An Executor that uses the tokio runtime.
pub struct RtExecutor;

impl<F> hyper::rt::Executor<F> for RtExecutor
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}
