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

    // Run detail sub-views
    CycleDetailSubView,

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
    RunDialogNextField,
    RunDialogToggleGpu,
    StopContainer,

    // Search
    EnterSearchMode,
    ExitSearchMode,
    SearchInput(char),
    SearchBackspace,
    SearchClear,

    // Shared input
    InputChar(char),
    InputBackspace,
    InputConfirm,
    InputCancel,

    // Chart
    CycleMetric,

    // Export
    ExportMarkdown,
    ExportCsv,
    ExportLatex,

    // Menu
    MenuSelect(usize),
    SplashDismiss,

    // Misc
    Refresh,
    ToggleHelp,
    None,
}

