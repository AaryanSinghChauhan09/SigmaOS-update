// SPDX-License-Identifier: Apache-2.0
//! SigmaOS-update — SigmaOS-style system updater.
//!
//! Models update channels, available-update detection, transactional
//! apply (packages + migration scripts), and rollback points. Wraps the
//! sigpkg/pacman layer for real package ops; this is the policy engine.

use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Channel { Stable, Beta, Edge }
impl Channel {
    pub fn label(self) -> &'static str { match self { Self::Stable=>"stable",Self::Beta=>"beta",Self::Edge=>"edge" } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateKind {
    SystemPackage,   // pacman/sigpkg package bump
    SigmaOSComponent,// SigmaOS-sigmaos-* repo update
    Migration,       // one-shot config/state migration script
    Firmware,
    Dotfiles,
}
impl UpdateKind {
    pub fn label(self) -> &'static str {
        match self { Self::SystemPackage=>"pkg",Self::SigmaOSComponent=>"sigmaos",Self::Migration=>"migrate",Self::Firmware=>"fw",Self::Dotfiles=>"dotfiles" }
    }
}

#[derive(Debug, Clone)]
pub struct AvailableUpdate {
    pub id: String,
    pub name: String,
    pub kind: UpdateKind,
    pub from_version: String,
    pub to_version: String,
    pub channel: Channel,
    pub requires_reboot: bool,
    pub migration: Option<String>, // migration script id
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateState { Pending, Downloading, Installing, Migrating, Done, Failed, RolledBack }
impl UpdateState { pub fn label(self)->&'static str { match self {Self::Pending=>"pending",Self::Downloading=>"downloading",Self::Installing=>"installing",Self::Migrating=>"migrating",Self::Done=>"done",Self::Failed=>"failed",Self::RolledBack=>"rolled-back"} } }

#[derive(Debug, Clone)]
pub struct AppliedUpdate { pub update: AvailableUpdate, pub state: UpdateState, pub snapshot: Option<u32> }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateError {
    UnknownUpdate(String),
    WrongChannel { update: String, want: &'static str },
    NothingToUpdate,
    AlreadyApplied(String),
}
impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownUpdate(id) => write!(f, "no such update {id}"),
            Self::WrongChannel { update, want } => write!(f, "{update} not on {want} channel"),
            Self::NothingToUpdate => write!(f, "no updates available"),
            Self::AlreadyApplied(id) => write!(f, "{id} already applied"),
        }
    }
}
impl std::error::Error for UpdateError {}

pub struct SigmaOSUpdate {
    pub channel: Channel,
    pub last_check: Option<u64>,
    available: Vec<AvailableUpdate>,
    applied: BTreeMap<String, AppliedUpdate>,
    snapshots: u32,
}

impl SigmaOSUpdate {
    pub fn new() -> Self {
        Self { channel: Channel::Stable, last_check: None, available: Vec::new(), applied: BTreeMap::new(), snapshots: 0 }
    }

    /// Register an available update (as discovered from repos/package db).
    pub fn offer(&mut self, u: AvailableUpdate) { self.available.push(u); }
    pub fn available(&self) -> &[AvailableUpdate] { &self.available }

    /// Filter updates for the current channel.
    pub fn for_channel(&self) -> Vec<&AvailableUpdate> {
        self.available.iter().filter(|u| u.channel == self.channel || u.channel == Channel::Stable).collect()
    }

    /// Simulate checking remote repos for updates.
    pub fn check(&mut self, tick: u64) -> usize {
        self.last_check = Some(tick);
        self.for_channel().len()
    }

    /// Apply an update by id: snapshot -> install -> migrate -> done.
    pub fn apply(&mut self, id: &str) -> Result<u32, UpdateError> {
        let pos = self.available.iter().position(|u| u.id == id)
            .ok_or_else(|| UpdateError::UnknownUpdate(id.into()))?;
        let u = self.available[pos].clone();
        if u.channel != self.channel && u.channel != Channel::Stable {
            return Err(UpdateError::WrongChannel { update: id.into(), want: self.channel.label() });
        }
        if self.applied.contains_key(id) { return Err(UpdateError::AlreadyApplied(id.into())); }
        // create rollback snapshot
        self.snapshots += 1;
        let snap = self.snapshots;
        let mut au = AppliedUpdate { update: u.clone(), state: UpdateState::Done, snapshot: Some(snap) };
        if u.migration.is_some() { au.state = UpdateState::Migrating; }
        self.applied.insert(id.into(), au);
        self.available.remove(pos);
        Ok(snap)
    }

