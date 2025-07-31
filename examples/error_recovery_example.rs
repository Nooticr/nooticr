/// Example of the Error Recovery Agent
/// 
/// This example shows how the ErrorRecovery agent analyzes errors and produces
/// specific actions to fix compilation, runtime, and build issues.

use orchy::prompts::Prompts;

fn main() {
    println!("=== ERROR RECOVERY AGENT EXAMPLES ===\n");

    // Example 1: TypeScript compilation error
    example_typescript_error();
    
    // Example 2: Missing dependency error
    example_missing_dependency();
    
    // Example 3: Rust compilation error
    example_rust_error();
    
    // Example 4: Vue.js import error
    example_vue_import_error();
    
    // Example 5: Python import error
    example_python_error();
}

fn example_typescript_error() {
    println!("🔧 EXAMPLE 1: TypeScript Compilation Error");
    println!("{}", "=".repeat(50));
    
    let tech_stack = "Vue 3, TypeScript, Vite";
    let command_that_failed = "npm run type-check";
    let error_output = r#"
src/components/TodoItem.vue:15:7 - error TS2339: Property 'id' does not exist on type 'unknown'.

15   todo.id
         ~~

src/components/TodoItem.vue:16:7 - error TS2339: Property 'title' does not exist on type 'unknown'.

16   todo.title
         ~~~~~

Found 2 errors in the same file, starting at: src/components/TodoItem.vue:15
"#;
    
    let project_files = vec![
        ("src/components/TodoItem.vue".to_string(), 
         r#"<template>
  <div class="todo-item">
    <input type="checkbox" v-model="todo.completed" />
    <span>{{ todo.title }}</span>
  </div>
</template>

<script setup lang="ts">
import { defineProps } from 'vue'

const props = defineProps(['todo'])
const todo = props.todo
</script>"#.to_string()),
        ("package.json".to_string(), 
         r#"{"name": "todo-app", "scripts": {"type-check": "vue-tsc --noEmit"}}"#.to_string()),
    ];
    
    let recent_changes = Some("Added TypeScript support to Vue component");
    
    let prompt = Prompts::error_recovery_prompt(
        tech_stack,
        error_output,
        command_that_failed,
        &project_files,
        recent_changes
    );
    
    println!("{}", prompt);
    println!("\n");
}

fn example_missing_dependency() {
    println!("📦 EXAMPLE 2: Missing Dependency Error");
    println!("{}", "=".repeat(50));
    
    let tech_stack = "Vue 3, TypeScript, Vite, Pinia";
    let command_that_failed = "npm run dev";
    let error_output = r#"
✘ [ERROR] Could not resolve "@pinia/nuxt"

    src/stores/todoStore.ts:1:23:
      1 │ import { defineStore } from '@pinia/nuxt'
        ╵                        ~~~~~~~~~~~~~~

  You can mark the path "@pinia/nuxt" as external to exclude it from the bundle, which will remove this error and leave the unresolved path in the bundle.

✘ [ERROR] Build failed with 1 error:
error: Could not resolve "@pinia/nuxt"
"#;
    
    let project_files = vec![
        ("src/stores/todoStore.ts".to_string(),
         r#"import { defineStore } from '@pinia/nuxt'

export const useTodoStore = defineStore('todo', () => {
  const todos = ref([])
  return { todos }
})"#.to_string()),
        ("package.json".to_string(),
         r#"{"name": "todo-app", "dependencies": {"vue": "^3.0.0"}}"#.to_string()),
    ];
    
    let prompt = Prompts::error_recovery_prompt(
        tech_stack,
        error_output,
        command_that_failed,
        &project_files,
        None
    );
    
    println!("{}", prompt);
    println!("\n");
}

fn example_rust_error() {
    println!("🦀 EXAMPLE 3: Rust Compilation Error");
    println!("{}", "=".repeat(50));
    
    let tech_stack = "Rust";
    let command_that_failed = "cargo run";
    let error_output = r#"
error: expected `;`, found `println`
 --> src/main.rs:3:18
  |
3 |     let x = 5
  |              ^ help: add `;` here
4 |     println!("Value: {}", x);
  |     ------- unexpected token

error: aborting due to previous error
"#;
    
    let project_files = vec![
        ("src/main.rs".to_string(),
         r#"fn main() {
    let x = 5
    println!("Value: {}", x);
}"#.to_string()),
        ("Cargo.toml".to_string(),
         r#"[package]
name = "hello-world"
version = "0.1.0"
edition = "2021""#.to_string()),
    ];
    
    let prompt = Prompts::error_recovery_prompt(
        tech_stack,
        error_output,
        command_that_failed,
        &project_files,
        Some("Added a new variable declaration")
    );
    
    println!("{}", prompt);
    println!("\n");
}

fn example_vue_import_error() {
    println!("🖖 EXAMPLE 4: Vue.js Import Error");
    println!("{}", "=".repeat(50));
    
    let tech_stack = "Vue 3, Vite";
    let command_that_failed = "npm run dev";
    let error_output = r#"
✘ [ERROR] Could not resolve "./App"

    src/main.ts:2:17:
      2 │ import App from './App'
        ╵                 ~~~~~~~

  The module "./App" was not found on the file system:

    src/App.vue

  Did you mean to import "./App.vue" instead?

✘ [ERROR] Build failed with 1 error:
error: Could not resolve "./App"
"#;
    
    let project_files = vec![
        ("src/main.ts".to_string(),
         r#"import { createApp } from 'vue'
import App from './App'

createApp(App).mount('#app')"#.to_string()),
        ("src/App.vue".to_string(),
         r#"<template>
  <div id="app">
    <h1>Todo App</h1>
  </div>
</template>"#.to_string()),
    ];
    
    let prompt = Prompts::error_recovery_prompt(
        tech_stack,
        error_output,
        command_that_failed,
        &project_files,
        Some("Created main.ts entry point")
    );
    
    println!("{}", prompt);
    println!("\n");
}

fn example_python_error() {
    println!("🐍 EXAMPLE 5: Python Import Error");
    println!("{}", "=".repeat(50));
    
    let tech_stack = "Python 3.9, FastAPI";
    let command_that_failed = "python main.py";
    let error_output = r#"
Traceback (most recent call last):
  File "main.py", line 2, in <module>
    from utils import database
ModuleNotFoundError: No module named 'utils'
"#;
    
    let project_files = vec![
        ("main.py".to_string(),
         r#"from fastapi import FastAPI
from utils import database

app = FastAPI()

@app.get("/")
def read_root():
    return {"Hello": "World"}"#.to_string()),
        ("utils/database.py".to_string(),
         r#"def connect():
    return "Connected to database""#.to_string()),
        ("utils/__init__.py".to_string(), "".to_string()),
    ];
    
    let prompt = Prompts::error_recovery_prompt(
        tech_stack,
        error_output,
        command_that_failed,
        &project_files,
        Some("Added database utility module")
    );
    
    println!("{}", prompt);
    println!("\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recovery_prompt() {
        let tech_stack = "Vue 3, TypeScript";
        let command = "npm run dev";
        let error = "Type error in component";
        let files = vec![];
        
        let prompt = Prompts::error_recovery_prompt(tech_stack, error, command, &files, None);
        
        assert!(prompt.contains("ErrorRecovery agent"));
        assert!(prompt.contains("FAILED COMMAND: npm run dev"));
        assert!(prompt.contains("Type error in component"));
        assert!(prompt.contains("MUST produce JSON ACTIONS"));
        assert!(prompt.contains("ERROR RECOVERY ACTION EXAMPLES"));
    }

    #[test]
    fn test_error_recovery_with_recent_changes() {
        let tech_stack = "Rust";
        let command = "cargo build";
        let error = "compilation error";
        let files = vec![];
        let changes = Some("Added new function");
        
        let prompt = Prompts::error_recovery_prompt(tech_stack, error, command, &files, changes);
        
        assert!(prompt.contains("RECENT CHANGES THAT MAY HAVE CAUSED THE ERROR"));
        assert!(prompt.contains("Added new function"));
    }
}
