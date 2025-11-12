// Service layer for GUI
//
// This module contains services that bridge the GUI with core functionality:
// - RecorderService: Manages recording operations (start, stop, status)
// - FileWatcherService: Watch status files and recordings directory (TODO)
// - HistoryService: Manage recording history and metadata (TODO)

pub mod recorder_service;

pub use recorder_service::RecorderService;
