use crate::models::project::Project;
use crate::utils::cli::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap,
    },
    Frame, Terminal,
};
use std::io;
use tokio::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    ProjectSelection,
    CreateProject,
    AddAgent,
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Dashboard,
    Projects,
    Tasks,
    Agents,
    Issues,
}

impl Tab {
    fn titles() -> Vec<&'static str> {
        vec!["Dashboard", "Projects", "Tasks", "Agents", "Issues"]
    }
    
    fn from_index(index: usize) -> Self {
        match index {
            0 => Tab::Dashboard,
            1 => Tab::Projects,
            2 => Tab::Tasks,
            3 => Tab::Agents,
            4 => Tab::Issues,
            _ => Tab::Dashboard,
        }
    }
}

pub struct App {
    pub current_tab: Tab,
    pub tab_index: usize,
    pub mode: AppMode,
    pub current_project: Option<Project>,
    pub projects: Vec<(String, Project)>,
    pub project_list_state: ListState,
    pub status_message: Option<String>,
    pub input_buffer: String,
    pub should_quit: bool,
    pub show_help: bool,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            current_tab: Tab::Dashboard,
            tab_index: 0,
            mode: AppMode::Normal,
            current_project: None,
            projects: Vec::new(),
            project_list_state: ListState::default(),
            status_message: Some("Welcome to Orchy! Press 'h' for help, 'q' to quit.".to_string()),
            input_buffer: String::new(),
            should_quit: false,
            show_help: false,
        };
        app.project_list_state.select(Some(0));
        app
    }

    pub async fn refresh_projects(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.projects = discover_projects().await?;
        if self.projects.is_empty() {
            self.project_list_state.select(None);
        } else if self.project_list_state.selected().is_none() {
            self.project_list_state.select(Some(0));
        }
        Ok(())
    }

    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % Tab::titles().len();
        self.current_tab = Tab::from_index(self.tab_index);
    }

    pub fn previous_tab(&mut self) {
        if self.tab_index > 0 {
            self.tab_index -= 1;
        } else {
            self.tab_index = Tab::titles().len() - 1;
        }
        self.current_tab = Tab::from_index(self.tab_index);
    }

    pub fn next_project(&mut self) {
        if !self.projects.is_empty() {
            let i = match self.project_list_state.selected() {
                Some(i) => (i + 1) % self.projects.len(),
                None => 0,
            };
            self.project_list_state.select(Some(i));
        }
    }

    pub fn previous_project(&mut self) {
        if !self.projects.is_empty() {
            let i = match self.project_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.projects.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.project_list_state.select(Some(i));
        }
    }

    pub fn select_current_project(&mut self) {
        if let Some(selected) = self.project_list_state.selected() {
            if let Some((_, project)) = self.projects.get(selected) {
                self.current_project = Some(project.clone());
                self.status_message = Some(format!("Selected project: {}", project.name));
            }
        }
    }

    pub fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}

