pub struct Prompts {}

impl Prompts {
    pub fn idea_breakdown_user_prompt(
        idea: &str,
        context: &str,
        available_agents_types: Vec<String>,
        tech_stack: &str
    ) -> String {
        format!(
            r#"Break down this software development idea into a comprehensive, well-structured project plan:

            PROJECT IDEA: {}

            TECHNOLOGY STACK: {}

            PROJECT CONTEXT:
            {}

            AVAILABLE AGENT TYPES: {:?}

            COMPREHENSIVE BREAKDOWN REQUIREMENTS:

            **Phase 1: Planning & Architecture (Critical Foundation)**
            - Requirements analysis and user story mapping
            - System architecture design and technology decisions
            - Database schema design and data modeling
            - API design and endpoint specification
            - Security architecture and authentication strategy
            - Performance requirements and scalability planning

            **Phase 2: Infrastructure & Backend (Core Systems)**
            - Development environment setup and CI/CD pipeline
            - Database setup and migration system
            - Core backend services and business logic
            - Authentication and authorization system
            - API implementation with proper error handling
            - Data validation and security measures

            **Phase 3: Frontend & Integration (User Experience)**
            - Frontend application structure and routing
            - UI component library and design system
            - API integration and state management
            - User authentication and session management
            - Responsive design and accessibility
            - Error handling and user feedback

            **Phase 4: Quality & Deployment (Production Readiness)**
            - Comprehensive testing strategy (unit, integration, e2e)
            - Performance optimization and monitoring
            - Security auditing and vulnerability assessment
            - Documentation (API docs, user guides, deployment)
            - Deployment automation and environment configuration
            - Monitoring, logging, and alerting setup

            TASK SPECIFICATION REQUIREMENTS:
            - Create 12-20 detailed tasks covering all development phases
            - Each task must have clear acceptance criteria and deliverables
            - Include estimated time ranges and complexity scores (1-10)
            - Specify exact technologies and frameworks to be used
            - Define proper dependency chains and parallel work opportunities
            - Include specific testing requirements for each component
            - Consider security, performance, and scalability from the start
            - Plan for proper error handling and edge cases
            - Include deployment and monitoring considerations

            AGENT ASSIGNMENT STRATEGY:
            - Match tasks to appropriate agent types based on specialization
            - Consider agent capabilities and technology expertise
            - Ensure balanced workload distribution across agents
            - Plan for knowledge transfer between dependent tasks

            Return a JSON array with detailed task specifications (important):

            [{{
                "id": "unique_task_id",
                "title": "Specific, actionable task title",
                "description": "Detailed description with acceptance criteria, deliverables, and technical specifications",
                "priority": "Critical/High/Medium/Low",
                "complexity": 1-10,
                "estimated_hours": "4-8 hours",
                "agent_type": "specific_agent_type",
                "tags": ["technology", "component", "phase"],
                "depends_on": ["prerequisite_task_ids"],
                "acceptance_criteria": [
                    "Specific measurable outcome 1",
                    "Specific measurable outcome 2"
                ],
                "deliverables": [
                    "Concrete deliverable 1",
                    "Concrete deliverable 2"
                ],
                "testing_requirements": "Specific testing approach and coverage",
                "security_considerations": "Security aspects to address",
                "performance_requirements": "Performance targets and considerations"
            }}]
            "#,
            idea, tech_stack, context, available_agents_types
        )
    }

