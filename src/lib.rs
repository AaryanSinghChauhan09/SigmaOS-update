pub mod config;
pub mod error;
pub mod service;
pub mod update_pipeline;
pub use config::Config;
pub use error::{ComponentError, Result};
pub use service::{Service, ServiceStatus};
pub use update_pipeline::{UpdateChannel, UpdatePipeline, UpdateSpec, UpdateState};
pub trait Component {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self) -> error::Result<()>;
    fn shutdown(&mut self) -> error::Result<()>;
    fn health_check(&self) -> error::Result<()>;
}
pub struct UpdateComponent { service: service::Service }
impl UpdateComponent {
    pub fn new(config: Config) -> Self { Self { service: service::Service::new(config) } }
}
impl Component for UpdateComponent {
    fn name(&self) -> &str { "sigmaos-update" }
    fn version(&self) -> &str { env!("CARGO_PKG_VERSION") }
    fn initialize(&mut self) -> error::Result<()> { self.service.start() }
    fn shutdown(&mut self) -> error::Result<()> { self.service.stop() }
    fn health_check(&self) -> error::Result<()> {
        if self.service.is_running() { Ok(()) }
        else { Err(error::ComponentError::Service("update service not running".to_string())) }
    }
}
