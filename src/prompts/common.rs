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
        r#"🚨 CRITICAL JSON-ONLY RESPONSE REQUIREMENTS 🚨

⚠️ ABSOLUTELY NO TEXT OUTSIDE OF JSON ⚠️
⚠️ DO NOT START WITH EXPLANATIONS ⚠️  
⚠️ DO NOT END WITH SUMMARIES ⚠️
⚠️ NO "Here are the actions:" OR SIMILAR TEXT ⚠️

YOUR RESPONSE MUST:
✅ Start immediately with [ (opening bracket)
✅ End immediately with ] (closing bracket)  
✅ Contain ONLY valid JSON array of actions
✅ Use proper JSON escaping: \\n for newlines, \\" for quotes, \\\\ for backslashes
✅ Have all braces and brackets properly matched

❌ NEVER include text like:
❌ "Here are the actions to complete this task:"
❌ "The following JSON actions will..."  
❌ "I'll create these files:"
❌ "Summary: The above actions will..."

EXAMPLE CORRECT FORMAT:
[
    {
        "Write": {
            "path": "src/App.vue",
            "content": "<template>\\n  <div>Hello</div>\\n</template>"
        }
    }
]

⚠️ FAILURE TO FOLLOW THIS FORMAT WILL CAUSE SYSTEM ERRORS ⚠️"#
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

    /// Global actions list - ALL AGENTS MUST PRODUCE THESE ACTIONS
    pub const AVAILABLE_ACTIONS: &'static str = r#"AVAILABLE ACTIONS (ALL AGENTS MUST RETURN JSON ACTIONS):
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
            • RunCommand - Execute shell commands"#;

    /// Available actions list (common across multiple prompts)
    pub fn available_actions_list() -> &'static str {
        Self::AVAILABLE_ACTIONS
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

    /// FeatureDev specific action examples
    pub fn feature_dev_action_examples() -> &'static str {
        r#"FEATURE DEV ACTION EXAMPLES:
            [
                {
                    "Write": {
                        "path": "src/components/TodoItem.vue",
                        "content": "<template>\n  <div class=\"todo-item\">\n    <input type=\"checkbox\" v-model=\"todo.completed\" />\n    <span>{{ todo.title }}</span>\n  </div>\n</template>\n\n<script setup>\ndefineProps(['todo'])\n</script>"
                    }
                },
                {
                    "RunCommand": {
                        "command": "npm run dev",
                        "env": []
                    }
                },
                {
                    "RunCommand": {
                        "command": "npm test",
                        "env": []
                    }
                }
            ]"#
    }

    /// QA specific action examples
    pub fn qa_action_examples() -> &'static str {
        r#"QA ACTION EXAMPLES:
            [
                {
                    "Write": {
                        "path": "tests/unit/TodoItem.test.js",
                        "content": "import { mount } from '@vue/test-utils'\nimport TodoItem from '@/components/TodoItem.vue'\n\ndescribe('TodoItem', () => {\n  it('renders todo title', () => {\n    const todo = { id: 1, title: 'Test Todo', completed: false }\n    const wrapper = mount(TodoItem, { props: { todo } })\n    expect(wrapper.text()).toContain('Test Todo')\n  })\n})"
                    }
                },
                {
                    "RunCommand": {
                        "command": "npm run test:unit",
                        "env": []
                    }
                },
                {
                    "RunCommand": {
                        "command": "npm run test:e2e",
                        "env": []
                    }
                }
            ]"#
    }

    /// DevOps specific action examples
    pub fn devops_action_examples() -> &'static str {
        r#"DEVOPS ACTION EXAMPLES:
            [
                {
                    "Write": {
                        "path": ".github/workflows/ci.yml",
                        "content": "name: CI\non:\n  push:\n    branches: [ main ]\n  pull_request:\n    branches: [ main ]\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n    - uses: actions/checkout@v3\n    - uses: actions/setup-node@v3\n      with:\n        node-version: '18'\n    - run: npm ci\n    - run: npm run build\n    - run: npm test"
                    }
                },
                {
                    "RunCommand": {
                        "command": "npm run build",
                        "env": []
                    }
                },
                {
                    "Write": {
                        "path": "vercel.json",
                        "content": "{\n  \"builds\": [\n    {\n      \"src\": \"package.json\",\n      \"use\": \"@vercel/static-build\"\n    }\n  ]\n}"
                    }
                }
            ]"#
    }

    /// CodeReviewer specific action examples
    pub fn code_review_action_examples() -> &'static str {
        r#"CODE REVIEW ACTION EXAMPLES:
            [
                {
                    "Replace": {
                        "path": "src/utils/helpers.js",
                        "old_content": "function formatDate(date) {\n  return date.toLocaleDateString()\n}\n\nfunction formatTime(date) {\n  return date.toLocaleTimeString()\n}",
                        "new_content": "// DRY: Extract common date formatting logic\nfunction formatDateTime(date, options = {}) {\n  return date.toLocaleString('en-US', options)\n}\n\nfunction formatDate(date) {\n  return formatDateTime(date, { dateStyle: 'short' })\n}\n\nfunction formatTime(date) {\n  return formatDateTime(date, { timeStyle: 'short' })\n}"
                    }
                },
                {
                    "Write": {
                        "path": "src/types/Todo.ts",
                        "content": "// Better type safety and separation of concerns\nexport interface Todo {\n  id: string\n  title: string\n  completed: boolean\n  createdAt: Date\n  updatedAt: Date\n}\n\nexport interface TodoFilters {\n  status: 'all' | 'active' | 'completed'\n  search: string\n}"
                    }
                }
            ]"#
    }

    /// Error Recovery specific action examples
    pub fn error_recovery_action_examples() -> &'static str {
        r#"ERROR RECOVERY ACTION EXAMPLES:

            // COMPILATION ERROR FIXES:
            [
                {
                    "Replace": {
                        "path": "src/components/TodoItem.vue",
                        "old_content": "import { defineProps } from 'vue'\n\nconst props = defineProps(['todo'])",
                        "new_content": "import { defineProps } from 'vue'\n\ninterface Props {\n  todo: { id: string; title: string; completed: boolean }\n}\n\nconst props = defineProps<Props>()"
                    }
                },
                {
                    "RunCommand": {
                        "command": "npm run type-check",
                        "env": []
                    }
                }
            ]

            // MISSING DEPENDENCY FIXES:
            [
                {
                    "RunCommand": {
                        "command": "npm install @types/node",
                        "env": []
                    }
                },
                {
                    "RunCommand": {
                        "command": "npm run dev",
                        "env": []
                    }
                }
            ]

            // IMPORT ERROR FIXES:
            [
                {
                    "Replace": {
                        "path": "src/main.ts",
                        "old_content": "import App from './App'",
                        "new_content": "import App from './App.vue'"
                    }
                },
                {
                    "RunCommand": {
                        "command": "npm run dev",
                        "env": []
                    }
                }
            ]

            // CONFIGURATION ERROR FIXES:
            [
                {
                    "Write": {
                        "path": "vite.config.ts",
                        "content": "import { defineConfig } from 'vite'\nimport vue from '@vitejs/plugin-vue'\n\nexport default defineConfig({\n  plugins: [vue()],\n  resolve: {\n    alias: {\n      '@': '/src'\n    }\n  }\n})"
                    }
                },
                {
                    "RunCommand": {
                        "command": "npm run dev",
                        "env": []
                    }
                }
            ]

            // RUST COMPILATION FIXES:
            [
                {
                    "Replace": {
                        "path": "src/main.rs",
                        "old_content": "fn main() {\n    let x = 5\n    println!(\"{}\", x);\n}",
                        "new_content": "fn main() {\n    let x = 5;\n    println!(\"{}\", x);\n}"
                    }
                },
                {
                    "RunCommand": {
                        "command": "cargo run",
                        "env": []
                    }
                }
            ]

            // PYTHON ERROR FIXES:
            [
                {
                    "Replace": {
                        "path": "main.py",
                        "old_content": "from utils import helper",
                        "new_content": "from .utils import helper"
                    }
                },
                {
                    "RunCommand": {
                        "command": "python main.py",
                        "env": []
                    }
                }
            ]"#
    }

    /// Unit Testing specific action examples
    pub fn unit_testing_action_examples() -> &'static str {
        r#"UNIT TESTING ACTION EXAMPLES:

            // JAVASCRIPT/JEST UNIT TESTS:
            [
                {
                    "Write": {
                        "path": "tests/unit/utils.test.js",
                        "content": "const { formatDate, validateEmail } = require('../../src/utils');\n\ndescribe('Utils', () => {\n  describe('formatDate', () => {\n    it('should format date correctly', () => {\n      const date = new Date('2023-12-25');\n      expect(formatDate(date)).toBe('2023-12-25');\n    });\n\n    it('should handle invalid date', () => {\n      expect(() => formatDate(null)).toThrow('Invalid date');\n    });\n  });\n\n  describe('validateEmail', () => {\n    it('should validate correct email', () => {\n      expect(validateEmail('test@example.com')).toBe(true);\n    });\n\n    it('should reject invalid email', () => {\n      expect(validateEmail('invalid-email')).toBe(false);\n    });\n  });\n});"
                    }
                },
                {
                    "RunCommand": {
                        "command": "npm test -- --coverage",
                        "env": []
                    }
                }
            ]

            // VUE/VITEST COMPONENT TESTS:
            [
                {
                    "Write": {
                        "path": "tests/unit/TodoItem.test.ts",
                        "content": "import { describe, it, expect } from 'vitest'\nimport { mount } from '@vue/test-utils'\nimport TodoItem from '@/components/TodoItem.vue'\n\ndescribe('TodoItem', () => {\n  it('renders todo title', () => {\n    const todo = { id: '1', title: 'Test Todo', completed: false }\n    const wrapper = mount(TodoItem, { props: { todo } })\n    expect(wrapper.text()).toContain('Test Todo')\n  })\n\n  it('emits toggle event when checkbox clicked', async () => {\n    const todo = { id: '1', title: 'Test Todo', completed: false }\n    const wrapper = mount(TodoItem, { props: { todo } })\n    await wrapper.find('input[type=\"checkbox\"]').trigger('change')\n    expect(wrapper.emitted('toggle')).toBeTruthy()\n    expect(wrapper.emitted('toggle')[0]).toEqual(['1'])\n  })\n\n  it('applies completed class when todo is completed', () => {\n    const todo = { id: '1', title: 'Test Todo', completed: true }\n    const wrapper = mount(TodoItem, { props: { todo } })\n    expect(wrapper.classes()).toContain('completed')\n  })\n})"
                    }
                },
                {
                    "RunCommand": {
                        "command": "npm run test:unit",
                        "env": []
                    }
                }
            ]

            // RUST UNIT TESTS:
            [
                {
                    "Write": {
                        "path": "src/utils.rs",
                        "content": "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\npub fn divide(a: f64, b: f64) -> Result<f64, String> {\n    if b == 0.0 {\n        Err(\"Division by zero\".to_string())\n    } else {\n        Ok(a / b)\n    }\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn test_add() {\n        assert_eq!(add(2, 3), 5);\n        assert_eq!(add(-1, 1), 0);\n        assert_eq!(add(0, 0), 0);\n    }\n\n    #[test]\n    fn test_divide_success() {\n        assert_eq!(divide(10.0, 2.0).unwrap(), 5.0);\n        assert_eq!(divide(7.0, 2.0).unwrap(), 3.5);\n    }\n\n    #[test]\n    fn test_divide_by_zero() {\n        assert!(divide(10.0, 0.0).is_err());\n        assert_eq!(divide(10.0, 0.0).unwrap_err(), \"Division by zero\");\n    }\n}"
                    }
                },
                {
                    "RunCommand": {
                        "command": "cargo test",
                        "env": []
                    }
                }
            ]"#
    }

    /// Integration Testing specific action examples
    pub fn integration_testing_action_examples() -> &'static str {
        r#"INTEGRATION TESTING ACTION EXAMPLES:

            // API INTEGRATION TESTS:
            [
                {
                    "Write": {
                        "path": "tests/integration/api.test.js",
                        "content": "const request = require('supertest');\nconst app = require('../../src/app');\n\ndescribe('API Integration Tests', () => {\n  beforeEach(async () => {\n    // Set up test database\n    await setupTestDatabase();\n  });\n\n  afterEach(async () => {\n    // Clean up test data\n    await cleanupTestDatabase();\n  });\n\n  describe('POST /api/todos', () => {\n    it('should create a new todo', async () => {\n      const todoData = { title: 'Test Todo', completed: false };\n      const response = await request(app)\n        .post('/api/todos')\n        .send(todoData)\n        .expect(201);\n      \n      expect(response.body.title).toBe('Test Todo');\n      expect(response.body.id).toBeDefined();\n    });\n\n    it('should validate required fields', async () => {\n      const response = await request(app)\n        .post('/api/todos')\n        .send({})\n        .expect(400);\n      \n      expect(response.body.error).toContain('title is required');\n    });\n  });\n\n  describe('GET /api/todos', () => {\n    it('should return all todos', async () => {\n      // Create test data\n      await createTestTodos();\n      \n      const response = await request(app)\n        .get('/api/todos')\n        .expect(200);\n      \n      expect(Array.isArray(response.body)).toBe(true);\n      expect(response.body.length).toBeGreaterThan(0);\n    });\n  });\n});"
                    }
                },
                {
                    "RunCommand": {
                        "command": "npm run test:integration",
                        "env": []
                    }
                }
            ]

            // DATABASE INTEGRATION TESTS:
            [
                {
                    "Write": {
                        "path": "tests/integration/database.test.js",
                        "content": "const { Pool } = require('pg');\nconst { TodoRepository } = require('../../src/repositories/TodoRepository');\n\ndescribe('Database Integration Tests', () => {\n  let pool;\n  let todoRepo;\n\n  beforeAll(async () => {\n    pool = new Pool({ connectionString: process.env.TEST_DATABASE_URL });\n    todoRepo = new TodoRepository(pool);\n  });\n\n  afterAll(async () => {\n    await pool.end();\n  });\n\n  beforeEach(async () => {\n    await pool.query('TRUNCATE TABLE todos RESTART IDENTITY CASCADE');\n  });\n\n  describe('TodoRepository', () => {\n    it('should create and retrieve todo', async () => {\n      const todoData = { title: 'Test Todo', completed: false };\n      const createdTodo = await todoRepo.create(todoData);\n      \n      expect(createdTodo.id).toBeDefined();\n      expect(createdTodo.title).toBe('Test Todo');\n      \n      const retrievedTodo = await todoRepo.findById(createdTodo.id);\n      expect(retrievedTodo.title).toBe('Test Todo');\n    });\n\n    it('should handle transaction rollback', async () => {\n      const client = await pool.connect();\n      try {\n        await client.query('BEGIN');\n        await client.query('INSERT INTO todos (title) VALUES ($1)', ['Test Todo']);\n        await client.query('ROLLBACK');\n        \n        const todos = await todoRepo.findAll();\n        expect(todos.length).toBe(0);\n      } finally {\n        client.release();\n      }\n    });\n  });\n});"
                    }
                },
                {
                    "RunCommand": {
                        "command": "npm run test:db",
                        "env": ["TEST_DATABASE_URL=postgresql://test:test@localhost:5432/test_db"]
                    }
                }
            ]"#
    }

    /// E2E Testing specific action examples
    pub fn e2e_testing_action_examples() -> &'static str {
        r#"E2E TESTING ACTION EXAMPLES:

            // PLAYWRIGHT E2E TESTS:
            [
                {
                    "Write": {
                        "path": "tests/e2e/todo-app.spec.ts",
                        "content": "import { test, expect } from '@playwright/test';\n\ntest.describe('Todo App E2E Tests', () => {\n  test.beforeEach(async ({ page }) => {\n    await page.goto('http://localhost:3000');\n  });\n\n  test('should add a new todo', async ({ page }) => {\n    // Add a new todo\n    await page.fill('[data-testid=\"todo-input\"]', 'Buy groceries');\n    await page.click('[data-testid=\"add-todo-btn\"]');\n    \n    // Verify todo appears in list\n    await expect(page.locator('[data-testid=\"todo-item\"]')).toContainText('Buy groceries');\n    \n    // Verify input is cleared\n    await expect(page.locator('[data-testid=\"todo-input\"]')).toHaveValue('');\n  });\n\n  test('should mark todo as completed', async ({ page }) => {\n    // Add a todo first\n    await page.fill('[data-testid=\"todo-input\"]', 'Test todo');\n    await page.click('[data-testid=\"add-todo-btn\"]');\n    \n    // Mark as completed\n    await page.click('[data-testid=\"todo-checkbox\"]');\n    \n    // Verify completed state\n    await expect(page.locator('[data-testid=\"todo-item\"]')).toHaveClass(/completed/);\n  });\n\n  test('should delete todo', async ({ page }) => {\n    // Add a todo first\n    await page.fill('[data-testid=\"todo-input\"]', 'Todo to delete');\n    await page.click('[data-testid=\"add-todo-btn\"]');\n    \n    // Delete the todo\n    await page.click('[data-testid=\"delete-todo-btn\"]');\n    \n    // Verify todo is removed\n    await expect(page.locator('[data-testid=\"todo-item\"]')).toHaveCount(0);\n  });\n\n  test('should filter todos', async ({ page }) => {\n    // Add multiple todos\n    await page.fill('[data-testid=\"todo-input\"]', 'Active todo');\n    await page.click('[data-testid=\"add-todo-btn\"]');\n    \n    await page.fill('[data-testid=\"todo-input\"]', 'Completed todo');\n    await page.click('[data-testid=\"add-todo-btn\"]');\n    \n    // Mark second todo as completed\n    await page.click('[data-testid=\"todo-checkbox\"]:nth-child(2)');\n    \n    // Filter by active\n    await page.click('[data-testid=\"filter-active\"]');\n    await expect(page.locator('[data-testid=\"todo-item\"]')).toHaveCount(1);\n    await expect(page.locator('[data-testid=\"todo-item\"]')).toContainText('Active todo');\n    \n    // Filter by completed\n    await page.click('[data-testid=\"filter-completed\"]');\n    await expect(page.locator('[data-testid=\"todo-item\"]')).toHaveCount(1);\n    await expect(page.locator('[data-testid=\"todo-item\"]')).toContainText('Completed todo');\n  });\n});"
                    }
                },
                {
                    "RunCommand": {
                        "command": "npx playwright test",
                        "env": []
                    }
                }
            ]

            // CYPRESS E2E TESTS:
            [
                {
                    "Write": {
                        "path": "cypress/e2e/user-authentication.cy.js",
                        "content": "describe('User Authentication Flow', () => {\n  beforeEach(() => {\n    cy.visit('/login');\n  });\n\n  it('should login with valid credentials', () => {\n    cy.get('[data-cy=\"email-input\"]').type('user@example.com');\n    cy.get('[data-cy=\"password-input\"]').type('password123');\n    cy.get('[data-cy=\"login-btn\"]').click();\n    \n    cy.url().should('include', '/dashboard');\n    cy.get('[data-cy=\"user-menu\"]').should('contain', 'user@example.com');\n  });\n\n  it('should show error for invalid credentials', () => {\n    cy.get('[data-cy=\"email-input\"]').type('invalid@example.com');\n    cy.get('[data-cy=\"password-input\"]').type('wrongpassword');\n    cy.get('[data-cy=\"login-btn\"]').click();\n    \n    cy.get('[data-cy=\"error-message\"]').should('contain', 'Invalid credentials');\n    cy.url().should('include', '/login');\n  });\n\n  it('should logout successfully', () => {\n    // Login first\n    cy.login('user@example.com', 'password123');\n    \n    // Logout\n    cy.get('[data-cy=\"user-menu\"]').click();\n    cy.get('[data-cy=\"logout-btn\"]').click();\n    \n    cy.url().should('include', '/login');\n    cy.get('[data-cy=\"login-form\"]').should('be.visible');\n  });\n\n  it('should redirect to login when accessing protected route', () => {\n    cy.visit('/dashboard');\n    cy.url().should('include', '/login');\n  });\n});"
                    }
                },
                {
                    "RunCommand": {
                        "command": "npx cypress run",
                        "env": []
                    }
                }
            ]"#
    }

    /// Performance Testing specific action examples
    pub fn performance_testing_action_examples() -> &'static str {
        r#"PERFORMANCE TESTING ACTION EXAMPLES:

            // K6 LOAD TESTING:
            [
                {
                    "Write": {
                        "path": "tests/performance/load-test.js",
                        "content": "import http from 'k6/http';\nimport { check, sleep } from 'k6';\nimport { Rate } from 'k6/metrics';\n\nconst errorRate = new Rate('errors');\n\nexport let options = {\n  stages: [\n    { duration: '2m', target: 10 }, // Ramp up to 10 users\n    { duration: '5m', target: 10 }, // Stay at 10 users\n    { duration: '2m', target: 20 }, // Ramp up to 20 users\n    { duration: '5m', target: 20 }, // Stay at 20 users\n    { duration: '2m', target: 0 },  // Ramp down to 0 users\n  ],\n  thresholds: {\n    http_req_duration: ['p(95)<500'], // 95% of requests must complete below 500ms\n    errors: ['rate<0.1'], // Error rate must be below 10%\n  },\n};\n\nexport default function() {\n  // Test homepage\n  let response = http.get('http://localhost:3000');\n  check(response, {\n    'homepage status is 200': (r) => r.status === 200,\n    'homepage loads in <200ms': (r) => r.timings.duration < 200,\n  }) || errorRate.add(1);\n  \n  sleep(1);\n  \n  // Test API endpoint\n  response = http.get('http://localhost:3000/api/todos');\n  check(response, {\n    'API status is 200': (r) => r.status === 200,\n    'API responds in <100ms': (r) => r.timings.duration < 100,\n    'API returns JSON': (r) => r.headers['Content-Type'] === 'application/json',\n  }) || errorRate.add(1);\n  \n  sleep(1);\n  \n  // Test POST request\n  const payload = JSON.stringify({ title: 'Performance test todo', completed: false });\n  response = http.post('http://localhost:3000/api/todos', payload, {\n    headers: { 'Content-Type': 'application/json' },\n  });\n  check(response, {\n    'POST status is 201': (r) => r.status === 201,\n    'POST responds in <200ms': (r) => r.timings.duration < 200,\n  }) || errorRate.add(1);\n  \n  sleep(1);\n}"
                    }
                },
                {
                    "RunCommand": {
                        "command": "k6 run tests/performance/load-test.js",
                        "env": []
                    }
                }
            ]

            // LIGHTHOUSE PERFORMANCE AUDIT:
            [
                {
                    "Write": {
                        "path": "tests/performance/lighthouse-audit.js",
                        "content": "const lighthouse = require('lighthouse');\nconst chromeLauncher = require('chrome-launcher');\nconst fs = require('fs');\n\nasync function runLighthouseAudit() {\n  const chrome = await chromeLauncher.launch({ chromeFlags: ['--headless'] });\n  \n  const options = {\n    logLevel: 'info',\n    output: 'html',\n    onlyCategories: ['performance', 'accessibility', 'best-practices', 'seo'],\n    port: chrome.port,\n  };\n  \n  const runnerResult = await lighthouse('http://localhost:3000', options);\n  \n  // Save the report\n  const reportHtml = runnerResult.report;\n  fs.writeFileSync('lighthouse-report.html', reportHtml);\n  \n  // Check performance score\n  const performanceScore = runnerResult.lhr.categories.performance.score * 100;\n  console.log(`Performance Score: ${performanceScore}`);\n  \n  // Assert performance targets\n  const metrics = runnerResult.lhr.audits;\n  const fcp = metrics['first-contentful-paint'].numericValue;\n  const lcp = metrics['largest-contentful-paint'].numericValue;\n  const cls = metrics['cumulative-layout-shift'].numericValue;\n  \n  console.log(`First Contentful Paint: ${fcp}ms`);\n  console.log(`Largest Contentful Paint: ${lcp}ms`);\n  console.log(`Cumulative Layout Shift: ${cls}`);\n  \n  // Performance assertions\n  if (performanceScore < 90) {\n    throw new Error(`Performance score ${performanceScore} is below target of 90`);\n  }\n  \n  if (fcp > 1800) {\n    throw new Error(`First Contentful Paint ${fcp}ms is above target of 1800ms`);\n  }\n  \n  if (lcp > 2500) {\n    throw new Error(`Largest Contentful Paint ${lcp}ms is above target of 2500ms`);\n  }\n  \n  if (cls > 0.1) {\n    throw new Error(`Cumulative Layout Shift ${cls} is above target of 0.1`);\n  }\n  \n  await chrome.kill();\n  console.log('All performance targets met!');\n}\n\nrunLighthouseAudit().catch(console.error);"
                    }
                },
                {
                    "RunCommand": {
                        "command": "node tests/performance/lighthouse-audit.js",
                        "env": []
                    }
                }
            ]

            // ARTILLERY STRESS TESTING:
            [
                {
                    "Write": {
                        "path": "tests/performance/stress-test.yml",
                        "content": "config:\n  target: 'http://localhost:3000'\n  phases:\n    - duration: 60\n      arrivalRate: 10\n      name: \"Warm up\"\n    - duration: 120\n      arrivalRate: 50\n      name: \"Normal load\"\n    - duration: 60\n      arrivalRate: 100\n      name: \"Stress test\"\n  processor: \"./stress-test-processor.js\"\n\nscenarios:\n  - name: \"Browse and interact\"\n    weight: 70\n    flow:\n      - get:\n          url: \"/\"\n          capture:\n            - json: \"$.csrfToken\"\n              as: \"csrfToken\"\n      - think: 2\n      - get:\n          url: \"/api/todos\"\n      - think: 1\n      - post:\n          url: \"/api/todos\"\n          json:\n            title: \"Stress test todo {{ $randomString() }}\"\n            completed: false\n          headers:\n            X-CSRF-Token: \"{{ csrfToken }}\"\n      - think: 1\n      \n  - name: \"API only\"\n    weight: 30\n    flow:\n      - get:\n          url: \"/api/todos\"\n      - think: 0.5\n      - get:\n          url: \"/api/todos/{{ $randomInt(1, 100) }}\"\n      - think: 0.5"
                    }
                },
                {
                    "RunCommand": {
                        "command": "artillery run tests/performance/stress-test.yml",
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
