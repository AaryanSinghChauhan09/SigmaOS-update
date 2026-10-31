# SigmaOS Update

This crate exports `UpdateManager` from the `update` module and a separate
MintUpdate-inspired API. The MintUpdate manager currently simulates finding
updates and keeps task, history, and update state in memory. A simulated update
record is not a package download or installation.

The repository also contains update-strategy and atomic-update modules. This
README does not establish that an OS update, signature check, slot switch, or
rollback has been performed safely; those paths require implementation-specific
review and recovery evidence before use on a host.

The crate depends on sibling `SigmaOS-kernel` and `SigmaOS-package` checkouts
through local paths.

## License

GPL-3.0-or-later. See [`LICENSE`](LICENSE) and [`Cargo.toml`](Cargo.toml).
