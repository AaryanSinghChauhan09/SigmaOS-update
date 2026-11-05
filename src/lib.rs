#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UpdateStatus { #[default] Idle, Staging, AppliedPendingReboot, RolledBack }

#[derive(Debug, Default)]
pub struct UpdateAgent {
    status: UpdateStatus,
}

impl UpdateAgent {
    pub fn new() -> Self { Self::default() }
    pub fn apply_update(&mut self) { self.status = UpdateStatus::AppliedPendingReboot; }
    pub fn rollback_on_failure(&mut self) { self.status = UpdateStatus::RolledBack; }
    pub fn get_status(&self) -> UpdateStatus { self.status }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_update_agent_status_flow() {
        let mut agent = UpdateAgent::new();
        agent.apply_update();
        assert_eq!(agent.get_status(), UpdateStatus::AppliedPendingReboot);
        agent.rollback_on_failure();
        assert_eq!(agent.get_status(), UpdateStatus::RolledBack);
    }
}
