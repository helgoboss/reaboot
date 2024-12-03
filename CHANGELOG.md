# Changelog

All notable changes to the installer (not the website) will be documented in this file, starting from 0.8.0.

## [0.8.0] - 2024-12-04 (coming soon)

## [Unreleased]

### Improved

- #10 Improved location of backup files (now in dedicated directory
  `ReaBoot/backups`)

### Fixed

- #1 Fixed installation error "no column found for name: flags" if old ReaPack version/registry installed
- #2 Fixed installation error "Simulated installation failed" if package files already exist in destination directory,
  but are not yet managed by ReaPack (this frequently happened when installing SWS, for example)
- #4 macOS: Fixed missing "New directory" button in portable REAPER
  installation picker (a side effect of updating to Tauri 2)
- #8 Fixed overflowing feature text if not enough horizontal space

### Changed

- Internal change: Migrated from Tauri 1 to Tauri 2