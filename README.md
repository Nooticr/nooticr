# Orchy - AI-Powered Project Orchestrator

Orchy is an intelligent project orchestration system that uses AI to break down development ideas into structured tasks and automatically generates functional applications through task-by-task development.

## 🚀 Features

- **AI-Powered Project Planning**: Breaks down project ideas into comprehensive, dependency-ordered tasks
- **Task-by-Task Development**: Executes tasks individually with full dependency resolution
- **Multi-Technology Support**: Vue.js, React, Rust, and full-stack applications
- **Action-Based File Generation**: Creates real, functional project files through precise actions
- **Comprehensive Testing**: Built-in testing framework for generated applications
- **Agent Management**: Multi-agent system for specialized development tasks

## 📋 Prerequisites

Before using Orchy, ensure you have:

1. **Rust** (latest stable version)
2. **Gemini API Key** - Set as environment variable
3. **Node.js & npm** (for frontend projects)
4. **Git** (for version control)

### Environment Setup

```bash
# Set your Gemini API key
export GEMINI_API_KEY="your-gemini-api-key-here"

# Optional: Set custom agents directory
export ORCHY_AGENTS_DIR="/path/to/custom/agents"
```

## 🛠 Installation

1. **Clone the repository:**
```bash
git clone <repository-url>
cd orchy
```

2. **Build the project:**
```bash
cargo build --release
```

3. **Run tests (optional):**
```bash
cargo test
```

## 🎯 Quick Start

### 1. Create Your First Project

```bash
# Create a Vue.js todo application
./target/release/orchy create \
  --name "my-todo-app" \
  --idea "Create a modern Vue.js todo application with task management, categories, and local storage" \
  --path "./projects/my-todo-app" \
  --tech-stack "vue"
```

### 2. Create a Rust API

```bash
# Create a Rust backend API
./target/release/orchy create \
  --name "todo-api" \
  --idea "Build a REST API for todo management with user authentication and PostgreSQL database" \
  --path "./projects/todo-api" \
  --tech-stack "rust"
```

### 3. Create a Full-Stack Application

```bash
# Create a full-stack application
./target/release/orchy create \
  --name "fullstack-app" \
  --idea "Build a complete task management system with Rust backend and Vue.js frontend" \
  --path "./projects/fullstack-app" \
  --tech-stack "fullstack-rust-vue"
```

## 📖 Available Commands

### Project Management

```bash
# Create a new project
orchy create --name <name> --idea <description> --path <path> --tech-stack <stack>

# List all tasks in existing projects
orchy list-tasks

# List all agents
orchy list-agents

# List project issues
orchy list-issues
```

### Development Utilities

```bash
# Add sample data to a project (for testing)
orchy add-sample-data --project-name <name>

# Add a new agent to a project
orchy add-agent --project-name <name> --name <agent-name> --description <desc> --file-path <path>
```

## 🔧 Technology Stacks

Orchy supports the following technology stacks:

- **`vue`** - Vue.js 3 with Composition API, Pinia, and Vite
- **`react`** - React 18 with TypeScript, hooks, and modern tooling
- **`rust`** - Rust with Actix-web, Serde, and async/await
- **`fullstack-rust-vue`** - Complete stack with Rust backend and Vue.js frontend
- **`fullstack-rust-react`** - Complete stack with Rust backend and React frontend

## 📁 Project Structure

After creating a project, you'll find:

```
project-directory/
├── orchy.json              # Project configuration and tasks
├── GEMINI.md              # AI context for Gemini model
├── CLAUDE.md              # AI context for Claude model
├── package.json           # Frontend dependencies (if applicable)
├── Cargo.toml            # Rust dependencies (if applicable)
├── src/                  # Source code
├── tests/                # Test files
└── ...                   # Additional project files
```

## 🧪 Testing Generated Applications

### Vue.js Projects

```bash
cd your-vue-project
npm install
npm run dev          # Start development server
npm run test         # Run tests
npm run build        # Build for production
```

### Rust Projects

```bash
cd your-rust-project
cargo run            # Run the application
cargo test           # Run tests
cargo build --release  # Build for production
```

### Full-Stack Projects

```bash
# Start backend (from backend directory)
cd backend
cargo run

# Start frontend (from frontend directory)  
cd frontend
npm install
npm run dev
```

## 🔍 Verification and Testing

Orchy includes comprehensive testing tools:

### Run File Creation Verification

```bash
cargo run --bin verify_file_creation
```

### Run App Startup Verification

```bash
cargo test --test app_startup_verification -- --nocapture
```

### Run All Tests

```bash
cargo test
```

## 🎨 Generated Project Features

### Vue.js Projects Include:
- ✅ Vue 3 with Composition API
- ✅ Pinia for state management
- ✅ Vue Router for navigation
- ✅ Tailwind CSS for styling
- ✅ Vitest for testing
- ✅ Component-based architecture
- ✅ TypeScript support (when applicable)

### Rust Projects Include:
- ✅ Actix-web framework
- ✅ Async/await support
- ✅ Serde for serialization
- ✅ Comprehensive error handling
- ✅ Database integration patterns
- ✅ API documentation
- ✅ Unit and integration tests

## 🐛 Troubleshooting

### Common Issues

1. **"GEMINI_API_KEY not set"**
   ```bash
   export GEMINI_API_KEY="your-api-key"
   ```

2. **"Project creation failed"**
   - Check your API key is valid
   - Ensure you have internet connectivity
   - Verify the path is writable

3. **"JSON parsing error"**
   - This typically indicates an AI response issue
   - Try running the command again
   - Check your API quotas

4. **"Build failed" for generated projects**
   - Run `npm install` for frontend projects
   - Run `cargo build` for Rust projects
   - Check that all dependencies are available

### Debug Mode

Enable detailed logging:

```bash
RUST_LOG=debug ./target/release/orchy create [options]
```

## 📊 Performance

- **Project Generation**: 2-10 minutes depending on complexity
- **Task Execution**: Parallel where possible with dependency resolution
- **File Creation**: Optimized batch operations
- **AI Calls**: Efficient prompt engineering with context management

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

## 📝 Configuration

### Custom Agents

Place custom agent definitions in `agents/` directory:

```json
{
  "name": "CustomAgent",
  "description": "Specialized agent for custom tasks",
  "capabilities": ["custom-skill-1", "custom-skill-2"],
  "tech_stack": ["rust", "vue"]
}
```

### Environment Variables

- `GEMINI_API_KEY` - Required for AI functionality
- `ORCHY_AGENTS_DIR` - Custom agents directory
- `RUST_LOG` - Logging level (debug, info, warn, error)

## 🔮 Roadmap

- [ ] Claude AI integration
- [ ] Docker deployment automation
- [ ] CI/CD pipeline generation
- [ ] More technology stacks (Python, Go, etc.)
- [ ] Visual project dashboard
- [ ] Real-time collaboration features

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- Built with Rust and modern async programming
- Powered by Google's Gemini AI
- Inspired by modern development workflows and automation

---

## 🆘 Support

If you encounter issues:

1. Check the troubleshooting section above
2. Review the debug logs with `RUST_LOG=debug`
3. Ensure all prerequisites are installed
4. Verify your API keys and network connectivity

For more help, please check the documentation or create an issue in the repository.