    pub fn feature_development_user_prompt(
        task_description: &str,
        codebase_context: &str,
        tech_stack: &str,
        existing_files: &[(String, String)], // (file_path, content) pairs for context
        requirements: &str,
        acceptance_criteria: &[String],
    ) -> String {
        let files_section = existing_files
            .iter()
            .map(|(file_path, content)| format!("FILE: {}\n```\n{}\n```", file_path, content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let criteria_section = acceptance_criteria
            .iter()
            .enumerate()
            .map(|(i, criteria)| format!("{}. {}", i + 1, criteria))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"Implement this specific task with comprehensive development approach:

            FEATURE TASK: {}

            DETAILED REQUIREMENTS:
            {}

            ACCEPTANCE CRITERIA:
            {}

            EXISTING CODEBASE CONTEXT:
            {}

            RELEVANT FILES FOR CONTEXT:
            {}

            TECHNOLOGY STACK: {}

            COMPREHENSIVE IMPLEMENTATION REQUIREMENTS:

            **Code Quality & Standards:**
            1. Follow existing architectural patterns and code conventions
            2. Implement proper error handling with meaningful error messages
            3. Add comprehensive input validation and sanitization
            4. Include detailed documentation and inline comments
            5. Ensure type safety and memory safety (for applicable languages)
            6. Follow SOLID principles and clean code practices

            **Security & Performance:**
            7. Implement proper authentication and authorization checks
            8. Validate all user inputs and prevent injection attacks
            9. Optimize for performance and minimize resource usage
            10. Handle edge cases and potential failure scenarios
            11. Implement proper logging for debugging and monitoring
            12. Consider scalability and concurrent access patterns

            **Testing & Validation:**
            13. Write comprehensive unit tests for all new functionality
            14. Include integration tests for API endpoints and database operations
            15. Add end-to-end tests for critical user workflows
            16. Test error conditions and edge cases thoroughly
            17. Ensure test coverage meets project standards
            18. Include performance and load testing where applicable

            **Integration & Deployment:**
            19. Ensure seamless integration with existing systems
            20. Update database schemas with proper migrations
            21. Update API documentation and OpenAPI specs
            22. Consider backward compatibility and versioning
            23. Plan for feature flags and gradual rollout
            24. Update deployment scripts and configuration

            **Documentation & Maintenance:**
            25. Update user documentation and API guides
            26. Add troubleshooting guides and FAQ entries
            27. Document configuration options and environment variables
            28. Include monitoring and alerting setup
            29. Plan for future maintenance and updates

            Return a JSON array of actions to be executed in order (important):

            [{{
                "Write": {{
                    "path": "exact/file/path.ext",
                    "content": "complete file content with proper formatting"
                }}
            }},
            {{
                "Update": {{
                    "path": "existing/file/path.ext",
                    "content": "updated complete file content"
                }}
            }},
            {{
                "Replace": {{
                    "path": "existing/file/path.ext",
                    "old_content": "content to replace",
                    "new_content": "replacement content"
                }}
            }},
            {{
                "CreateDirectory": {{
                    "path": "new/directory/path"
                }}
            }},
            {{
                "Backup": {{
                    "path": "important/file.ext",
                    "backup_suffix": ".bak"
                }}
            }},
            {{
                "Append": {{
                    "path": "existing/file.ext",
                    "content": "content to append"
                }}
            }},
            {{
                "SetPermissions": {{
                    "path": "file/path.ext",
                    "permissions": "755"
                }}
            }},
            {{
                "Archive": {{
                    "source_paths": ["file1.txt", "file2.txt"],
                    "archive_path": "backup.tar.gz",
                    "format": "tar.gz"
                }}
            }},
            {{
                "Grep": {{
                    "pattern": "search_pattern",
                    "path": "file_or_directory",
                    "recursive": true,
                    "case_sensitive": false
                }}
            }},
            {{
                "RunCommand": {{
                    "command": "specific command to execute",
                    "env": [["ENV_VAR", "value"]]
                }}
            }}]

            AVAILABLE ACTIONS:
            • Write - Create new files with content
            • Read - Read file contents
            • Update - Update existing file with new content
            • Replace - Replace specific content in files
            • Delete - Remove files
            • Move - Move/rename files
            • Copy - Copy files
            • CreateDirectory - Create directories
            • RemoveDirectory - Remove directories (with recursive option)
            • ListDirectory - List directory contents
            • Backup - Create file backups
            • Append - Append content to files
            • SetPermissions - Set file permissions (Unix)
            • CreateSymlink - Create symbolic links
            • Grep - Search for patterns in files
            • Archive - Create archives (zip, tar, tar.gz)
            • Extract - Extract archives
            • Download - Download files from URLs
            • Watch - Watch files for changes
            • RunCommand - Execute shell commands

            IMPORTANT GUIDELINES:
            - Provide complete, production-ready code
            - Include all necessary imports and dependencies
            - Ensure proper file structure and organization
            - Add comprehensive error handling
            - Include meaningful variable and function names
            - Add detailed comments for complex logic
            - Follow the project's coding standards and conventions
            - Consider performance implications of your implementation
            - Ensure security best practices are followed
            - Make the code maintainable and extensible
            "#,
            task_description, requirements, criteria_section, codebase_context, files_section, tech_stack
        )
    }

    /// Enhanced prompt for individual task development with dependency awareness
    pub fn task_development_user_prompt(
        task_title: &str,
        task_description: &str,
        task_complexity: u8,
        task_priority: &str,
        task_tags: &[String],
        tech_stack: &str,
        existing_files: &[(String, String)], // (file_path, content) pairs for context
        completed_dependencies: &[String], // Names of completed dependency tasks
        acceptance_criteria: &[String],
        codebase_context: &str,
    ) -> String {
        let files_section = if existing_files.is_empty() {
            "No existing files in the project yet.".to_string()
        } else {
            existing_files
                .iter()
                .map(|(file_path, content)| {
                    let preview = if content.len() > 500 {
                        format!("{}...\n[Content truncated - {} total characters]", 
                               &content[..500], content.len())
                    } else {
                        content.clone()
                    };
                    format!("FILE: {}\n```\n{}\n```", file_path, preview)
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        };

        let criteria_section = if acceptance_criteria.is_empty() {
            "No specific acceptance criteria provided.".to_string()
        } else {
            acceptance_criteria
                .iter()
                .enumerate()
                .map(|(i, criteria)| format!("{}. {}", i + 1, criteria))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let dependencies_section = if completed_dependencies.is_empty() {
            "No dependencies - this is a foundational task.".to_string()
        } else {
            format!("This task builds upon the following completed tasks:\n{}",
                    completed_dependencies
                        .iter()
                        .enumerate()
                        .map(|(i, dep)| format!("{}. {}", i + 1, dep))
                        .collect::<Vec<_>>()
                        .join("\n"))
        };

        let tags_section = task_tags.join(", ");

        format!(
            r#"Implement this specific development task with precise, production-ready code:

            TASK DETAILS:
            Title: {}
            Description: {}
            Priority: {} (Critical/High/Medium/Low)
            Complexity: {}/10
            Categories: {}
            Technology Stack: {}

            DEPENDENCY CONTEXT:
            {}

            ACCEPTANCE CRITERIA:
            {}

            EXISTING CODEBASE:
            {}

            EXISTING FILES FOR CONTEXT:
            {}

            SPECIFIC IMPLEMENTATION REQUIREMENTS:

            **Task-Focused Development:**
            1. Focus ONLY on this specific task - do not implement unrelated features
            2. Build upon existing files and patterns where applicable
            3. Create only the files and code necessary for THIS task
            4. Ensure compatibility with existing codebase structure
            5. Follow the established naming conventions and code organization

            **Quality Standards:**
            6. Write production-ready, maintainable code
            7. Include comprehensive error handling and validation
            8. Add meaningful comments for complex logic
            9. Follow language-specific best practices and idioms
            10. Ensure type safety and proper resource management

            **Integration Requirements:**
            11. Respect existing API contracts and interfaces
            12. Maintain backward compatibility where applicable
            13. Update related configuration files if necessary
            14. Ensure proper imports and module structure
            15. Handle edge cases and error scenarios gracefully

            **Package and Dependency Management:**
            16. ALWAYS use the LATEST STABLE versions of packages and dependencies
            17. Follow the MOST UP-TO-DATE installation guides and best practices
            18. Use current syntax and API calls (avoid deprecated methods)
            19. For frontend-only projects (Vue/React), use LOCAL data storage (SQLite, LocalStorage)
            20. DO NOT create backend services unless explicitly specified in tech stack
            21. If no backend is specified, use client-side data management only

            **Testing Considerations:**
            22. Write code that is easily testable
            23. Include basic test files if this task involves core functionality
            24. Consider mocking requirements for external dependencies
            25. Ensure proper separation of concerns for unit testing

            **Documentation Requirements:**
            26. Update README or documentation files if this task changes user-facing functionality
            27. Add inline documentation for public APIs
            28. Include configuration examples where applicable

            **CRITICAL TECHNOLOGY STACK GUIDELINES:**

            FOR FRONTEND-ONLY PROJECTS (Vue, React):
            - Use ONLY client-side technologies and local storage
            - DO NOT create any backend servers (Express, Flask, etc.)
            - Use localStorage, sessionStorage, or IndexedDB for data persistence
            - For databases, use client-side options like SQLite WASM or similar
            - Focus on component architecture and state management
            - Use mock data or JSON files for initial data

            FOR BACKEND-ONLY PROJECTS (Rust):
            - Focus on API endpoints and server functionality
            - Use appropriate databases (PostgreSQL, SQLite, etc.)
            - Include proper error handling and logging
            - Create comprehensive API documentation

            FOR FULLSTACK PROJECTS:
            - Create separate backend and frontend directories
            - Ensure proper API communication between services
            - Use appropriate databases and state management
            - Include deployment configurations

            PACKAGE VERSION REQUIREMENTS:
            - Always use the latest stable versions (e.g., "^18.0.0", not "17.x")
            - Check official documentation for current installation methods
            - Use modern build tools (Vite instead of Webpack where applicable)
            - Follow current best practices and avoid deprecated APIs

            Return a JSON array of actions to be executed in order. Each action should be specific and complete:

            CRITICAL REQUIREMENTS FOR ACTIONS:
            - CreateDirectory: Create any necessary directory structure first
            - Write: Create complete, functional files with all necessary content
            - Update: Modify existing files with precise changes
            - RunCommand: Include any build, install, or setup commands needed
            - Each file should be complete and functional, not just a skeleton
            - Include ALL necessary imports, dependencies, and configurations
            - Ensure the task is fully implemented and ready for use

            AVAILABLE ACTIONS AND THEIR EXACT JSON FORMAT:

            {{
                "Write": {{
                    "path": "exact/file/path.ext",
                    "content": "complete file content - MUST be valid and complete code"
                }}
            }}

            {{
                "Read": {{
                    "path": "file/to/read.ext"
                }}
            }}

            {{
                "Update": {{
                    "path": "existing/file.ext", 
                    "content": "complete updated file content"
                }}
            }}

            {{
                "Replace": {{
                    "path": "existing/file.ext",
                    "old_content": "exact text to replace",
                    "new_content": "replacement text"
                }}
            }}

            {{
                "Delete": {{
                    "path": "file/to/delete.ext"
                }}
            }}

            {{
                "Move": {{
                    "old_path": "current/path.ext",
                    "new_path": "new/path.ext"
                }}
            }}

            {{
                "Copy": {{
                    "old_path": "source/file.ext",
                    "new_path": "destination/file.ext"
                }}
            }}

            {{
                "CreateDirectory": {{
                    "path": "directory/path"
                }}
            }}

            {{
                "RemoveDirectory": {{
                    "path": "directory/to/remove",
                    "recursive": true
                }}
            }}

            {{
                "ListDirectory": {{
                    "path": "directory/to/list",
                    "recursive": false
                }}
            }}

            {{
                "Append": {{
                    "path": "existing/file.ext",
                    "content": "content to append to end of file"
                }}
            }}

            {{
                "Backup": {{
                    "path": "file/to/backup.ext",
                    "backup_suffix": ".bak"
                }}
            }}

            {{
                "SetPermissions": {{
                    "path": "file/or/directory",
                    "permissions": "755"
                }}
            }}

            {{
                "CreateSymlink": {{
                    "target": "path/to/target",
                    "link_path": "path/to/symlink"
                }}
            }}

            {{
                "RunCommand": {{
                    "command": "shell command to execute",
                    "env": [["ENV_VAR", "value"], ["ANOTHER_VAR", "value2"]]
                }}
            }}

            {{
                "Grep": {{
                    "pattern": "search pattern or regex",
                    "path": "file/or/directory/to/search",
                    "recursive": true,
                    "case_sensitive": false
                }}
            }}

            {{
                "Archive": {{
                    "source_paths": ["file1.txt", "directory/", "file2.txt"],
                    "archive_path": "backup.tar.gz",
                    "format": "tar.gz"
                }}
            }}

            {{
                "Extract": {{
                    "archive_path": "archive.tar.gz",
                    "destination": "extract/destination/"
                }}
            }}

            {{
                "Download": {{
                    "url": "https://example.com/file.zip",
                    "destination": "local/file.zip"
                }}
            }}

            {{
                "Watch": {{
                    "path": "file/or/directory/to/watch",
                    "duration_seconds": 60
                }}
            }}

            EXAMPLE ACTION ARRAY:
            [
                {{
                    "CreateDirectory": {{
                        "path": "src/components"
                    }}
                }},
                {{
                    "Write": {{
                        "path": "src/components/TaskManager.tsx",
                        "content": "import React from 'react';\n\ninterface Task {{\n  id: string;\n  title: string;\n  completed: boolean;\n}}\n\nconst TaskManager: React.FC = () => {{\n  const [tasks, setTasks] = React.useState<Task[]>([]);\n  return (\n    <div className=\"task-manager\">\n      <h1>Task Manager</h1>\n    </div>\n  );\n}};\n\nexport default TaskManager;"
                    }}
                }},
                {{
                    "Update": {{
                        "path": "package.json",
                        "content": "{{\n  \"name\": \"my-app\",\n  \"version\": \"1.0.0\",\n  \"dependencies\": {{\n    \"react\": \"^18.0.0\"\n  }}\n}}"
                    }}
                }},
                {{
                    "RunCommand": {{
                        "command": "npm install",
                        "env": []
                    }}
                }}
            ]

            CRITICAL JSON FORMATTING REQUIREMENTS:
            - Return ONLY a valid JSON array of actions, no other text
            - Escape all special characters in strings (quotes, backslashes, newlines)
            - Use \\n for newlines, \\" for quotes, \\\\ for backslashes in content
            - Ensure all braces and brackets are properly matched
            - Do not include any comments or explanations outside the JSON
            - Validate that your JSON can be parsed by standard JSON parsers

            IMPORTANT GUIDELINES:
            - Provide COMPLETE, working code - not placeholders or comments like "// TODO"
            - Each file should compile/run successfully after creation
            - Include all necessary imports, types, and dependencies
            - Follow the existing project structure and conventions
            - Make the implementation specific to the task requirements
            - Ensure proper error handling and edge case coverage
            - Test that your actions would create a functional implementation
            - Double-check that all JSON is properly escaped and valid

            Focus on quality over quantity - create exactly what's needed for this task, no more, no less.
            "#,
            task_title,
            task_description,
            task_priority,
            task_complexity,
            tags_section,
            tech_stack,
            dependencies_section,
            criteria_section,
            codebase_context,
            files_section
        )
    }

    pub fn code_review_user_prompt(
        files_and_code: &[(String, String)], // (file_path, code_content) pairs
        requirements: &str,
        context: &str,
        pull_request_id: &str
    ) -> String {
        let files_section = files_and_code
            .iter()
            .map(|(file_path, code)| format!("FILE: {}\n```\n{}\n```", file_path, code))
            .collect::<Vec<_>>()
            .join("\n\n");

        format!(
            r#"Review this code implementation against the requirements:

            FILES AND CODE:
            {}

            REQUIREMENTS:
            {}

            CONTEXT:
            {}

            Please provide a thorough review focusing on:
            1. Does the code meet the stated requirements?
            2. Are there any bugs or edge cases missed?
            3. Is the code maintainable and well-structured?
            4. Are there security or performance concerns?
            5. Is error handling appropriate?
            6. Would you approve this code for production?
            7. Are there any performance bottlenecks?
            8. Is the code following best practices for the language/framework?
            9. Are there any potential security vulnerabilities?
            10. Is test coverage adequate?

            For each issue or suggestion, please specify:
            - The exact file path
            - Line numbers (if applicable)
            - Severity level (Critical, Major, Minor, Info)
            - Type of feedback (Issue, Suggestion, Nitpick, Praise, Question, Security, Performance)
            - Specific message explaining the concern
            - Suggested code changes (if applicable)
            - Code snippet showing the problematic area (if applicable)

            Return a JSON output that can be deserialized into this Rust struct (important):

            pub struct CodeReviewInput {{
                pub id: Option<Uuid>, // Always null
                pub pull_request_id: String, // Use "{}"
                pub approved: bool, // Overall approval status
                pub overall_comment: String, // General summary of the review
                pub comments: Vec<ReviewCommentInput>, // Detailed line-by-line feedback
                pub summary: ReviewSummary, // Overall metrics and scores
            }}

            pub struct ReviewCommentInput {{
                pub file_path: String, // Exact file path
                pub line_start: Option<u32>, // Starting line number (null for general file comments)
                pub line_end: Option<u32>, // Ending line number (null for single line)
                pub feedback_type: ReviewFeedbackType, // "Issue", "Suggestion", "Nitpick", "Praise", "Question", "Security", "Performance"
                pub severity: ReviewSeverity, // "Critical", "Major", "Minor", "Info"
                pub message: String, // Detailed explanation
                pub suggested_change: Option<String>, // Proposed code fix (null if not applicable)
                pub code_snippet: Option<String>, // Problematic code excerpt (null if not applicable)
            }}

            pub struct ReviewSummary {{
                pub total_files_reviewed: u32,
                pub total_lines_reviewed: u32,
                pub issues_found: u32,
                pub suggestions_made: u32,
                pub security_concerns: u32,
                pub performance_concerns: u32,
                pub test_coverage_adequate: bool,
                pub overall_quality_score: u8, // 0-100 score
            }}

            Example response format:
            {{
                "id": null,
                "pull_request_id": "{}",
                "approved": false,
                "overall_comment": "The code generally meets requirements but has several security concerns that need to be addressed before approval.",
                "comments": [
                    {{
                        "file_path": "src/auth.rs",
                        "line_start": 42,
                        "line_end": 45,
                        "feedback_type": "Security",
                        "severity": "Critical",
                        "message": "SQL injection vulnerability: user input is directly concatenated into SQL query without sanitization",
                        "suggested_change": "Use parameterized queries or an ORM to prevent SQL injection",
                        "code_snippet": "let query = format!(\"SELECT * FROM users WHERE id = {{}}\", user_id);"
                    }}
                ],
                "summary": {{
                    "total_files_reviewed": 3,
                    "total_lines_reviewed": 245,
                    "issues_found": 2,
                    "suggestions_made": 5,
                    "security_concerns": 1,
                    "performance_concerns": 1,
                    "test_coverage_adequate": false,
                    "overall_quality_score": 65
                }}
            }}
            "#,
            files_section, requirements, context, pull_request_id, pull_request_id
        )
    }

    pub fn conflict_resolution_user_prompt(
        conflicts_data: &[(String, String, String, String)], // (file_path, our_content, their_content, base_content) tuples
        branch_info: &str,
        context: &str,
        merge_commit_message: &str,
    ) -> String {
        let conflicts_section = conflicts_data
            .iter()
            .map(|(file_path, our_content, their_content, base_content)| {
                format!(
                    "FILE: {}\n\nOUR VERSION ({}):```\n{}\n```\n\nTHEIR VERSION ({}):```\n{}\n```\n\nBASE VERSION (common ancestor):```\n{}\n```",
                    file_path,
                    branch_info.split(" -> ").next().unwrap_or("current"),
                    our_content,
                    branch_info.split(" -> ").last().unwrap_or("incoming"),
                    their_content,
                    base_content
                )
            })
            .collect::<Vec<_>>()
            .join(&format!("\n\n{}\n\n", "=".repeat(80)));

        format!(
            r#"Resolve these Git merge conflicts with detailed analysis:

            CONFLICTS TO RESOLVE:
            {}

            BRANCH INFORMATION:
            {}

            CONTEXT:
            {}

            MERGE COMMIT MESSAGE:
            {}

            Please provide a comprehensive conflict resolution focusing on:
            1. **Conflict Analysis**: Identify the type of each conflict (ContentConflict, ModifyDelete, AddAdd, etc.)
            2. **Resolution Strategy**: Choose appropriate strategy (TakeOurs, TakeTheirs, ManualMerge, Custom, etc.)
            3. **Code Integration**: Merge functionality from both branches where possible
            4. **Style Consistency**: Maintain consistent code style and formatting
            5. **Functionality Preservation**: Ensure no functionality is lost unless intentional
            6. **Testing Requirements**: Identify what needs to be tested after resolution
            7. **Risk Assessment**: Evaluate confidence level and potential issues
            8. **Post-Resolution Actions**: Commands to run after applying the resolution

            For each conflict, provide:
            - Exact file path and line numbers (if applicable)
            - Conflict type classification
            - Resolution strategy used
            - Detailed explanation of the resolution
            - Confidence score (0-100)
            - Whether testing is required
            - Complete resolved content

            Return a JSON output that can be deserialized into this Rust struct (important):

            pub struct ConflictResolutionInput {{
                pub id: Option<Uuid>, // Always null
                pub merge_commit_message: String, // Use provided message or create appropriate one
                pub branch_info: BranchInfo, // Extract from branch information
                pub conflicts: Vec<ConflictDetailInput>, // Detailed resolution for each conflict
                pub summary: ConflictResolutionSummary, // Overall metrics and assessment
                pub post_resolution_actions: Vec<String>, // Commands to run after resolution
            }}

            pub struct ConflictDetailInput {{
                pub file_path: String, // Exact file path
                pub conflict_type: ConflictType, // "ContentConflict", "ModifyDelete", "AddAdd", "RenameRename", "RenameModify", "BinaryConflict", "SubmoduleConflict"
                pub line_start: Option<u32>, // Starting line number (null for file-level conflicts)
                pub line_end: Option<u32>, // Ending line number (null for single line)
                pub resolution_strategy: ResolutionStrategy, // "TakeOurs", "TakeTheirs", "ManualMerge", "Custom", "Delete", "Rename"
                pub resolved_content: String, // Complete resolved file content
                pub explanation: String, // Detailed explanation of the resolution
                pub confidence_score: u8, // 0-100 confidence in the resolution
                pub requires_testing: bool, // Whether this resolution needs testing
            }}

            pub struct BranchInfo {{
                pub source_branch: String, // Branch being merged from
                pub target_branch: String, // Branch being merged into
                pub source_commit: String, // Latest commit on source branch
                pub target_commit: String, // Latest commit on target branch
                pub merge_base: Option<String>, // Common ancestor commit (if available)
                pub source_author: Option<String>, // Author of source changes (if available)
                pub target_author: Option<String>, // Author of target changes (if available)
            }}

            pub struct ConflictResolutionSummary {{
                pub total_files_with_conflicts: u32,
                pub total_conflicts_resolved: u32,
                pub conflicts_by_type: HashMap<String, u32>, // Count of each conflict type
                pub resolution_strategies_used: HashMap<String, u32>, // Count of each strategy used
                pub high_risk_resolutions: u32, // Number of low-confidence or complex resolutions
                pub requires_manual_review: bool, // Whether human review is recommended
                pub estimated_test_time_minutes: u32, // Estimated time needed for testing
                pub overall_confidence_score: u8, // 0-100 overall confidence
            }}

            Example response format:
            {{
                "id": null,
                "merge_commit_message": "Merge feature/auth-system into main",
                "branch_info": {{
                    "source_branch": "feature/auth-system",
                    "target_branch": "main",
                    "source_commit": "abc123",
                    "target_commit": "def456",
                    "merge_base": "xyz789",
                    "source_author": "developer@company.com",
                    "target_author": "maintainer@company.com"
                }},
                "conflicts": [
                    {{
                        "file_path": "src/auth.rs",
                        "conflict_type": "ContentConflict",
                        "line_start": 42,
                        "line_end": 58,
                        "resolution_strategy": "ManualMerge",
                        "resolved_content": "// Complete resolved file content here",
                        "explanation": "Merged authentication methods from both branches, preserving new security features while maintaining backward compatibility",
                        "confidence_score": 85,
                        "requires_testing": true
                    }}
                ],
                "summary": {{
                    "total_files_with_conflicts": 3,
                    "total_conflicts_resolved": 5,
                    "conflicts_by_type": {{"ContentConflict": 4, "ModifyDelete": 1}},
                    "resolution_strategies_used": {{"ManualMerge": 3, "TakeOurs": 1, "TakeTheirs": 1}},
                    "high_risk_resolutions": 1,
                    "requires_manual_review": true,
                    "estimated_test_time_minutes": 45,
                    "overall_confidence_score": 78
                }},
                "post_resolution_actions": [
                    "cargo test",
                    "cargo clippy",
                    "git add .",
                    "git commit -m 'Resolve merge conflicts in auth system'"
                ]
            }}
            "#,
            conflicts_section, branch_info, context, merge_commit_message
        )
    }

    pub fn ci_cd_fix_user_prompt(
        pipeline_config: &str,
        error_logs: &str,
        project_context: &str,
        tech_stack: &str,
    ) -> String {
        format!(
            r#"Fix this CI/CD pipeline issue with comprehensive analysis and solution:

            PIPELINE CONFIGURATION:
            {}

            ERROR LOGS:
            {}

            PROJECT CONTEXT:
            {}

            TECHNOLOGY STACK: {}

            COMPREHENSIVE CI/CD ANALYSIS REQUIREMENTS:

            **Error Analysis:**
            1. Identify root cause of the pipeline failure
            2. Analyze error patterns and failure points
            3. Determine if it's a configuration, dependency, or code issue
            4. Check for environment-specific problems
            5. Identify any security or permission issues

            **Solution Strategy:**
            6. Provide step-by-step fix for the immediate issue
            7. Suggest improvements to prevent similar failures
            8. Optimize pipeline performance and reliability
            9. Ensure proper error handling and reporting
            10. Consider security best practices

            **Pipeline Optimization:**
            11. Review and optimize build stages and dependencies
            12. Implement proper caching strategies
            13. Add comprehensive testing stages
            14. Include security scanning and vulnerability checks
            15. Set up proper monitoring and alerting

            Return a JSON response with detailed analysis and fixes (important):

            {{
                "id": null,
                "issue_analysis": {{
                    "root_cause": "Detailed explanation of the root cause",
                    "failure_type": "Configuration/Dependency/Code/Environment/Security",
                    "affected_stages": ["stage1", "stage2"],
                    "severity": "Critical/High/Medium/Low",
                    "estimated_fix_time": "time estimate"
                }},
                "immediate_fixes": [
                    {{
                        "file_path": "path/to/config/file",
                        "change_type": "Update/Add/Remove",
                        "description": "What this fix does",
                        "content": "Fixed configuration content",
                        "reasoning": "Why this fix is needed"
                    }}
                ],
                "pipeline_improvements": [
                    {{
                        "improvement_type": "Performance/Security/Reliability/Monitoring",
                        "description": "Improvement description",
                        "implementation": "How to implement",
                        "benefits": "Expected benefits"
                    }}
                ],
                "testing_strategy": {{
                    "unit_tests": "Unit testing approach",
                    "integration_tests": "Integration testing strategy",
                    "security_tests": "Security testing requirements",
                    "performance_tests": "Performance testing plan"
                }},
                "monitoring_setup": {{
                    "metrics_to_track": ["metric1", "metric2"],
                    "alerting_rules": ["alert1", "alert2"],
                    "logging_configuration": "Logging setup details"
                }},
                "post_fix_actions": [
                    "Action 1: Verify fix works",
                    "Action 2: Update documentation",
                    "Action 3: Monitor for issues"
                ]
            }}
            "#,
            pipeline_config, error_logs, project_context, tech_stack
        )
    }

    pub fn docker_deployment_user_prompt(
        application_context: &str,
        deployment_requirements: &str,
        tech_stack: &str,
        environment: &str, // dev/staging/production
    ) -> String {
        format!(
            r#"Create comprehensive Docker deployment configuration for this application:

            APPLICATION CONTEXT:
            {}

            DEPLOYMENT REQUIREMENTS:
            {}

            TECHNOLOGY STACK: {}

            TARGET ENVIRONMENT: {}

            COMPREHENSIVE DOCKER DEPLOYMENT REQUIREMENTS:

            **Container Strategy:**
            1. Design multi-stage Docker builds for optimization
            2. Implement proper layer caching and build efficiency
            3. Use appropriate base images with security considerations
            4. Configure proper resource limits and health checks
            5. Implement graceful shutdown and signal handling

            **Security & Best Practices:**
            6. Run containers as non-root users
            7. Implement proper secrets management
            8. Use minimal base images and remove unnecessary packages
            9. Scan for vulnerabilities and security issues
            10. Configure proper network security and isolation

            **Orchestration & Scaling:**
            11. Create Docker Compose for local development
            12. Design Kubernetes manifests for production
            13. Implement horizontal pod autoscaling
            14. Configure load balancing and service discovery
            15. Set up persistent storage and data management

            **Monitoring & Observability:**
            16. Configure logging and log aggregation
            17. Set up metrics collection and monitoring
            18. Implement distributed tracing
            19. Configure health checks and readiness probes
            20. Set up alerting and notification systems

            Return a JSON response with complete deployment configuration (important):

            {{
                "id": null,
                "deployment_strategy": {{
                    "container_architecture": "Single/Multi-container/Microservices",
                    "orchestration_platform": "Docker Compose/Kubernetes/Docker Swarm",
                    "scaling_approach": "Horizontal/Vertical/Auto",
                    "environment_specific_configs": {{}}
                }},
                "docker_files": [
                    {{
                        "file_path": "Dockerfile",
                        "content": "Complete Dockerfile content",
                        "description": "Main application Dockerfile"
                    }},
                    {{
                        "file_path": "docker-compose.yml",
                        "content": "Complete docker-compose configuration",
                        "description": "Local development setup"
                    }}
                ],
                "kubernetes_manifests": [
                    {{
                        "file_path": "k8s/deployment.yaml",
                        "content": "Kubernetes deployment manifest",
                        "description": "Application deployment configuration"
                    }},
                    {{
                        "file_path": "k8s/service.yaml",
                        "content": "Kubernetes service manifest",
                        "description": "Service exposure configuration"
                    }}
                ],
                "configuration_files": [
                    {{
                        "file_path": "config/app.env",
                        "content": "Environment variables",
                        "description": "Application configuration"
                    }}
                ],
                "security_configuration": {{
                    "secrets_management": "How secrets are handled",
                    "network_policies": "Network security setup",
                    "rbac_configuration": "Role-based access control",
                    "security_scanning": "Vulnerability scanning setup"
                }},
                "monitoring_setup": {{
                    "logging_configuration": "Logging setup details",
                    "metrics_collection": "Metrics and monitoring",
                    "health_checks": "Health check configuration",
                    "alerting_rules": "Alerting and notification setup"
                }},
                "deployment_commands": [
                    "docker build -t app:latest .",
                    "docker-compose up -d",
                    "kubectl apply -f k8s/"
                ]
            }}
            "#,
            application_context, deployment_requirements, tech_stack, environment
        )
    }

    pub fn qa_analysis_user_prompt(
        application_code: &[(String, String)], // (file_path, content) pairs
        test_results: &str,
        requirements: &str,
        user_scenarios: &[String],
    ) -> String {
        let files_section = application_code
            .iter()
            .map(|(file_path, content)| format!("FILE: {}\n```\n{}\n```", file_path, content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let scenarios_section = user_scenarios
            .iter()
            .enumerate()
            .map(|(i, scenario)| format!("{}. {}", i + 1, scenario))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"Perform comprehensive Quality Assurance analysis for this application:

            APPLICATION CODE:
            {}

            CURRENT TEST RESULTS:
            {}

            REQUIREMENTS:
            {}

            USER SCENARIOS TO TEST:
            {}

            COMPREHENSIVE QA ANALYSIS REQUIREMENTS:

            **Functional Testing:**
            1. Verify all requirements are properly implemented
            2. Test all user workflows and edge cases
            3. Validate input/output behavior and data integrity
            4. Check error handling and recovery mechanisms
            5. Test integration points and external dependencies

            **Non-Functional Testing:**
            6. Performance testing and load capacity analysis
            7. Security vulnerability assessment
            8. Usability and accessibility evaluation
            9. Compatibility testing across platforms/browsers
            10. Scalability and resource usage analysis

            **Test Coverage Analysis:**
            11. Identify gaps in current test coverage
            12. Recommend additional test cases and scenarios
            13. Evaluate test quality and effectiveness
            14. Suggest automation opportunities
            15. Plan regression testing strategy

            **Quality Metrics:**
            16. Code quality assessment and technical debt
            17. Documentation completeness and accuracy
            18. Deployment and rollback procedures
            19. Monitoring and observability coverage
            20. Maintenance and support considerations

            Return a JSON response with comprehensive QA analysis (important):

            {{
                "id": null,
                "overall_quality_score": 85,
                "functional_analysis": {{
                    "requirements_coverage": {{
                        "covered_requirements": ["req1", "req2"],
                        "missing_requirements": ["req3"],
                        "coverage_percentage": 85
                    }},
                    "user_workflow_testing": [
                        {{
                            "scenario": "User login workflow",
                            "status": "Pass/Fail/Partial",
                            "issues_found": ["issue1", "issue2"],
                            "recommendations": ["rec1", "rec2"]
                        }}
                    ],
                    "edge_cases": [
                        {{
                            "case": "Empty input handling",
                            "tested": true,
                            "result": "Pass/Fail",
                            "notes": "Additional details"
                        }}
                    ]
                }},
                "non_functional_analysis": {{
                    "performance": {{
                        "response_times": "Average response time analysis",
                        "throughput": "Requests per second capacity",
                        "resource_usage": "CPU/Memory usage patterns",
                        "bottlenecks": ["bottleneck1", "bottleneck2"]
                    }},
                    "security": {{
                        "vulnerabilities_found": ["vuln1", "vuln2"],
                        "security_score": 75,
                        "recommendations": ["sec_rec1", "sec_rec2"]
                    }},
                    "usability": {{
                        "accessibility_score": 80,
                        "user_experience_issues": ["ux_issue1"],
                        "improvement_suggestions": ["ux_improvement1"]
                    }}
                }},
                "test_coverage_analysis": {{
                    "current_coverage": {{
                        "unit_tests": 85,
                        "integration_tests": 70,
                        "e2e_tests": 60,
                        "overall_coverage": 75
                    }},
                    "coverage_gaps": [
                        {{
                            "area": "Error handling",
                            "current_coverage": 40,
                            "recommended_tests": ["test1", "test2"]
                        }}
                    ],
                    "recommended_test_cases": [
                        {{
                            "test_type": "Unit/Integration/E2E",
                            "description": "Test case description",
                            "priority": "High/Medium/Low",
                            "implementation_effort": "Low/Medium/High"
                        }}
                    ]
                }},
                "quality_metrics": {{
                    "code_quality": {{
                        "maintainability_score": 80,
                        "technical_debt": "Low/Medium/High",
                        "code_smells": ["smell1", "smell2"],
                        "refactoring_recommendations": ["refactor1"]
                    }},
                    "documentation": {{
                        "completeness_score": 70,
                        "accuracy_score": 85,
                        "missing_documentation": ["doc1", "doc2"]
                    }}
                }},
                "critical_issues": [
                    {{
                        "severity": "Critical/High/Medium/Low",
                        "category": "Functional/Security/Performance/Usability",
                        "description": "Issue description",
                        "impact": "Impact on users/system",
                        "recommended_fix": "How to fix the issue",
                        "priority": "Immediate/High/Medium/Low"
                    }}
                ],
                "recommendations": [
                    {{
                        "category": "Testing/Performance/Security/Usability",
                        "recommendation": "Specific recommendation",
                        "implementation_effort": "Low/Medium/High",
                        "expected_impact": "Expected improvement",
                        "timeline": "Suggested implementation timeline"
                    }}
                ],
                "next_steps": [
                    "Immediate action 1",
                    "Short-term improvement 2",
                    "Long-term enhancement 3"
                ]
            }}
            "#,
            files_section, test_results, requirements, scenarios_section
        )
    }

    pub fn api_synchronization_user_prompt(
        backend_api_spec: &str,
        frontend_code: &[(String, String)], // (file_path, content) pairs
        api_documentation: &str,
        tech_stack: &str,
    ) -> String {
        let frontend_files_section = frontend_code
            .iter()
            .map(|(file_path, content)| format!("FILE: {}\n```\n{}\n```", file_path, content))
            .collect::<Vec<_>>()
            .join("\n\n");

        format!(
            r#"Synchronize API calls between backend and frontend with comprehensive analysis:

            BACKEND API SPECIFICATION:
            {}

            FRONTEND CODE:
            {}

            API DOCUMENTATION:
            {}

            TECHNOLOGY STACK: {}

            COMPREHENSIVE API SYNCHRONIZATION REQUIREMENTS:

            **API Contract Analysis:**
            1. Compare backend API endpoints with frontend API calls
            2. Identify missing, outdated, or incorrect API integrations
            3. Validate request/response data structures and types
            4. Check authentication and authorization implementations
            5. Verify error handling and status code management

            **Data Type Synchronization:**
            6. Ensure frontend models match backend data structures
            7. Validate serialization/deserialization compatibility
            8. Check for type safety and null handling
            9. Verify date/time format consistency
            10. Validate enum values and constants synchronization

            **API Client Generation:**
            11. Generate type-safe API client code for frontend
            12. Create request/response interfaces and types
            13. Implement proper error handling and retry logic
            14. Add request/response interceptors and middleware
            15. Configure timeout and connection management

            **Testing & Validation:**
            16. Generate integration tests for API endpoints
            17. Create mock data and test fixtures
            18. Implement contract testing between services
            19. Add API versioning and backward compatibility checks
            20. Set up automated API documentation generation

            Return a JSON response with comprehensive synchronization analysis (important):

            {{
                "id": null,
                "synchronization_analysis": {{
                    "api_coverage": {{
                        "total_backend_endpoints": 25,
                        "frontend_integrated_endpoints": 20,
                        "missing_integrations": ["POST /api/users", "DELETE /api/posts"],
                        "outdated_integrations": ["GET /api/profile"],
                        "coverage_percentage": 80
                    }},
                    "data_type_mismatches": [
                        {{
                            "endpoint": "GET /api/users",
                            "field": "created_at",
                            "backend_type": "ISO 8601 string",
                            "frontend_type": "Date object",
                            "severity": "High/Medium/Low",
                            "fix_required": true
                        }}
                    ],
                    "authentication_sync": {{
                        "backend_auth_method": "JWT Bearer Token",
                        "frontend_implementation": "Correct/Incorrect/Missing",
                        "token_refresh_logic": "Implemented/Missing",
                        "security_issues": ["issue1", "issue2"]
                    }}
                }},
                "generated_code": [
                    {{
                        "file_path": "src/api/types.ts",
                        "content": "TypeScript interface definitions",
                        "description": "API response/request types"
                    }},
                    {{
                        "file_path": "src/api/client.ts",
                        "content": "API client implementation",
                        "description": "Type-safe API client with error handling"
                    }},
                    {{
                        "file_path": "src/api/hooks.ts",
                        "content": "React hooks for API calls",
                        "description": "Custom hooks for data fetching"
                    }}
                ],
                "integration_fixes": [
                    {{
                        "endpoint": "POST /api/users",
                        "issue": "Missing frontend integration",
                        "fix_type": "Add/Update/Remove",
                        "implementation": "Code to implement the fix",
                        "testing_requirements": "How to test the fix"
                    }}
                ],
                "api_client_configuration": {{
                    "base_url_config": "Environment-based URL configuration",
                    "timeout_settings": "Request timeout configuration",
                    "retry_logic": "Retry strategy implementation",
                    "error_handling": "Global error handling setup",
                    "interceptors": "Request/response interceptor setup"
                }},
                "testing_strategy": {{
                    "unit_tests": [
                        {{
                            "test_file": "src/api/__tests__/client.test.ts",
                            "content": "Unit test implementation",
                            "description": "Tests for API client functions"
                        }}
                    ],
                    "integration_tests": [
                        {{
                            "test_file": "src/api/__tests__/integration.test.ts",
                            "content": "Integration test implementation",
                            "description": "End-to-end API integration tests"
                        }}
                    ],
                    "mock_data": [
                        {{
                            "file_path": "src/api/__mocks__/data.ts",
                            "content": "Mock data for testing",
                            "description": "Test fixtures and mock responses"
                        }}
                    ]
                }},
                "documentation_updates": [
                    {{
                        "file_path": "docs/api-integration.md",
                        "content": "Updated API integration documentation",
                        "description": "How to use the API client"
                    }}
                ],
                "migration_plan": {{
                    "breaking_changes": [
                        {{
                            "change": "User model updated",
                            "impact": "Frontend user interfaces need updates",
                            "migration_steps": ["step1", "step2"],
                            "timeline": "Immediate/Next release/Future"
                        }}
                    ],
                    "backward_compatibility": "How to maintain compatibility",
                    "rollout_strategy": "Gradual rollout plan"
                }},
                "monitoring_setup": {{
                    "api_metrics": "Metrics to track API usage",
                    "error_tracking": "Error monitoring configuration",
                    "performance_monitoring": "API performance tracking",
                    "alerting_rules": "When to alert on API issues"
                }}
            }}
            "#,
            backend_api_spec, frontend_files_section, api_documentation, tech_stack
        )
    }

    pub fn performance_optimization_user_prompt(
        application_code: &[(String, String)], // (file_path, content) pairs
        performance_metrics: &str,
        bottlenecks: &[String],
        tech_stack: &str,
    ) -> String {
        let files_section = application_code
            .iter()
            .map(|(file_path, content)| format!("FILE: {}\n```\n{}\n```", file_path, content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let bottlenecks_section = bottlenecks
            .iter()
            .enumerate()
            .map(|(i, bottleneck)| format!("{}. {}", i + 1, bottleneck))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"Optimize application performance with comprehensive analysis and solutions:

            APPLICATION CODE:
            {}

            CURRENT PERFORMANCE METRICS:
            {}

            IDENTIFIED BOTTLENECKS:
            {}

            TECHNOLOGY STACK: {}

            COMPREHENSIVE PERFORMANCE OPTIMIZATION REQUIREMENTS:

            **Performance Analysis:**
            1. Analyze CPU usage patterns and optimization opportunities
            2. Identify memory leaks and inefficient memory usage
            3. Evaluate database query performance and optimization
            4. Assess network latency and data transfer efficiency
            5. Review caching strategies and implementation

            **Code Optimization:**
            6. Optimize algorithms and data structures
            7. Implement efficient error handling and logging
            8. Reduce computational complexity where possible
            9. Optimize loops, recursion, and data processing
            10. Implement lazy loading and pagination

            **Infrastructure Optimization:**
            11. Configure proper caching layers (Redis, CDN)
            12. Optimize database indexes and query patterns
            13. Implement connection pooling and resource management
            14. Set up load balancing and horizontal scaling
            15. Configure compression and minification

            **Frontend Optimization:**
            16. Optimize bundle size and code splitting
            17. Implement efficient state management
            18. Optimize rendering and re-rendering patterns
            19. Add progressive loading and skeleton screens
            20. Implement service workers and offline capabilities

            Return a JSON response with comprehensive optimization plan:

            {{
                "id": null,
                "performance_analysis": {{
                    "current_metrics": {{
                        "response_time_avg": "500ms",
                        "throughput": "100 req/sec",
                        "cpu_usage": "75%",
                        "memory_usage": "2GB",
                        "database_query_time": "200ms"
                    }},
                    "target_metrics": {{
                        "response_time_target": "200ms",
                        "throughput_target": "500 req/sec",
                        "cpu_usage_target": "50%",
                        "memory_usage_target": "1GB",
                        "database_query_time_target": "50ms"
                    }},
                    "bottleneck_analysis": [
                        {{
                            "bottleneck": "Database queries",
                            "impact": "High/Medium/Low",
                            "root_cause": "Missing indexes on user_id column",
                            "optimization_potential": "70% improvement expected"
                        }}
                    ]
                }},
                "optimization_recommendations": [
                    {{
                        "category": "Database/Code/Infrastructure/Frontend",
                        "priority": "Critical/High/Medium/Low",
                        "description": "Specific optimization recommendation",
                        "implementation": "How to implement the optimization",
                        "expected_improvement": "Performance gain expected",
                        "effort_required": "Low/Medium/High",
                        "risk_level": "Low/Medium/High"
                    }}
                ],
                "code_optimizations": [
                    {{
                        "file_path": "src/services/user.rs",
                        "optimization_type": "Algorithm/Query/Caching/Memory",
                        "current_code": "Current inefficient code",
                        "optimized_code": "Optimized version",
                        "explanation": "Why this optimization helps",
                        "performance_impact": "Expected improvement"
                    }}
                ],
                "infrastructure_changes": [
                    {{
                        "component": "Database/Cache/Load Balancer/CDN",
                        "change_type": "Configuration/Addition/Upgrade",
                        "description": "Infrastructure change description",
                        "implementation_steps": ["step1", "step2"],
                        "cost_impact": "Cost implications",
                        "maintenance_requirements": "Ongoing maintenance needs"
                    }}
                ],
                "monitoring_improvements": {{
                    "new_metrics": ["metric1", "metric2"],
                    "alerting_thresholds": {{
                        "response_time": "500ms",
                        "error_rate": "1%",
                        "cpu_usage": "80%"
                    }},
                    "dashboard_updates": "Performance dashboard improvements",
                    "profiling_setup": "Continuous profiling configuration"
                }},
                "testing_strategy": {{
                    "load_testing": "Load testing plan and scenarios",
                    "stress_testing": "Stress testing approach",
                    "performance_regression_tests": "Automated performance testing",
                    "benchmarking": "Before/after performance comparison"
                }},
                "implementation_plan": {{
                    "phase_1": {{
                        "timeline": "1-2 weeks",
                        "optimizations": ["quick_win1", "quick_win2"],
                        "expected_improvement": "30% performance gain"
                    }},
                    "phase_2": {{
                        "timeline": "3-4 weeks",
                        "optimizations": ["medium_effort1", "medium_effort2"],
                        "expected_improvement": "50% additional gain"
                    }},
                    "phase_3": {{
                        "timeline": "2-3 months",
                        "optimizations": ["major_refactor1", "infrastructure_upgrade"],
                        "expected_improvement": "20% additional gain"
                    }}
                }}
            }}
            "#,
            files_section, performance_metrics, bottlenecks_section, tech_stack
        )
    }

    /// Agent error recovery prompt for autonomous problem-solving
    pub fn agent_error_recovery_prompt(
        agent_name: &str,
        agent_type: &str,
        task_title: Option<&str>,
        project_path: &str,
        tech_stack: &str,
        error_action_type: &str,
        error_action_description: &str,
        error_message: &str,
        error_code: Option<i32>,
        working_directory: Option<&str>,
        retry_count: u32,
        stdout: Option<&str>,
        stderr: Option<&str>,
        previous_actions: &[String],
        project_structure: &[String],
        relevant_files: &[(String, String)], // (relative_path, content) pairs
    ) -> String {
        let error_code_str = error_code.map_or("None".to_string(), |code| code.to_string());
        let working_dir_str = working_directory.unwrap_or("Not specified");
        let task_title_str = task_title.unwrap_or("No specific task");
        let stdout_str = stdout.unwrap_or("No stdout");
        let stderr_str = stderr.unwrap_or("No stderr");

        let previous_actions_section = if previous_actions.is_empty() {
            "No previous actions recorded".to_string()
        } else {
            previous_actions
                .iter()
                .enumerate()
                .map(|(i, action)| format!("{}. {}", i + 1, action))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let project_structure_section = if project_structure.is_empty() {
            "Project structure not available".to_string()
        } else {
            project_structure.join("\n")
        };

        let relevant_files_section = if relevant_files.is_empty() {
            "No relevant files provided".to_string()
        } else {
            relevant_files
                .iter()
                .map(|(path, content)| {
                    let extension = std::path::Path::new(path)
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("");
                    format!("### {}\n```{}\n{}\n```\n", path, extension, content)
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        format!(
            r#"# Agent Error Recovery and Problem Solving

You are an intelligent software development agent with the ability to analyze action execution failures and provide autonomous recovery solutions. Your role is to diagnose problems, understand their root causes, and suggest precise corrective actions.

## Context Information

**Agent Details:**
- Name: {}
- Type: {}
- Task: {}
- Project Path: {}
- Technology Stack: {}

**Error Information:**
- Action Type: {}
- Action Description: {}
- Error Message: {}
- Exit Code: {}
- Working Directory: {}
- Retry Count: {}

**Command Output:**
```
STDOUT:
{}

STDERR:
{}
```

**Previous Actions:**
{}

**Project Structure:**
```
{}
```

**Relevant Files:**
{}

## Your Task

Analyze the error and provide a comprehensive recovery plan. Consider:

1. **Root Cause Analysis**: What exactly went wrong and why?
2. **Environmental Factors**: Are there missing dependencies, permissions, or configuration issues?
3. **File System State**: Do required files/directories exist? Are permissions correct?
4. **Command/Tool Issues**: Is the command syntax correct? Are required tools installed?
5. **Project Configuration**: Are configuration files properly set up?
6. **Dependency Issues**: Are all required packages/libraries available?

## Output Format

Provide your response as a JSON object with the following structure:

```json
{{
  "analysis": "Detailed analysis of what went wrong and the context surrounding the failure",
  "root_cause": "The fundamental reason for the failure in 1-2 sentences",
  "confidence_level": 0.85,
  "recovery_actions": [
    {{
      "action_type": "Command|FileModification|FileCreation|FileDeletion",
      "description": "Clear description of what this action does",
      "command": "exact command to run (if action_type is Command)",
      "file_path": "path/to/file (if file operation)",
      "content": "file content (if FileModification or FileCreation)",
      "priority": 9,
      "estimated_success_rate": 0.9
    }}
  ],
  "preventive_measures": [
    "Future steps to prevent similar errors",
    "Configuration changes or checks to add"
  ],
  "should_retry_original": true,
  "estimated_recovery_time": 5
}}
```

## Recovery Action Guidelines

**Priority Levels (1-10):**
- 10: Critical - Must be done first (e.g., fix syntax errors, install missing tools)
- 8-9: High - Important for success (e.g., create missing directories, fix permissions)
- 5-7: Medium - Helpful optimizations (e.g., update configurations)
- 1-4: Low - Nice-to-have improvements

**Action Types:**
- **Command**: Execute a shell command
- **FileModification**: Update existing file content
- **FileCreation**: Create a new file
- **FileDeletion**: Remove a file or directory

**Best Practices:**
- Be specific and actionable
- Consider the technology stack and project context
- Prioritize actions that address the root cause
- Include verification steps where appropriate
- Consider rollback strategies for risky operations
- Suggest incremental changes over major refactoring

## Common Error Patterns

**Dependency Issues:**
- Missing package managers (npm, cargo, pip)
- Uninstalled dependencies
- Version conflicts
- Lock file inconsistencies

**Configuration Problems:**
- Missing environment variables
- Incorrect file paths
- Wrong permissions
- Missing configuration files

**Build/Compilation Errors:**
- Syntax errors in code
- Missing imports/modules
- Type errors
- Build tool configuration issues

**Runtime Errors:**
- Port conflicts
- File system permissions
- Network connectivity
- Resource constraints

Remember: Your goal is to provide actionable, precise solutions that an automated system can execute to recover from the error and continue with the original task."#,
            agent_name,
            agent_type,
            task_title_str,
            project_path,
            tech_stack,
            error_action_type,
            error_action_description,
            error_message,
            error_code_str,
            working_dir_str,
            retry_count,
            stdout_str,
            stderr_str,
            previous_actions_section,
            project_structure_section,
            relevant_files_section
        )
    }
}
