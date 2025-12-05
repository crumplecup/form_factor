# Planning Document Index

This document provides a comprehensive index of all planning and strategy documents in the Form Factor workspace, organized by category with their last commit information.

## About This Index

This index tracks all planning documents in the workspace. When documents are completed or superseded, they are **deleted from the workspace** but remain accessible in git history. The index preserves entries for deleted documents with their last commit hash, allowing easy retrieval via `git show <commit>:<path>`.

**To view a deleted document**: `git show <commit-hash>:<document-path>`

## Active Planning Documents

### Template/Instance System
- **TEMPLATE_UI_PLAN.md** - `current` (2025-12-05)
  - Visual template editor implementation plan
  - Template manager panel, drag-and-drop field placement, property editor
  - Supersedes Priority 5 from TEMPLATE_SYSTEM_PLAN

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

## Document Categories Summary

- **Active Planning**: 1 document
- **Development Guidelines**: 1 document
- **Build & Workspace**: 2 documents
- **Feature Guides**: 4 documents
- **Archived**: 1 document

**Total**: 9 markdown documents tracked (8 active, 1 archived)

---

*Last Updated: 2025-12-05*
*Generated automatically - see git log for detailed history*