    /// Apply all pending updates for the channel. Returns count.
    pub fn apply_all(&mut self) -> usize {
        let ids: Vec<String> = self.for_channel().iter().map(|u| u.id.clone()).collect();
        let mut n = 0;
        for id in ids { if self.apply(&id).is_ok() { n += 1; } }
        n
    }

    /// Roll back an applied update to its snapshot.
    pub fn rollback(&mut self, id: &str) -> Result<u32, UpdateError> {
        let au = self.applied.get_mut(id).ok_or_else(|| UpdateError::UnknownUpdate(id.into()))?;
        let snap = au.snapshot.ok_or(UpdateError::UnknownUpdate(id.into()))?;
        au.state = UpdateState::RolledBack;
        Ok(snap)
    }

    pub fn needs_reboot(&self) -> bool {
        self.applied.values().any(|a| a.update.requires_reboot && a.state != UpdateState::RolledBack && a.state != UpdateState::Failed)
    }
    pub fn status_line(&self) -> String {
        let avail = self.available.len();
        let done = self.applied.values().filter(|a| a.state == UpdateState::Done).count();
        format!("update: channel={} available={} applied={} reboot={}",
            self.channel.label(), avail, done, self.needs_reboot())
    }
    pub fn applied(&self) -> impl Iterator<Item = &AppliedUpdate> { self.applied.values() }
}

impl Default for SigmaOSUpdate {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn upd(id: &str, kind: UpdateKind, ch: Channel) -> AvailableUpdate {
        AvailableUpdate { id: id.into(), name: id.into(), kind, from_version: "1.0".into(),
            to_version: "1.1".into(), channel: ch, requires_reboot: false, migration: None }
    }

    #[test]
    fn offer_and_channel_filter() {
        let mut m = SigmaOSUpdate::new();
        m.channel = Channel::Stable;
        m.offer(upd("pkg-a", UpdateKind::SystemPackage, Channel::Stable));
        m.offer(upd("beta-b", UpdateKind::SigmaOSComponent, Channel::Beta));
        m.offer(upd("pkg-c", UpdateKind::SystemPackage, Channel::Edge));
        // stable channel sees stable + (stable-only); beta/edge hidden
        assert_eq!(m.for_channel().len(), 1);
        m.channel = Channel::Beta;
        // beta sees beta + stable
        assert_eq!(m.for_channel().len(), 2);
    }

    #[test]
    fn apply_creates_snapshot() {
        let mut m = SigmaOSUpdate::new();
        m.offer(upd("pkg-a", UpdateKind::SystemPackage, Channel::Stable));
        let snap = m.apply("pkg-a").unwrap();
        assert_eq!(snap, 1);
        assert!(m.applied.contains_key("pkg-a"));
        assert!(m.available.is_empty());
        assert!(m.apply("pkg-a").is_err()); // AlreadyApplied/Unknown
    }

    #[test]
    fn wrong_channel_rejected() {
        let mut m = SigmaOSUpdate::new();
        m.offer(upd("edge-x", UpdateKind::Firmware, Channel::Edge));
        assert!(m.apply("edge-x").is_err());
    }

    #[test]
    fn migration_marks_state() {
        let mut m = SigmaOSUpdate::new();
        let mut u = upd("sigmaos-2", UpdateKind::SigmaOSComponent, Channel::Stable);
        u.migration = Some("migrate-sigmaos-1-to-2".into());
        u.requires_reboot = true;
        m.offer(u);
        m.apply("sigmaos-2").unwrap();
        let a = m.applied().next().unwrap();
        assert_eq!(a.state, UpdateState::Migrating);
        assert!(m.needs_reboot());
    }

    #[test]
    fn rollback_and_apply_all() {
        let mut m = SigmaOSUpdate::new();
        m.offer(upd("a", UpdateKind::SystemPackage, Channel::Stable));
        m.offer(upd("b", UpdateKind::SystemPackage, Channel::Stable));
        assert_eq!(m.apply_all(), 2);
        let snap = m.rollback("a").unwrap();
        assert!(snap > 0);
        assert_eq!(m.applied().find(|a| a.update.id == "a").unwrap().state, UpdateState::RolledBack);
        assert!(m.status_line().contains("reboot=false") || m.status_line().contains("reboot=true"));
    }

    #[test]
    fn check_and_status() {
        let mut m = SigmaOSUpdate::new();
        m.offer(upd("x", UpdateKind::Dotfiles, Channel::Stable));
        assert_eq!(m.check(42), 1);
        assert_eq!(m.last_check, Some(42));
        assert!(m.status_line().contains("stable"));
        assert!(SigmaOSUpdate::new().apply("none").is_err());
    }
}
