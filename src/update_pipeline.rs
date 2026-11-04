#[derive(Debug, Clone, PartialEq)]
pub enum UpdateChannel { Stable, Beta, Nightly }
#[derive(Debug, Clone, PartialEq)]
pub enum UpdateState { Idle, Checking, Downloading, Applying, Failed(String) }
#[derive(Debug, Clone)]
pub struct UpdateSpec { pub version: String, pub channel: UpdateChannel, pub size_bytes: u64 }
impl UpdateSpec {
    pub fn new(version: &str, channel: UpdateChannel) -> Self {
        Self { version: version.to_string(), channel, size_bytes: 0 }
    }
}
pub struct UpdatePipeline { state: UpdateState, channel: UpdateChannel, history: Vec<String> }
impl UpdatePipeline {
    pub fn new(channel: UpdateChannel) -> Self { Self { state: UpdateState::Idle, channel, history: Vec::new() } }
    pub fn check(&mut self) -> Option<UpdateSpec> {
        self.state = UpdateState::Checking;
        self.state = UpdateState::Idle;
        None
    }
    pub fn apply(&mut self, spec: &UpdateSpec) -> Result<(), String> {
        self.state = UpdateState::Applying;
        self.history.push(spec.version.clone());
        self.state = UpdateState::Idle;
        Ok(())
    }
    pub fn history(&self) -> &[String] { &self.history }
    pub fn channel(&self) -> &UpdateChannel { &self.channel }
    pub fn state(&self) -> &UpdateState { &self.state }
}
impl Default for UpdatePipeline { fn default() -> Self { Self::new(UpdateChannel::Stable) } }
