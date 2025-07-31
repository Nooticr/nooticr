/// Example of the new simplified workflow
/// 
/// This example shows how the new 4-stage pipeline works:
/// 1. FeatureDev: Build MVP with 8-12 todos, verify each step works
/// 2. CodeReviewer: Review for quality, DRY, maintainability  
/// 3. QA: Comprehensive testing, fix any issues
/// 4. DevOps: CI/CD setup and deployment

use orchy::prompts::Prompts;

fn main() {
    // Example 1: Simple idea breakdown
    let idea = "Build a Vue.js todo app with user authentication";
    let context = "Need a simple task management app for small teams";
    let agents = vec!["FeatureDev".to_string(), "CodeReviewer".to_string(), "QA".to_string(), "DevOps".to_string()];
    let tech_stack = "Vue 3, TypeScript, Vite, Pinia, Vue Router, Vitest";

    println!("=== SIMPLIFIED IDEA BREAKDOWN ===");
    let breakdown_prompt = Prompts::idea_breakdown_user_prompt(
        idea,
        context,
        agents.clone(),
        tech_stack
    );
    println!("{}", breakdown_prompt);

    println!("\n=== FEATURE DEV WORKFLOW ===");
    // Example 2: FeatureDev working on todos
    let objective = "Build a todo app with add, edit, delete, and mark complete functionality";
    let existing_files = vec![
        ("package.json".to_string(), r#"{"name": "todo-app", "dependencies": {"vue": "^3.0.0"}}"#.to_string()),
        ("src/main.ts".to_string(), "import { createApp } from 'vue'\nimport App from './App.vue'\n\ncreateApp(App).mount('#app')".to_string()),
    ];
    
    let feature_prompt = Prompts::feature_dev_todo_prompt(
        objective,
        tech_stack,
        &existing_files,
        None // No current error
    );
    println!("{}", feature_prompt);

    println!("\n=== CODE REVIEW WORKFLOW ===");
    // Example 3: CodeReviewer reviewing the code
    let files_to_review = vec![
        ("src/components/TodoList.vue".to_string(), 
         "<template><div><todo-item v-for=\"todo in todos\" :key=\"todo.id\" :todo=\"todo\" /></div></template>".to_string()),
        ("src/stores/todoStore.ts".to_string(),
         "export const useTodoStore = defineStore('todo', () => { const todos = ref([]); return { todos }; })".to_string()),
    ];
    let focus_areas = vec![
        "Check for DRY violations".to_string(),
        "Ensure proper TypeScript usage".to_string(),
        "Validate Vue 3 best practices".to_string(),
    ];

    let review_prompt = Prompts::code_review_agent_prompt(
        tech_stack,
        &files_to_review,
        &focus_areas
    );
    println!("{}", review_prompt);

    println!("\n=== QA WORKFLOW ===");
    // Example 4: QA testing the application
    let test_types = vec![
        "Unit tests for components".to_string(),
        "Integration tests for store".to_string(),
        "E2E tests for user flows".to_string(),
        "Performance tests".to_string(),
    ];

    let qa_prompt = Prompts::qa_agent_prompt(
        tech_stack,
        &files_to_review,
        &test_types,
        None // No current test failures
    );
    println!("{}", qa_prompt);

    println!("\n=== DEVOPS WORKFLOW ===");
    // Example 5: DevOps setting up CI/CD
    let deployment_target = "Vercel";
    let project_files = vec![
        ("vite.config.ts".to_string(), "export default defineConfig({ plugins: [vue()] })".to_string()),
        ("package.json".to_string(), r#"{"scripts": {"build": "vite build", "test": "vitest"}}"#.to_string()),
    ];

    let devops_prompt = Prompts::devops_agent_prompt(
        tech_stack,
        &project_files,
        deployment_target,
        None // No current CI failures
    );
    println!("{}", devops_prompt);

    println!("\n=== WORKFLOW SUMMARY ===");
    println!("🔧 FeatureDev: Produces JSON ACTIONS to build features, verify each step works");
    println!("🔍 CodeReviewer: Produces JSON ACTIONS to fix code quality issues (DRY, maintainability)");
    println!("🧪 QA: Produces JSON ACTIONS to write/run/fix tests, ensure no regressions");
    println!("🚀 DevOps: Produces JSON ACTIONS to set up CI/CD, ensure deployment works");
    println!("\n🎯 KEY CHANGE: ALL AGENTS NOW PRODUCE EXECUTABLE JSON ACTIONS!");
    println!("   - No more text reports or reviews");
    println!("   - Every agent creates specific file operations");
    println!("   - Actions can be executed directly by the system");
    println!("   - Each agent works in loops, fixing issues before proceeding");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplified_workflow() {
        // Test that the new prompts are much shorter and focused
        let idea = "Simple Vue app";
        let context = "Test context";
        let agents = vec!["FeatureDev".to_string()];
        let tech_stack = "Vue 3";

        let prompt = Prompts::idea_breakdown_user_prompt(idea, context, agents, tech_stack);
        
        // The new prompt should be much shorter and focused
        assert!(prompt.contains("4-6 task pipeline"));
        assert!(prompt.contains("STAGE 1: FEATURE DEVELOPMENT"));
        assert!(prompt.contains("8-12 dependent todos"));
        assert!(prompt.contains("npm run dev"));
        assert!(prompt.len() < 5000); // Much shorter than the old version
    }

    #[test]
    fn test_feature_dev_prompt() {
        let objective = "Build todo functionality";
        let tech_stack = "Vue 3";
        let files = vec![];

        let prompt = Prompts::feature_dev_todo_prompt(objective, tech_stack, &files, None);

        assert!(prompt.contains("FeatureDev agent"));
        assert!(prompt.contains("8-12 small, dependent todos"));
        assert!(prompt.contains("npm run dev"));
        assert!(prompt.contains("verify tests pass"));
        assert!(prompt.contains("MUST RETURN JSON ACTIONS"));
        assert!(prompt.contains("FEATURE DEV ACTION EXAMPLES"));
    }

    #[test]
    fn test_code_review_prompt() {
        let tech_stack = "Vue 3";
        let files = vec![];
        let focus = vec!["DRY violations".to_string()];

        let prompt = Prompts::code_review_agent_prompt(tech_stack, &files, &focus);

        assert!(prompt.contains("CodeReviewer agent"));
        assert!(prompt.contains("DRY Violations"));
        assert!(prompt.contains("maintainability"));
        assert!(prompt.contains("MUST produce JSON ACTIONS"));
        assert!(prompt.contains("CODE REVIEW ACTION EXAMPLES"));
    }

    #[test]
    fn test_qa_prompt() {
        let tech_stack = "Vue 3";
        let files = vec![];
        let test_types = vec!["Unit tests".to_string()];

        let prompt = Prompts::qa_agent_prompt(tech_stack, &files, &test_types, None);

        assert!(prompt.contains("QA agent"));
        assert!(prompt.contains("Unit Tests"));
        assert!(prompt.contains("All tests must pass"));
        assert!(prompt.contains("MUST produce JSON ACTIONS"));
        assert!(prompt.contains("QA ACTION EXAMPLES"));
    }

    #[test]
    fn test_devops_prompt() {
        let tech_stack = "Vue 3";
        let files = vec![];
        let target = "Vercel";

        let prompt = Prompts::devops_agent_prompt(tech_stack, &files, target, None);

        assert!(prompt.contains("DevOps agent"));
        assert!(prompt.contains("CI/CD"));
        assert!(prompt.contains("deployment"));
        assert!(prompt.contains("MUST produce JSON ACTIONS"));
        assert!(prompt.contains("DEVOPS ACTION EXAMPLES"));
    }
}
