use orchy::prompts::Prompts;

fn main() {
    println!("=== Enhanced Software Development Prompts Example ===\n");

    // 1. Enhanced Idea Breakdown
    println!("1. ENHANCED IDEA BREAKDOWN PROMPT");
    println!("{}", "=".repeat(50));
    
    let idea_prompt = Prompts::idea_breakdown_user_prompt(
        "Build a real-time collaborative document editor like Google Docs",
        "This will be used by teams for collaborative writing and editing with real-time synchronization",
        vec![
            "BackendEngineerRust".to_string(),
            "FrontendEngineerReact".to_string(),
            "DevOpsEngineer".to_string(),
            "DatabaseEngineer".to_string(),
            "SecurityEngineer".to_string(),
        ],
        "Rust backend with Actix-web, React frontend with TypeScript, PostgreSQL database, Redis for caching, WebSocket for real-time updates"
    );
    
    println!("{}\n", idea_prompt);

    // 2. Enhanced Feature Development
    println!("2. ENHANCED FEATURE DEVELOPMENT PROMPT");
    println!("{}", "=".repeat(50));
    
    let existing_files = vec![
        ("src/models/user.rs".to_string(), r#"
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
"#.to_string()),
        ("src/routes/auth.rs".to_string(), r#"
pub async fn login(req: HttpRequest) -> Result<HttpResponse, Error> {
    // Basic login implementation
    Ok(HttpResponse::Ok().json("Login successful"))
}
"#.to_string()),
    ];

    let feature_prompt = Prompts::feature_development_user_prompt(
        "Implement JWT-based authentication with refresh tokens",
        "Existing user management system with basic login/logout functionality",
        "Rust with Actix-web, PostgreSQL, Redis",
        &existing_files,
        "Implement secure JWT authentication with access tokens (15min expiry) and refresh tokens (7 days expiry). Include proper token validation, refresh mechanism, and logout functionality.",
        &[
            "User can login with email/password and receive JWT tokens".to_string(),
            "Access tokens expire after 15 minutes".to_string(),
            "Refresh tokens expire after 7 days".to_string(),
            "User can refresh access token using refresh token".to_string(),
            "User can logout and invalidate all tokens".to_string(),
            "All API endpoints validate JWT tokens".to_string(),
        ]
    );
    
    println!("{}\n", feature_prompt);

    // 3. CI/CD Fix Prompt
    println!("3. CI/CD FIX PROMPT");
    println!("{}", "=".repeat(50));
    
    let cicd_prompt = Prompts::ci_cd_fix_user_prompt(
        r#"
name: CI/CD Pipeline
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test
"#,
        r#"
Error: failed to run custom build command for `openssl-sys v0.9.72`
  --- stderr
  thread 'main' panicked at 'Unable to find libssl-dev'
Build failed with exit code 101
"#,
        "Rust web application with OpenSSL dependencies for HTTPS and JWT token handling",
        "Rust, Actix-web, PostgreSQL, Redis"
    );
    
    println!("{}\n", cicd_prompt);

    // 4. Docker Deployment Prompt
    println!("4. DOCKER DEPLOYMENT PROMPT");
    println!("{}", "=".repeat(50));
    
    let docker_prompt = Prompts::docker_deployment_user_prompt(
        "Rust web API with PostgreSQL database and Redis cache, serving a React frontend",
        "Deploy to production Kubernetes cluster with high availability, auto-scaling, and monitoring",
        "Rust backend, React frontend, PostgreSQL, Redis, Nginx",
        "production"
    );
    
    println!("{}\n", docker_prompt);

    // 5. QA Analysis Prompt
    println!("5. QA ANALYSIS PROMPT");
    println!("{}", "=".repeat(50));
    
    let qa_code = vec![
        ("src/auth.rs".to_string(), r#"
pub fn authenticate_user(token: &str) -> Result<User, AuthError> {
    if token.is_empty() {
        return Err(AuthError::InvalidToken);
    }
    // JWT validation logic
    Ok(User::default())
}
"#.to_string()),
    ];

    let qa_prompt = Prompts::qa_analysis_user_prompt(
        &qa_code,
        "Unit tests: 85% coverage, Integration tests: 70% coverage, 2 failing tests in auth module",
        "Secure user authentication with JWT tokens, password reset functionality, and session management",
        &[
            "User logs in with valid credentials".to_string(),
            "User attempts login with invalid credentials".to_string(),
            "User resets password via email".to_string(),
            "User session expires and requires re-authentication".to_string(),
        ]
    );
    
    println!("{}\n", qa_prompt);

    // 6. API Synchronization Prompt
    println!("6. API SYNCHRONIZATION PROMPT");
    println!("{}", "=".repeat(50));
    
    let frontend_code = vec![
        ("src/api/auth.ts".to_string(), r#"
interface LoginRequest {
  email: string;
  password: string;
}

export const login = async (data: LoginRequest) => {
  const response = await fetch('/api/auth/login', {
    method: 'POST',
    body: JSON.stringify(data),
  });
  return response.json();
};
"#.to_string()),
    ];

    let api_sync_prompt = Prompts::api_synchronization_user_prompt(
        r#"
POST /api/auth/login
Request: { "email": "string", "password": "string" }
Response: { "access_token": "string", "refresh_token": "string", "user": { "id": "uuid", "email": "string", "name": "string" } }

GET /api/users/profile
Headers: Authorization: Bearer <token>
Response: { "id": "uuid", "email": "string", "name": "string", "created_at": "ISO8601" }
"#,
        &frontend_code,
        "JWT-based authentication API with user management endpoints",
        "TypeScript React frontend, Rust Actix-web backend"
    );
    
    println!("{}\n", api_sync_prompt);

    // 7. Performance Optimization Prompt
    println!("7. PERFORMANCE OPTIMIZATION PROMPT");
    println!("{}", "=".repeat(50));
    
    let perf_code = vec![
        ("src/handlers/users.rs".to_string(), r#"
pub async fn get_users() -> Result<HttpResponse, Error> {
    let users = sqlx::query!("SELECT * FROM users")
        .fetch_all(&pool)
        .await?;
    Ok(HttpResponse::Ok().json(users))
}
"#.to_string()),
    ];

    let perf_prompt = Prompts::performance_optimization_user_prompt(
        &perf_code,
        "Average response time: 800ms, CPU usage: 85%, Memory usage: 3.2GB, Database query time: 400ms",
        &[
            "Database queries without indexes".to_string(),
            "No caching layer implemented".to_string(),
            "Large JSON responses without pagination".to_string(),
            "Inefficient memory allocation in loops".to_string(),
        ],
        "Rust Actix-web, PostgreSQL, Redis"
    );
    
    println!("{}\n", perf_prompt);

    println!("=== All Enhanced Prompts Generated Successfully! ===");
}
