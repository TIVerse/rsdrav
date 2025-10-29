# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-10-29

### 🎉 First Stable Release

This is the first production-ready release of rsdrav with all core features complete and all TODOs resolved.

### Added

- **Event System Enhancements**
  - Full paste event support with string data
  - SUPER, META, and HYPER key modifier support
  - Complete keyboard modifier handling

- **Container Layout System**
  - Proper ViewNode::Container with Row/Column/Stack integration
  - ContainerDirection enum (Vertical, Horizontal, Stacked)
  - Automatic layout calculation based on direction
  - Equal space distribution among children

- **Button Widget Improvements**
  - Complete mouse hit-testing using position tracking
  - Hover state detection based on mouse position
  - Click detection with proper bounds checking
  - Interior mutability for position storage using Cell

- **Rendering Optimizations**
  - Exact x-span detection in diff algorithm
  - Adjacent dirty region merging to reduce draw calls
  - Line-hash-based change detection
  - Optimized terminal writes

- **Command System Enhancements**
  - Redesigned undo system with UndoableAction trait
  - Proper action cloning and state management
  - ValueChangeAction example implementation
  - EventBus for pub-sub messaging between commands
  - PluginManager integration in CommandContext
  - Full filesystem completion for file paths

- **Table Widget Features**
  - Functional column sorting with sort keys
  - Custom sort_by() method for columns
  - Ascending/Descending/None sort states
  - Automatic data reordering on sort
  - Sort indicator toggling

### Changed

- Event enum now uses Clone instead of Copy (due to String in Paste)
- EventContext now includes area field for component hit-testing
- Container rendering uses proper layout algorithms instead of sequential placement
- compute_diff_precise now delegates to compute_diff (optimized by default)
- CommandContext extended with event_bus and plugin_manager fields
- Layout containers documented to use full cross-axis (alignment is documented behavior)

### Fixed

- All TODO comments in source code resolved
- Button mouse interactions now work correctly
- Container children render with proper layout
- Table sorting actually sorts data (not just UI indicator)
- File completion implements real filesystem access
- Undo system properly clones and manages actions

### Removed

- All TODO comments from production code
- Placeholder implementations replaced with full functionality

## [0.2.0] - Previous Release

### Added
- 12 production-ready widgets
- Plugin system architecture
- Animation system with tweens & easing
- Integration test suite
- Performance benchmarks

## [0.1.5] - Previous Release

### Added
- Component system with lifecycle
- 9 widgets
- Focus management
- 6 examples

## [0.1.0] - Initial Release

### Added
- Foundation: rendering, state, layout, commands
- Basic widget set
- Reactive state management
- Backend abstraction

[1.0.0]: https://github.com/vedanthq/rsdrav/releases/tag/v1.0.0
[0.2.0]: https://github.com/vedanthq/rsdrav/releases/tag/v0.2.0
[0.1.5]: https://github.com/vedanthq/rsdrav/releases/tag/v0.1.5
[0.1.0]: https://github.com/vedanthq/rsdrav/releases/tag/v0.1.0
