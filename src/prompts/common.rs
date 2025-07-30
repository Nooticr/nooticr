/// Common prompt building utilities to reduce duplication
pub struct PromptBuilder;

impl PromptBuilder {
    /// Format files section consistently across all prompts
    pub fn format_files_section(files: &[(String, String)], truncate_at: Option<usize>) -> String {
        if files.is_empty() {
            return "No existing files in the project yet.".to_string();
        }

        files
            .iter()
            .map(|(file_path, content)| {
                let formatted_content = if let Some(limit) = truncate_at {
                    if content.len() > limit {
                        format!("{}...\n[Content truncated - {} total characters]", 
                               &content[..limit], content.len())
                    } else {
                        content.clone()
                    }
                } else {
                    content.clone()
                };
                format!("FILE: {}\n```\n{}\n```", file_path, formatted_content)
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Format acceptance criteria consistently
    pub fn format_acceptance_criteria(criteria: &[String]) -> String {
        if criteria.is_empty() {
            return "No specific acceptance criteria provided.".to_string();
        }

        criteria
            .iter()
            .enumerate()
            .map(|(i, criteria)| format!("{}. {}", i + 1, criteria))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format dependencies section consistently
    pub fn format_dependencies_section(dependencies: &[String]) -> String {
        if dependencies.is_empty() {
            return "No dependencies - this is a foundational task.".to_string();
        }

        format!("This task builds upon the following completed tasks:\n{}",
                dependencies
                    .iter()
                    .enumerate()
                    .map(|(i, dep)| format!("{}. {}", i + 1, dep))
                    .collect::<Vec<_>>()
                    .join("\n"))
    }

    /// Format bottlenecks section consistently
    pub fn format_bottlenecks_section(bottlenecks: &[String]) -> String {
        bottlenecks
            .iter()
            .enumerate()
            .map(|(i, bottleneck)| format!("{}. {}", i + 1, bottleneck))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format user scenarios section consistently
    pub fn format_user_scenarios(scenarios: &[String]) -> String {
        scenarios
            .iter()
            .enumerate()
            .map(|(i, scenario)| format!("{}. {}", i + 1, scenario))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Common JSON formatting requirements section
    pub fn json_formatting_requirements() -> &'static str {
        r#"CRITICAL JSON FORMATTING REQUIREMENTS:
            - Return ONLY a valid JSON array of actions, no other text
            - Escape all special characters in strings (quotes, backslashes, newlines)
            - Use \\n for newlines, \\" for quotes, \\\\ for backslashes in content
            - Ensure all braces and brackets are properly matched
            - Do not include any comments or explanations outside the JSON
            - Validate that your JSON can be parsed by standard JSON parsers"#
    }

    /// Common implementation guidelines section
    pub fn implementation_guidelines() -> &'static str {
        r#"IMPORTANT GUIDELINES:
            - Provide COMPLETE, working code - not placeholders or comments like "// TODO"
            - Each file should compile/run successfully after creation
            - Include all necessary imports, types, and dependencies
            - Follow the existing project structure and conventions
            - Make the implementation specific to the task requirements
            - Ensure proper error handling and edge case coverage
            - Test that your actions would create a functional implementation
            - Double-check that all JSON is properly escaped and valid"#
    }

    /// Common quality standards section
    pub fn quality_standards() -> &'static str {
        r#"**Quality Standards:**
            6. Write production-ready, maintainable code
            7. Include comprehensive error handling and validation
            8. Add meaningful comments for complex logic
            9. Follow language-specific best practices and idioms
            10. Ensure type safety and proper resource management"#
    }

    /// Common integration requirements section
    pub fn integration_requirements() -> &'static str {
        r#"**Integration Requirements:**
            11. Respect existing API contracts and interfaces
            12. Maintain backward compatibility where applicable
            13. Update related configuration files if necessary
            14. Ensure proper imports and module structure
            15. Handle edge cases and error scenarios gracefully"#
    }

    /// Common package management guidelines
    pub fn package_management_guidelines() -> &'static str {
        r#"**Package and Dependency Management:**
            16. ALWAYS use the LATEST STABLE versions of packages and dependencies
            17. Follow the MOST UP-TO-DATE installation guides and best practices
            18. Use current syntax and API calls (avoid deprecated methods)
            19. For frontend-only projects (Vue/React), use LOCAL data storage (SQLite, LocalStorage)
            20. DO NOT create backend services unless explicitly specified in tech stack
            21. If no backend is specified, use client-side data management only"#
    }

    /// Common testing considerations section
    pub fn testing_considerations() -> &'static str {
        r#"**Testing Considerations:**
            22. Write code that is easily testable
            23. Include basic test files if this task involves core functionality
            24. Consider mocking requirements for external dependencies
            25. Ensure proper separation of concerns for unit testing"#
    }

