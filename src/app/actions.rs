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

    // Run Operations
    DeleteRun,
    ToggleRunStatus,

    // Tag Operations
    AddTag(String),
    RemoveTag(String),

    // Search
    EnterSearchMode,
    ExitSearchMode,
    SearchInput(char),
    SearchBackspace,
    SearchClear,

    // Chart
    CycleMetric,

    // Export (Day 4)
    ExportMarkdown,
    ExportCsv,
    ExportLatex,

    // Misc
    Refresh,
    ToggleHelp,
    None,
}
