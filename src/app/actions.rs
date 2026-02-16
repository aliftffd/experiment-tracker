/// All possible user-triggered actions in the app
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    // Navigation
    Quit,
    NextTab,
    PrevTab,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Select,
    Back,

    // Views
    GoToDashboard,
    GoToRunDetail(i64),
    GoToCompare,
    GoToGpuMonitor,

    // Run operations
    DeleteRun,
    ConfirmDelete,
    CancelDelete,
    ToggleRunStatus,

    // Tag operations
    OpenTagList,
    OpenTagInput,
    ConfirmTagInput,
    RemoveSelectedTag,

    // Notes
    OpenNotesEditor,
    ConfirmNotesInput,

    // Compare
    ToggleCompareSelection,
    CycleCompareMetric,

    // Docker
    OpenRunDialog,
    ConfirmRunDialog,

    // Search
    EnterSearchMode,
    ExitSearchMode,
    SearchInput(char),
    SearchBackspace,
    SearchClear,

    // Shared input (used by tag input, notes, run dialog)
    InputChar(char),
    InputBackspace,
    InputConfirm,
    InputCancel,

    // Chart
    CycleMetric,

    // Export (Day 4)
    ExportMarkdown,
    ExportCsv,
    ExportLatex,

    // Container
    StopContainer,

    // Misc
    Refresh,
    ToggleHelp,
    None,
}

