# Planning Document Index

This document provides a comprehensive index of all planning and strategy documents in the Form Factor workspace, organized by category with their last commit information.

## About This Index

This index tracks all planning documents in the workspace. When documents are completed or superseded, they are **deleted from the workspace** but remain accessible in git history. The index preserves entries for deleted documents with their last commit hash, allowing easy retrieval via `git show <commit>:<path>`.

**To view a deleted document**: `git show <commit-hash>:<document-path>`

## Active Planning Documents

### Refactoring
- **MAIN_REFACTOR_PLAN.md** - `current` (2025-12-07)
  - Refactoring main.rs from "dumping ground" to maintainable structure
  - Extract event handlers, detection tasks, file dialogs, plugin setup
  - Phase-by-phase plan with testing strategy

### Template/Instance System
- **TEMPLATE_UI_PLAN.md** - `current` (2025-12-05)
  - Visual template editor implementation plan
  - Template manager panel, drag-and-drop field placement, property editor
  - Supersedes Priority 5 from TEMPLATE_SYSTEM_PLAN



### UI/UX Development
- **UI_ROADMAP.md** - `current` (2025-12-05)
  - UI assessment and roadmap from current state to awesome UI
  - 4 phases: Template UI Integration, Instance Data Entry, Workflow Enhancements, Advanced Features
  - Addresses plugin architecture transition gaps and missing user workflows
- **UI_ROADMAP_OVERLAY.md** - `current` (2025-12-20)
  - **UNIFIED PLAN:** Simplified overlay-based UI architecture
  - Canvas + overlays approach: Template browser, data entry panel, instance browser
  - **Includes critical "Add to Template" workflow** (detection/shape → template field conversion)
  - Complete user stories: Load form → detect → add to template → save
  - Estimated 10-15 days (4 phases with gap analysis integrated)

## Development Guidelines

- **CLAUDE.md** - `380adc7` (2025-12-04)
  - Project development guidelines and conventions
  - Testing patterns, error handling, module organization
  - Workflow and commit best practices

## Build & Workspace

- **BUILD.md** - `b282109` (2025-11-02)
  - Build system documentation and compilation instructions
- **WORKSPACE.md** - `2c22421` (2025-11-04)
  - Workspace organization and crate structure

## Feature Guides

### OpenCV Integration
- **LOGO_DETECTION.md** - `7f08db2` (2025-11-08)
  - Logo detection using OpenCV template matching
- **TEXT_DETECTION.md** - `daf81ff` (2025-11-02)
  - Text region detection using EAST model
- **OCR.md** - `2438602` (2025-11-02)
  - Optical character recognition with Tesseract

### Extensibility
- **PLUGINS.md** - `614510c` (2025-11-09)
  - Plugin system architecture and development guide

## Archived Planning Documents

*These documents have been deleted from the workspace but remain in git history. View with `git show <commit>:<path>`*

### Template/Instance System (Completed)
- **TEMPLATE_SYSTEM_PLAN.md** - `1cb1717` (2025-12-05)
  - Strategic planning document for template/instance system implementation
  - Priorities 1-4 completed: Template builder, field extraction, canvas integration, legacy migration
  - Priority 5 (Template UI) expanded into separate planning document

### Testing Strategy (Completed)
- **INTEGRATION_TESTING_PLAN.md** - `da60861` (2025-12-06)
  - Comprehensive integration testing implementation plan
  - 4 phases: Foundation, User Workflows, Domain Features, Optional Features
  - form_factor_health crate created for test helpers and integration tests

## Document Categories Summary

- **Active Planning**: 3 documents
- **Development Guidelines**: 1 document
- **Build & Workspace**: 2 documents
- **Feature Guides**: 4 documents
- **Archived**: 2 documents

**Total**: 12 markdown documents tracked (10 active, 2 archived)

---

*Last Updated: 2025-12-07*
*Generated automatically - see git log for detailed history*
- [ERROR_REFACTOR_STRATEGY.md](ERROR_REFACTOR_STRATEGY.md) - Error handling architecture cleanup
