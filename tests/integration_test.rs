use sigmaos_update::{Config, UpdateChannel, UpdateComponent, UpdatePipeline, UpdateSpec, UpdateState, Component};

#[test]
fn test_config_default_aaa() {
    let c = Config::default();
    assert!(c.enabled);
    assert_eq!(c.channel, "stable");
}

#[test]
fn test_pipeline_initial_state_aaa() {
    let pipeline = UpdatePipeline::new(UpdateChannel::Stable);
    assert_eq!(pipeline.state(), &UpdateState::Idle);
    assert_eq!(pipeline.channel(), &UpdateChannel::Stable);
}

#[test]
fn test_pipeline_check_returns_none_aaa() {
    let mut pipeline = UpdatePipeline::new(UpdateChannel::Stable);
    let result = pipeline.check();
    assert!(result.is_none());
    assert_eq!(pipeline.state(), &UpdateState::Idle);
}

#[test]
fn test_pipeline_apply_records_history_aaa() {
    let mut pipeline = UpdatePipeline::new(UpdateChannel::Beta);
    let spec = UpdateSpec::new("0.2.0", UpdateChannel::Beta);
    pipeline.apply(&spec).unwrap();
    assert_eq!(pipeline.history().len(), 1);
    assert_eq!(pipeline.history()[0], "0.2.0");
}

#[test]
fn test_component_lifecycle_aaa() {
    let mut c = UpdateComponent::new(Config::default());
    assert!(c.initialize().is_ok());
    assert!(c.health_check().is_ok());
    assert!(c.shutdown().is_ok());
}

#[test]
fn test_component_name_aaa() {
    let c = UpdateComponent::new(Config::default());
    assert_eq!(c.name(), "sigmaos-update");
}
