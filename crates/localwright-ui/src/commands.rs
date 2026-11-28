//! Shared command descriptors used by the built-in menu and toolbar.

/// Stable identifier for actions exposed by the generic workbench.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkbenchCommand {
    /// Open the native input-file picker.
    OpenInput,
    /// Open the native output-file picker.
    ChooseOutput,
    /// Open a new independent project tab.
    NewProject,
    /// Close the active project tab.
    CloseProject,
    /// Save operation context and shared layout as a versioned session file.
    SaveSession,
    /// Restore a saved session by appending its project tabs.
    RestoreSession,
    /// Run the selected operation.
    Run,
    /// Request cancellation of the active job.
    Cancel,
    /// Show operation output.
    ShowResults,
    /// Show the editable draft.
    ShowDraft,
    /// Display the registered keyboard shortcuts.
    ShowShortcuts,
    /// Toggle the common action toolbar.
    ToggleToolbar,
}

/// Menu containing a workbench command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandMenu {
    /// File operations.
    File,
    /// Editing and draft actions.
    Edit,
    /// Workspace view actions.
    View,
    /// General tools.
    Tools,
    /// Local help.
    Help,
}

/// Static description for a command shared by the menu and toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandDescriptor {
    /// Stable action identifier.
    pub command: WorkbenchCommand,
    /// User-facing label.
    pub label: &'static str,
    /// Menu where the action appears.
    pub menu: CommandMenu,
    /// Whether the action also appears in the toolbar.
    pub toolbar: bool,
}

/// Built-in registry for the common commands implemented by the workbench.
/// Applications can use the descriptors to keep common action labels consistent.
pub const COMMAND_REGISTRY: &[CommandDescriptor] = &[
    CommandDescriptor {
        command: WorkbenchCommand::OpenInput,
        label: "Open input…",
        menu: CommandMenu::File,
        toolbar: true,
    },
    CommandDescriptor {
        command: WorkbenchCommand::ChooseOutput,
        label: "Save as…",
        menu: CommandMenu::File,
        toolbar: true,
    },
    CommandDescriptor {
        command: WorkbenchCommand::NewProject,
        label: "New project tab",
        menu: CommandMenu::File,
        toolbar: false,
    },
    CommandDescriptor {
        command: WorkbenchCommand::CloseProject,
        label: "Close project tab",
        menu: CommandMenu::File,
        toolbar: false,
    },
    CommandDescriptor {
        command: WorkbenchCommand::SaveSession,
        label: "Save workspace session…",
        menu: CommandMenu::File,
        toolbar: false,
    },
    CommandDescriptor {
        command: WorkbenchCommand::RestoreSession,
        label: "Restore workspace session…",
        menu: CommandMenu::File,
        toolbar: false,
    },
    CommandDescriptor {
        command: WorkbenchCommand::Run,
        label: "Run operation",
        menu: CommandMenu::Tools,
        toolbar: true,
    },
    CommandDescriptor {
        command: WorkbenchCommand::Cancel,
        label: "Cancel operation",
        menu: CommandMenu::Tools,
        toolbar: true,
    },
    CommandDescriptor {
        command: WorkbenchCommand::ShowResults,
        label: "Results",
        menu: CommandMenu::View,
        toolbar: false,
    },
    CommandDescriptor {
        command: WorkbenchCommand::ShowDraft,
        label: "Draft / transcript",
        menu: CommandMenu::Edit,
        toolbar: false,
    },
    CommandDescriptor {
        command: WorkbenchCommand::ShowShortcuts,
        label: "Keyboard shortcuts",
        menu: CommandMenu::Help,
        toolbar: false,
    },
    CommandDescriptor {
        command: WorkbenchCommand::ToggleToolbar,
        label: "Toggle toolbar",
        menu: CommandMenu::View,
        toolbar: false,
    },
];

impl CommandMenu {
    /// Stable menu title.
    pub fn label(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::View => "View",
            Self::Tools => "Tools",
            Self::Help => "Help",
        }
    }

    /// Ordered built-in menu groups.
    pub const fn all() -> &'static [Self] {
        &[Self::File, Self::Edit, Self::View, Self::Tools, Self::Help]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn command_registry_has_unique_actions_and_complete_menu_groups() {
        let mut commands = HashSet::new();
        for descriptor in COMMAND_REGISTRY {
            assert!(commands.insert(descriptor.command));
            assert!(!descriptor.label.is_empty());
            assert!(CommandMenu::all().contains(&descriptor.menu));
        }
        assert_eq!(commands.len(), 12);
        assert!(COMMAND_REGISTRY.iter().any(|item| item.toolbar));
    }
}
