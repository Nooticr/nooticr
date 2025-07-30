/// Database schema definitions for SQLite

pub const CREATE_PROJECTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    idea TEXT NOT NULL,
    name TEXT NOT NULL,
    repository_url TEXT,
    project_path TEXT NOT NULL,
    status TEXT NOT NULL,
    tech_stack TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
)
"#;

pub const CREATE_TASKS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    priority TEXT NOT NULL,
    estimated_complexity INTEGER,
    estimated_duration INTEGER,
    created_at TEXT,
    updated_at TEXT,
    completed_at TEXT,
    due_date TEXT,
    rapporter_id TEXT,
    assigned_to_id TEXT,
    pull_request_id TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (rapporter_id) REFERENCES agents(id) ON DELETE SET NULL,
    FOREIGN KEY (assigned_to_id) REFERENCES agents(id) ON DELETE SET NULL,
    FOREIGN KEY (pull_request_id) REFERENCES pull_requests(id) ON DELETE SET NULL
)
"#;

pub const CREATE_AGENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    description TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_active_at TEXT,
    error_count INTEGER NOT NULL DEFAULT 0,
    total_tasks_completed INTEGER NOT NULL DEFAULT 0,
    recovery_attempts INTEGER NOT NULL DEFAULT 0,
    last_error_recovery_at TEXT,
    autonomous_recovery_enabled BOOLEAN NOT NULL DEFAULT 1,
    max_recovery_attempts INTEGER NOT NULL DEFAULT 3,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
)
"#;

pub const CREATE_ISSUES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS issues (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    github_issue_number INTEGER,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    assignee TEXT,
    branch_name TEXT,
    issue_type TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    closed_at TEXT,
    reopened_count INTEGER NOT NULL DEFAULT 0,
    remotly_synced BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
)
"#;

pub const CREATE_PULL_REQUESTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS pull_requests (
    id TEXT PRIMARY KEY,
    github_pr_number INTEGER,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    source_branch TEXT NOT NULL,
    target_branch TEXT NOT NULL,
    author TEXT NOT NULL,
    code_status TEXT NOT NULL,
    ci_attemps INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    merged_at TEXT,
    closed_at TEXT,
    remotly_synced BOOLEAN NOT NULL DEFAULT 0
)
"#;

pub const CREATE_COMMENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS comments (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    author TEXT NOT NULL,
    comment_type TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    remotly_synced BOOLEAN NOT NULL DEFAULT 0,
    -- Foreign keys for different parent types
    task_id TEXT,
    issue_id TEXT,
    pull_request_id TEXT,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE,
    FOREIGN KEY (pull_request_id) REFERENCES pull_requests(id) ON DELETE CASCADE,
    -- Ensure comment belongs to exactly one parent
    CHECK (
        (task_id IS NOT NULL AND issue_id IS NULL AND pull_request_id IS NULL) OR
        (task_id IS NULL AND issue_id IS NOT NULL AND pull_request_id IS NULL) OR
        (task_id IS NULL AND issue_id IS NULL AND pull_request_id IS NOT NULL)
    )
)
"#;

pub const CREATE_CODE_REVIEWS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS code_reviews (
    id TEXT PRIMARY KEY,
    pull_request_id TEXT NOT NULL,
    reviewer TEXT NOT NULL,
    approved BOOLEAN NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (pull_request_id) REFERENCES pull_requests(id) ON DELETE CASCADE
)
"#;

pub const CREATE_TASK_STATUS_HISTORY_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS task_status_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT NOT NULL,
    status TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
)
"#;

pub const CREATE_AGENT_STATUS_HISTORY_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS agent_status_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id TEXT NOT NULL,
    from_status TEXT,
    to_status TEXT NOT NULL,
    reason TEXT,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
)
"#;

pub const CREATE_AGENT_ERRORS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS agent_errors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id TEXT NOT NULL,
    error_type TEXT NOT NULL,
    error_message TEXT NOT NULL,
    context TEXT,
    timestamp TEXT NOT NULL,
    resolved BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
)
"#;

