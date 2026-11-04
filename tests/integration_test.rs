use anyhow::Result;
use sigmaos_package_component::{
    Config, Service, Update, PackageRepository, Package, Version,
    PackageInstaller, SovereignSystemUpdateManager, SystemUpdatePolicy,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sovereign_update_manager_aaa() {
        // Arrange
        let policy = SystemUpdatePolicy::default();
        let mut mgr = SovereignSystemUpdateManager::new("1.0.0", policy);

        // Act & Assert - No updates applied yet so rollback fails fast
        let rollback_res = mgr.rollback_last_update();
        assert!(rollback_res.is_err());
    }

    #[test]
    fn test_update_controller_aaa() -> Result<()> {
        // Arrange
        let config = Config::default();
        let mut update = Update::new(config);

        // Act & Assert
        update.initialize()?;
        update.run()?;
        Ok(())
    }

    #[test]
    fn test_package_repository_and_installer_aaa() -> Result<()> {
        // Arrange
        let mut repo = PackageRepository::new();
        let pkg = Package::new(
            "linux-firmware".to_string(),
            Version::new(2026, 11, 4),
            "Updated Linux kernel firmware".to_string(),
        );
        repo.add_package(pkg);

        // Act
        let retrieved = repo.get_package("linux-firmware");

        // Assert
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().version, Version::new(2026, 11, 4));

        // Arrange - Installer
        let mut installer = PackageInstaller::default();
        let mut tx = installer.create_transaction("linux-firmware".to_string());

        // Act
        installer.install(&mut tx).map_err(|e| anyhow::anyhow!(e))?;

        // Assert
        assert!(tx.is_complete());
        Ok(())
    }

    #[test]
    fn test_service_lifecycle_aaa() -> Result<()> {
        // Arrange
        let config = Config::default();
        let mut service = Service::new(config);
        assert!(!service.is_running());

        // Act & Assert - Start
        service.start()?;
        assert!(service.is_running());

        // Act & Assert - Redundant start fails cleanly
        assert!(service.start().is_err());

        // Act & Assert - Stop
        service.stop()?;
        assert!(!service.is_running());

        // Act & Assert - Redundant stop fails cleanly
        assert!(service.stop().is_err());

        Ok(())
    }
}