    /// Common documentation requirements section
    pub fn documentation_requirements() -> &'static str {
        r#"**Documentation Requirements:**
            26. Update README or documentation files if this task changes user-facing functionality
            27. Add inline documentation for public APIs
            28. Include configuration examples where applicable"#
    }

    /// Available actions list (common across multiple prompts)
    pub fn available_actions_list() -> &'static str {
        r#"AVAILABLE ACTIONS:
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
            • RunCommand - Execute shell commands"#
    }

    /// Technology stack specific guidelines
    pub fn tech_stack_guidelines() -> &'static str {
        r#"**CRITICAL TECHNOLOGY STACK GUIDELINES:**

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
            - Include deployment configurations"#
    }

    /// Build a comprehensive prompt with common sections
    pub fn build_prompt(
        title: &str,
        context_sections: &[(&str, &str)],
        requirements_sections: &[&str],
        output_format: &str,
        include_json_requirements: bool,
        include_guidelines: bool,
    ) -> String {
        let mut prompt = format!("{}\n\n", title);

        // Add context sections
        for (section_title, content) in context_sections {
            prompt.push_str(&format!("{}:\n{}\n\n", section_title, content));
        }

        // Add requirements sections
        if !requirements_sections.is_empty() {
            prompt.push_str("COMPREHENSIVE REQUIREMENTS:\n\n");
            for (i, section) in requirements_sections.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, section));
            }
            prompt.push_str("\n");
        }

        // Add output format
        prompt.push_str(&format!("{}\n\n", output_format));

        // Add common sections if requested
        if include_json_requirements {
            prompt.push_str(&format!("{}\n\n", Self::json_formatting_requirements()));
        }

        if include_guidelines {
            prompt.push_str(&format!("{}\n\n", Self::implementation_guidelines()));
        }

        prompt
    }

    /// Create a standard JSON action format example
    pub fn json_action_example() -> &'static str {
        r#"EXAMPLE ACTION ARRAY:
            [
                {
                    "CreateDirectory": {
                        "path": "src/components"
                    }
                },
                {
                    "Write": {
                        "path": "src/components/TaskManager.tsx",
                        "content": "import React from 'react';\n\ninterface Task {\n  id: string;\n  title: string;\n  completed: boolean;\n}\n\nconst TaskManager: React.FC = () => {\n  const [tasks, setTasks] = React.useState<Task[]>([]);\n  return (\n    <div className=\"task-manager\">\n      <h1>Task Manager</h1>\n    </div>\n  );\n};\n\nexport default TaskManager;"
                    }
                },
                {
                    "RunCommand": {
                        "command": "npm install",
                        "env": []
                    }
                }
            ]"#
    }
}

/// Trait for prompt builders to ensure consistency
pub trait PromptBuilderTrait {
    fn build_prompt(&self) -> String;
    fn validate_inputs(&self) -> Result<(), String>;
}

/// Common prompt parameters structure
#[derive(Debug, Clone)]
pub struct CommonPromptParams {
    pub tech_stack: String,
    pub existing_files: Vec<(String, String)>,
    pub acceptance_criteria: Vec<String>,
    pub context: String,
}

impl CommonPromptParams {
    pub fn new(
        tech_stack: String,
        existing_files: Vec<(String, String)>,
        acceptance_criteria: Vec<String>,
        context: String,
    ) -> Self {
        Self {
            tech_stack,
            existing_files,
            acceptance_criteria,
            context,
        }
    }

    pub fn files_section(&self) -> String {
        PromptBuilder::format_files_section(&self.existing_files, Some(500))
    }

    pub fn criteria_section(&self) -> String {
        PromptBuilder::format_acceptance_criteria(&self.acceptance_criteria)
    }
}