pub const CREATE_TASKS_HISTORY_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS tasks_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    task_data TEXT NOT NULL, -- JSON serialized task
    timestamp TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
)
"#;

pub const CREATE_PROJECT_DEPENDENCIES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS project_dependencies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL,
    dependency_url TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
)
"#;

pub const CREATE_TASK_TAGS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS task_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT NOT NULL,
    tag TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    UNIQUE(task_id, tag)
)
"#;

pub const CREATE_TASK_DEPENDENCIES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS task_dependencies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT NOT NULL,
    depends_on_task_id TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (depends_on_task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    UNIQUE(task_id, depends_on_task_id)
)
"#;

pub const CREATE_ISSUE_LABELS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS issue_labels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    label TEXT NOT NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE,
    UNIQUE(issue_id, label)
)
"#;

pub const CREATE_PULL_REQUEST_ASSIGNEES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS pull_request_assignees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pull_request_id TEXT NOT NULL,
    assignee TEXT NOT NULL,
    FOREIGN KEY (pull_request_id) REFERENCES pull_requests(id) ON DELETE CASCADE,
    UNIQUE(pull_request_id, assignee)
)
"#;

pub const CREATE_PULL_REQUEST_REVIEWERS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS pull_request_reviewers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pull_request_id TEXT NOT NULL,
    reviewer TEXT NOT NULL,
    FOREIGN KEY (pull_request_id) REFERENCES pull_requests(id) ON DELETE CASCADE,
    UNIQUE(pull_request_id, reviewer)
)
"#;

pub const CREATE_PULL_REQUEST_LABELS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS pull_request_labels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pull_request_id TEXT NOT NULL,
    label TEXT NOT NULL,
    FOREIGN KEY (pull_request_id) REFERENCES pull_requests(id) ON DELETE CASCADE,
    UNIQUE(pull_request_id, label)
)
"#;

pub const CREATE_CODE_STATUS_HISTORY_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS code_status_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pull_request_id TEXT NOT NULL,
    status TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (pull_request_id) REFERENCES pull_requests(id) ON DELETE CASCADE
)
"#;

pub const CREATE_CODE_REVIEW_COMMENTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS code_review_comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code_review_id TEXT NOT NULL,
    comment TEXT NOT NULL,
    FOREIGN KEY (code_review_id) REFERENCES code_reviews(id) ON DELETE CASCADE
)
"#;

pub const CREATE_ISSUE_STATUS_HISTORY_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS issue_status_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    from_status TEXT,
    to_status TEXT NOT NULL,
    reason TEXT,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(id) ON DELETE CASCADE
)
"#;

/// All table creation statements in dependency order
pub const ALL_TABLES: &[&str] = &[
    CREATE_PROJECTS_TABLE,
    CREATE_AGENTS_TABLE,
    CREATE_PULL_REQUESTS_TABLE,
    CREATE_TASKS_TABLE,
    CREATE_ISSUES_TABLE,
    CREATE_COMMENTS_TABLE,
    CREATE_CODE_REVIEWS_TABLE,
    CREATE_TASK_STATUS_HISTORY_TABLE,
    CREATE_AGENT_STATUS_HISTORY_TABLE,
    CREATE_AGENT_ERRORS_TABLE,
    CREATE_TASKS_HISTORY_TABLE,
    CREATE_PROJECT_DEPENDENCIES_TABLE,
    CREATE_TASK_TAGS_TABLE,
    CREATE_TASK_DEPENDENCIES_TABLE,
    CREATE_ISSUE_LABELS_TABLE,
    CREATE_PULL_REQUEST_ASSIGNEES_TABLE,
    CREATE_PULL_REQUEST_REVIEWERS_TABLE,
    CREATE_PULL_REQUEST_LABELS_TABLE,
    CREATE_CODE_STATUS_HISTORY_TABLE,
    CREATE_CODE_REVIEW_COMMENTS_TABLE,
    CREATE_ISSUE_STATUS_HISTORY_TABLE,
];
