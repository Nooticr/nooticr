use orchy::enums::Action;
use tempfile::TempDir;
use tracing::debug;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    debug!("🧪 Testing Action Execution and File Creation");
    
    // Create temp directory for test
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();
    
    debug!("📁 Test project directory: {:?}", project_path);
    
    // Change to project directory
    let original_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    debug!("📂 Changing directory from {:?} to {:?}", original_dir, project_path);
    
    if let Err(e) = std::env::set_current_dir(&project_path) {
        debug!("❌ Failed to change to project directory: {}", e);
        return Err(e.into());
    } else {
        debug!("✅ Successfully changed to project directory");
    }
    
    // Create test actions similar to what the AI would generate
    let test_actions = vec![
        Action::CreateDirectory {
            path: "src".to_string(),
        },
        Action::CreateDirectory {
            path: "src/components".to_string(),
        },
        Action::Write {
            path: "package.json".to_string(),
            content: r#"{
  "name": "test-vue-app",
  "version": "1.0.0",
  "description": "Test Vue application",
  "main": "index.js",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "vue": "^3.4.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.0.0",
    "vite": "^5.0.0"
  }
}"#.to_string(),
        },
        Action::Write {
            path: "index.html".to_string(),
            content: r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <link rel="icon" href="/favicon.ico">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Test Vue App</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.js"></script>
  </body>
</html>"#.to_string(),
        },
        Action::Write {
            path: "src/main.js".to_string(),
            content: r#"import { createApp } from 'vue'
import App from './App.vue'

createApp(App).mount('#app')"#.to_string(),
        },
        Action::Write {
            path: "src/App.vue".to_string(),
            content: r#"<template>
  <div id="app">
    <h1>Hello Vue!</h1>
    <TodoList />
  </div>
</template>

<script>
import TodoList from './components/TodoList.vue'

export default {
  name: 'App',
  components: {
    TodoList
  }
}
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>"#.to_string(),
        },
        Action::Write {
            path: "src/components/TodoList.vue".to_string(),
            content: r#"<template>
  <div class="todo-list">
    <h2>Todo List</h2>
    <input v-model="newTodo" @keyup.enter="addTodo" placeholder="Add a new todo">
    <ul>
      <li v-for="todo in todos" :key="todo.id" :class="{ completed: todo.completed }">
        <span @click="toggleTodo(todo.id)">{{ todo.text }}</span>
        <button @click="removeTodo(todo.id)">Remove</button>
      </li>
    </ul>
  </div>
</template>

<script>
export default {
  name: 'TodoList',
  data() {
    return {
      newTodo: '',
      todos: [
        { id: 1, text: 'Learn Vue.js', completed: false },
        { id: 2, text: 'Build a todo app', completed: false }
      ]
    }
  },
  methods: {
    addTodo() {
      if (this.newTodo.trim()) {
        this.todos.push({
          id: Date.now(),
          text: this.newTodo,
          completed: false
        })
        this.newTodo = ''
      }
    },
    toggleTodo(id) {
      const todo = this.todos.find(t => t.id === id)
      if (todo) {
        todo.completed = !todo.completed
      }
    },
    removeTodo(id) {
      this.todos = this.todos.filter(t => t.id !== id)
    }
  }
}
</script>

<style scoped>
.todo-list {
  max-width: 500px;
  margin: 0 auto;
}

.completed {
  text-decoration: line-through;
  opacity: 0.6;
}

li {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px;
  border: 1px solid #ddd;
  margin: 5px 0;
}

button {
  background: #ff4444;
  color: white;
  border: none;
  padding: 5px 10px;
  cursor: pointer;
}
</style>"#.to_string(),
        },
        Action::Write {
            path: "vite.config.js".to_string(),
            content: r#"import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  server: {
    port: 3000
  }
})"#.to_string(),
        },
    ];
    
    debug!("🎬 Testing {} actions for file creation", test_actions.len());
    
    // Log each action before execution
    for (index, action) in test_actions.iter().enumerate() {
        debug!("   🎬 Action {}/{}: {:?}", index + 1, test_actions.len(), action);
    }
    
    // Execute the actions
    debug!("🔄 Executing actions in batch...");
    match Action::execute_batch(&test_actions).await {
        Ok(_) => {
            debug!("✅ Successfully executed all {} actions", test_actions.len());
            
            // Verify files were created
            debug!("🔍 Verifying file creation...");
            
            // List all files in project directory recursively
            fn list_files_recursive(dir: &std::path::Path, indent: usize) -> Result<(), Box<dyn std::error::Error>> {
                let prefix = "   ".repeat(indent);
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            debug!("{}📁 {}/", prefix, entry.file_name().to_string_lossy());
                            list_files_recursive(&path, indent + 1)?;
                        } else {
                            let size = fs::metadata(&path)?.len();
                            debug!("{}📄 {} ({} bytes)", prefix, entry.file_name().to_string_lossy(), size);
                            
                            // Read and show first few lines of text files
                            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                                if matches!(ext, "json" | "js" | "vue" | "html" | "css") {
                                    if let Ok(content) = fs::read_to_string(&path) {
                                        let lines: Vec<&str> = content.lines().take(3).collect();
                                        debug!("{}     Preview: {}", prefix, lines.join(" | "));
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
            
            debug!("📋 Final project structure:");
            list_files_recursive(&project_path, 0)?;
            
            // Verify specific files exist with expected content
            debug!("🔍 Verifying specific file contents...");
            
            let expected_files = vec![
                "package.json",
                "index.html", 
                "src/main.js",
                "src/App.vue",
                "src/components/TodoList.vue",
                "vite.config.js"
            ];
            
            let mut all_files_exist = true;
            for expected_file in &expected_files {
                let file_path = project_path.join(expected_file);
                if file_path.exists() {
                    let size = fs::metadata(&file_path)?.len();
                    debug!("   ✅ {} exists ({} bytes)", expected_file, size);
                    
                    // Check if it's not empty
                    if size == 0 {
                        debug!("   ⚠️  {} is empty!", expected_file);
                        all_files_exist = false;
                    }
                } else {
                    debug!("   ❌ {} MISSING", expected_file);
                    all_files_exist = false;
                }
            }
            
            if all_files_exist {
                debug!("🎉 SUCCESS: All expected files were created with content!");
                debug!("✅ Action execution system is working correctly");
                
                // Test if it looks like a valid Vue project
                let package_json_path = project_path.join("package.json");
                if let Ok(content) = fs::read_to_string(&package_json_path) {
                    if content.contains("vue") && content.contains("vite") {
                        debug!("✅ Generated project appears to be a valid Vue.js project");
                    }
                }
                
                debug!("📊 Summary:");
                debug!("   🎬 Actions executed: {}", test_actions.len());
                debug!("   📄 Files created: {}", expected_files.len()); 
                debug!("   📁 Directories created: 2 (src, src/components)");
                debug!("   ✅ File creation verification: PASSED");
                
            } else {
                debug!("❌ FAILURE: Some files were not created properly");
                return Err("File creation verification failed".into());
            }
        }
        Err(e) => {
            debug!("❌ Action execution failed: {}", e);
            return Err(format!("Action execution failed: {}", e).into());
        }
    }
    
    // Restore original directory
    let _ = std::env::set_current_dir(&original_dir);
    debug!("🔙 Restored original directory: {:?}", original_dir);
    
    debug!("🎉 File Creation Verification Test Complete!");
    
    Ok(())
}