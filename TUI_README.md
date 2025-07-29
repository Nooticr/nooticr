# Orchy Modern TUI Interface

## Overview

Orchy now features a modern Terminal User Interface (TUI) built with [ratatui](https://ratatui.rs/), providing an intuitive and visually appealing way to manage your projects, tasks, agents, and issues.

## Features

### 🎨 **Modern Interface**
- Clean, tabbed interface with professional styling
- Real-time updates and responsive design
- Color-coded status indicators
- Keyboard-driven navigation

### 📊 **Dashboard View**
- Overview of current project information
- Quick statistics and metrics
- Recent activity summary
- Project selection guidance

### 🗂️ **Tabbed Navigation**
- **Dashboard**: Project overview and quick stats
- **Projects**: Browse and select projects
- **Tasks**: View tasks in the current project
- **Agents**: View agents in the current project  
- **Issues**: View issues in the current project

## Usage

### Starting the TUI

```bash
# Start the TUI interface (default behavior)
orchy

# Or explicitly start TUI
orchy tui

# Or use the alias
orchy ui
```

### Navigation

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch between tabs |
| `↑` / `↓` or `j` / `k` | Navigate lists |
| `Enter` | Select item |
| `h` | Toggle help |
| `q` | Quit application |
| `r` | Refresh projects |

### Interface Layout

```
┌─ Orchy - Project Orchestration ─────────────────────────────────┐
│ Dashboard │ Projects │ Tasks │ Agents │ Issues                  │
└─────────────────────────────────────────────────────────────────┘
┌─ Current Project ─────────────┐ ┌─ Quick Stats ─────────────────┐
│ Project: My Project           │ │ Total Projects: 3             │
│ Description: ...              │ │ Recent Activity:              │
│ Tasks: 5                      │ │ • Projects loaded             │
│ Agents: 2                     │ │ • Ready for action            │
│ Issues: 1                     │ │                               │
└───────────────────────────────┘ └───────────────────────────────┘
┌─────────────────────────────────────────────────────────────────┐
│ Status: Ready | Press 'h' for help, 'q' to quit               │
└─────────────────────────────────────────────────────────────────┘
```

## Key Features

### 🔄 **Real-time Updates**
- Automatic project discovery
- Live status updates
- Responsive interface

### 🎯 **Project Management**
- Browse all available projects
- Select and switch between projects
- View detailed project information

### 📋 **Task Visualization**
- View all tasks in the current project
- See task status, priority, and assignments
- Track task dependencies

### 🤖 **Agent Management**
- View all agents in the current project
- See agent status and descriptions
- Track agent activity

### 🐛 **Issue Tracking**
- View all issues in the current project
- See issue status and details
- Track issue resolution

### ❓ **Built-in Help**
- Press `h` to access comprehensive help
- Context-sensitive guidance
- Keyboard shortcut reference

## Architecture

The TUI is built with a clean, modular architecture:

```
src/utils/tui.rs
├── App struct - Main application state
├── Tab enum - Tab navigation
├── AppMode enum - Application modes
├── UI rendering functions
├── Event handling
└── Terminal management
```

### Key Components

- **App State Management**: Centralized state with project data
- **Event Loop**: Async event handling with crossterm
- **Rendering Pipeline**: Efficient UI updates with ratatui
- **Navigation System**: Tab-based interface with keyboard shortcuts

## Comparison with CLI Modes

| Feature | CLI Commands | TUI Mode (Default) |
|---------|-------------|-------------------|
| **Interface** | Command-line | Visual interface |
| **Navigation** | Separate commands | Tab-based |
| **Real-time** | Static | Full real-time |
| **Visual Appeal** | Basic | Modern & colorful |
| **Ease of Use** | Expert-friendly | User-friendly |
| **Usage** | `orchy <command>` | `orchy` (default) |

## Benefits

### 🚀 **Improved Productivity**
- Quick navigation between different views
- Visual overview of project status
- Efficient keyboard shortcuts

### 👁️ **Better Visibility**
- Clear visual hierarchy
- Color-coded status indicators
- Organized information layout

### 🎮 **Enhanced User Experience**
- Intuitive navigation
- Responsive interface
- Professional appearance

## Technical Details

### Dependencies
- `ratatui`: Modern TUI framework
- `crossterm`: Cross-platform terminal manipulation
- `tokio`: Async runtime for event handling

### Performance
- Efficient rendering with minimal redraws
- Async event handling for responsiveness
- Optimized state management

### Compatibility
- Works on all major platforms (Linux, macOS, Windows)
- Terminal-agnostic (works with any modern terminal)
- No external dependencies required

## Future Enhancements

- [ ] Project creation and editing within TUI
- [ ] Agent management (add/remove/edit)
- [ ] Task management (create/update/assign)
- [ ] Issue management (create/update/close)
- [ ] Real-time collaboration features
- [ ] Custom themes and color schemes
- [ ] Search and filtering capabilities
- [ ] Export functionality

## Getting Started

1. **Build the project**: `cargo build`
2. **Start the TUI**: `./target/debug/orchy` (default behavior)
3. **Navigate**: Use Tab to switch between views
4. **Get help**: Press `h` for keyboard shortcuts
5. **Select project**: Go to Projects tab and press Enter
6. **Explore**: Navigate through Tasks, Agents, and Issues

### Command-Line Mode

For non-interactive operations, use specific commands:

```bash
# Create a project
orchy create --name "My Project" --idea "Description" --path "./my-project"

# List agents
orchy list-agents

# Add an agent
orchy add-agent --project "My Project" --name "Agent" --description "Desc" --file-path "/path/to/agent.json"

# View help
orchy --help
```

The TUI provides a modern, efficient way to manage your Orchy projects with a professional interface that scales from simple project browsing to complex project management workflows.