pub async fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initial data load
    app.refresh_projects().await?;

    loop {
        terminal.draw(|f| ui(f, app))?;

        // Handle events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.mode {
                        AppMode::Normal => handle_normal_mode(app, key.code).await?,
                        AppMode::Help => handle_help_mode(app, key.code),
                        _ => {} // Handle other modes as needed
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

async fn handle_normal_mode(app: &mut App, key: KeyCode) -> Result<(), Box<dyn std::error::Error>> {
    match key {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('h') => app.toggle_help(),
        KeyCode::Tab => app.next_tab(),
        KeyCode::BackTab => app.previous_tab(),
        KeyCode::Char('r') => {
            app.refresh_projects().await?;
            app.set_status("Projects refreshed".to_string());
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.current_tab == Tab::Projects {
                app.next_project();
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.current_tab == Tab::Projects {
                app.previous_project();
            }
        }
        KeyCode::Enter => {
            if app.current_tab == Tab::Projects {
                app.select_current_project();
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_help_mode(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('h') | KeyCode::Esc => app.toggle_help(),
        _ => {}
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Header with tabs
    render_header(f, chunks[0], app);
    
    // Main content
    render_main_content(f, chunks[1], app);
    
    // Footer with status
    render_footer(f, chunks[2], app);

    // Help overlay
    if app.show_help {
        render_help_popup(f, app);
    }
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let titles: Vec<Line> = Tab::titles()
        .iter()
        .map(|t| Line::from(*t))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Orchy - Project Orchestration"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(app.tab_index);

    f.render_widget(tabs, area);
}

fn render_main_content(f: &mut Frame, area: Rect, app: &App) {
    match app.current_tab {
        Tab::Dashboard => render_dashboard(f, area, app),
        Tab::Projects => render_projects(f, area, app),
        Tab::Tasks => render_tasks(f, area, app),
        Tab::Agents => render_agents(f, area, app),
        Tab::Issues => render_issues(f, area, app),
    }
}

fn render_dashboard(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Project info
    let project_info = if let Some(ref project) = app.current_project {
        format!(
            "Current Project: {}\n\nDescription: {}\nPath: {}\nTasks: {}\nAgents: {}\nIssues: {}",
            project.name,
            project.idea,
            project.project_path,
            project.tasks.len(),
            project.agents.len(),
            project.issues.len()
        )
    } else {
        "No project selected\n\nPress Tab to navigate to Projects tab\nPress Enter to select a project".to_string()
    };

    let project_block = Paragraph::new(project_info)
        .block(Block::default().borders(Borders::ALL).title("Current Project"))
        .wrap(Wrap { trim: true });

    f.render_widget(project_block, chunks[0]);

    // Quick stats
    let stats = format!(
        "Total Projects: {}\n\nRecent Activity:\n• Projects loaded\n• Ready for action",
        app.projects.len()
    );

    let stats_block = Paragraph::new(stats)
        .block(Block::default().borders(Borders::ALL).title("Quick Stats"))
        .wrap(Wrap { trim: true });

    f.render_widget(stats_block, chunks[1]);
}

fn render_projects(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .projects
        .iter()
        .map(|(name, project)| {
            let content = format!(
                "{} (Tasks: {}, Agents: {}, Issues: {})",
                name,
                project.tasks.len(),
                project.agents.len(),
                project.issues.len()
            );
            ListItem::new(content)
        })
        .collect();

    let projects_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Projects"))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    f.render_stateful_widget(projects_list, area, &mut app.project_list_state.clone());
}

fn render_tasks(f: &mut Frame, area: Rect, app: &App) {
    let content = if let Some(ref project) = app.current_project {
        if project.tasks.is_empty() {
            "No tasks found in the current project.".to_string()
        } else {
            project
                .tasks
                .iter()
                .enumerate()
                .map(|(i, task)| {
                    format!(
                        "{}. {} [{}]\n   Priority: {:?}\n   Status: {}\n",
                        i + 1,
                        task.title,
                        format_task_status(&task.status),
                        task.priority,
                        if task.assigned_to.is_some() { "Assigned" } else { "Unassigned" }
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    } else {
        "No project selected. Go to Projects tab and select a project.".to_string()
    };

    let tasks_block = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Tasks"))
        .wrap(Wrap { trim: true });

    f.render_widget(tasks_block, area);
}

fn render_agents(f: &mut Frame, area: Rect, app: &App) {
    let content = if let Some(ref project) = app.current_project {
        if project.agents.is_empty() {
            "No agents found in the current project.".to_string()
        } else {
            project
                .agents
                .iter()
                .enumerate()
                .map(|(i, agent)| {
                    format!(
                        "{}. {} [{}]\n   Description: {}\n   Created: {}\n",
                        i + 1,
                        agent.name,
                        format_agent_status(&agent.status),
                        agent.description,
                        agent.created_at.format("%Y-%m-%d %H:%M:%S")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    } else {
        "No project selected. Go to Projects tab and select a project.".to_string()
    };

    let agents_block = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Agents"))
        .wrap(Wrap { trim: true });

    f.render_widget(agents_block, area);
}

fn render_issues(f: &mut Frame, area: Rect, app: &App) {
    let content = if let Some(ref project) = app.current_project {
        if project.issues.is_empty() {
            "No issues found in the current project.".to_string()
        } else {
            project
                .issues
                .iter()
                .enumerate()
                .map(|(i, issue)| {
                    format!(
                        "{}. {} [{}]\n   Description: {}\n   Created: {}\n",
                        i + 1,
                        issue.title,
                        format_issue_status(&issue.status),
                        issue.body,
                        issue.created_at.format("%Y-%m-%d %H:%M:%S")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    } else {
        "No project selected. Go to Projects tab and select a project.".to_string()
    };

    let issues_block = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Issues"))
        .wrap(Wrap { trim: true });

    f.render_widget(issues_block, area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let status_text = app.status_message.as_deref().unwrap_or("Ready");
    let help_text = " | Press 'h' for help, 'q' to quit, Tab/Shift+Tab to navigate";

    let footer_text = format!("{}{}", status_text, help_text);

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(footer, area);
}

fn render_help_popup(f: &mut Frame, _app: &App) {
    let popup_area = centered_rect(80, 80, f.area());

    f.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from("Orchy - Help"),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  Tab / Shift+Tab  - Switch between tabs"),
        Line::from("  ↑/↓ or j/k       - Navigate lists"),
        Line::from("  Enter            - Select item"),
        Line::from(""),
        Line::from("Commands:"),
        Line::from("  h                - Toggle this help"),
        Line::from("  q                - Quit application"),
        Line::from("  r                - Refresh projects"),
        Line::from(""),
        Line::from("Tabs:"),
        Line::from("  Dashboard        - Overview of current project"),
        Line::from("  Projects         - Browse and select projects"),
        Line::from("  Tasks            - View tasks in current project"),
        Line::from("  Agents           - View agents in current project"),
        Line::from("  Issues           - View issues in current project"),
        Line::from(""),
        Line::from("Press 'h' or Esc to close this help"),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .style(Style::default().fg(Color::Yellow))
        )
        .wrap(Wrap { trim: true });

    f.render_widget(help_paragraph, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
