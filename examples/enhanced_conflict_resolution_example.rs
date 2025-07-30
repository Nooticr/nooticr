use orchy::models::conflict_resolution::*;
use orchy::prompts::Prompts;

fn main() {
    println!("=== Enhanced Conflict Resolution Example ===\n");

    // Example conflict data: (file_path, our_content, their_content, base_content)
    let conflicts_data = vec![
        (
            "src/auth.rs".to_string(),
            r#"
// Our version - added OAuth support
use oauth2::{Client, TokenResponse};

pub struct AuthService {
    oauth_client: Client,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(oauth_client: Client, jwt_secret: String) -> Self {
        Self { oauth_client, jwt_secret }
    }

    pub fn authenticate(&self, username: &str, password: &str) -> Result<String, AuthError> {
        // OAuth authentication logic
        let token = self.oauth_client.authenticate(username, password)?;
        Ok(self.generate_jwt(token))
    }

    fn generate_jwt(&self, token: String) -> String {
        // JWT generation logic
        format!("jwt_{}", token)
    }
}
"#.to_string(),
            r#"
// Their version - added 2FA support
use totp_rs::{Algorithm, TOTP};

pub struct AuthService {
    totp: TOTP,
    session_store: SessionStore,
}

impl AuthService {
    pub fn new(totp: TOTP, session_store: SessionStore) -> Self {
        Self { totp, session_store }
    }

    pub fn authenticate(&self, username: &str, password: &str, totp_code: Option<String>) -> Result<String, AuthError> {
        // Basic authentication
        if !self.verify_credentials(username, password) {
            return Err(AuthError::InvalidCredentials);
        }

        // 2FA verification
        if let Some(code) = totp_code {
            if !self.totp.check_current(&code)? {
                return Err(AuthError::Invalid2FA);
            }
        }

        Ok(self.create_session(username))
    }

    fn verify_credentials(&self, username: &str, password: &str) -> bool {
        // Credential verification logic
        true
    }

    fn create_session(&self, username: &str) -> String {
        // Session creation logic
        format!("session_{}", username)
    }
}
"#.to_string(),
            r#"
// Base version - simple authentication
pub struct AuthService {
    database: Database,
}

impl AuthService {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn authenticate(&self, username: &str, password: &str) -> Result<String, AuthError> {
        // Simple database authentication
        if self.database.verify_user(username, password) {
            Ok(format!("token_{}", username))
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }
}
"#.to_string(),
        ),
        (
            "Cargo.toml".to_string(),
            r#"
[package]
name = "auth-service"
version = "0.1.0"

[dependencies]
oauth2 = "4.0"
jsonwebtoken = "8.0"
serde = { version = "1.0", features = ["derive"] }
"#.to_string(),
            r#"
[package]
name = "auth-service"
version = "0.1.0"

[dependencies]
totp-rs = "5.0"
uuid = "1.0"
serde = { version = "1.0", features = ["derive"] }
"#.to_string(),
            r#"
[package]
name = "auth-service"
version = "0.1.0"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
"#.to_string(),
        ),
    ];

    // Generate the enhanced conflict resolution prompt
    let prompt = Prompts::conflict_resolution_user_prompt(
        &conflicts_data,
        "feature/oauth-integration -> main",
        "Merging OAuth integration feature with 2FA security improvements",
        "Merge OAuth and 2FA authentication systems",
    );

    println!("Generated Enhanced Conflict Resolution Prompt:");
    println!("{}", prompt);

    // Example of what the AI would return (in JSON format)
    let example_response = r#"{
        "id": null,
        "merge_commit_message": "Merge OAuth and 2FA authentication systems",
        "branch_info": {
            "source_branch": "feature/oauth-integration",
            "target_branch": "main",
            "source_commit": "abc123",
            "target_commit": "def456",
            "merge_base": "xyz789",
            "source_author": "oauth-dev@company.com",
            "target_author": "security-dev@company.com"
        },
        "conflicts": [
            {
                "file_path": "src/auth.rs",
                "conflict_type": "ContentConflict",
                "line_start": 1,
                "line_end": 50,
                "resolution_strategy": "ManualMerge",
                "resolved_content": "// Merged version - OAuth + 2FA support\nuse oauth2::{Client, TokenResponse};\nuse totp_rs::{Algorithm, TOTP};\n\npub struct AuthService {\n    oauth_client: Client,\n    totp: TOTP,\n    jwt_secret: String,\n    session_store: SessionStore,\n}\n\nimpl AuthService {\n    pub fn new(oauth_client: Client, totp: TOTP, jwt_secret: String, session_store: SessionStore) -> Self {\n        Self { oauth_client, totp, jwt_secret, session_store }\n    }\n\n    pub fn authenticate(&self, username: &str, password: &str, totp_code: Option<String>) -> Result<String, AuthError> {\n        // OAuth authentication with 2FA\n        let oauth_token = self.oauth_client.authenticate(username, password)?;\n        \n        // 2FA verification if code provided\n        if let Some(code) = totp_code {\n            if !self.totp.check_current(&code)? {\n                return Err(AuthError::Invalid2FA);\n            }\n        }\n        \n        let jwt = self.generate_jwt(oauth_token);\n        let session = self.create_session(username);\n        Ok(format!(\"{};{}\", jwt, session))\n    }\n\n    fn generate_jwt(&self, token: String) -> String {\n        format!(\"jwt_{}\", token)\n    }\n\n    fn create_session(&self, username: &str) -> String {\n        format!(\"session_{}\", username)\n    }\n}",
                "explanation": "Merged OAuth and 2FA authentication systems by combining both approaches. The new AuthService supports both OAuth tokens and 2FA verification, providing enhanced security while maintaining OAuth integration.",
                "confidence_score": 85,
                "requires_testing": true
            },
            {
                "file_path": "Cargo.toml",
                "conflict_type": "ContentConflict",
                "line_start": 5,
                "line_end": 10,
                "resolution_strategy": "ManualMerge",
                "resolved_content": "[package]\nname = \"auth-service\"\nversion = \"0.1.0\"\n\n[dependencies]\noauth2 = \"4.0\"\njsonwebtoken = \"8.0\"\ntotp-rs = \"5.0\"\nuuid = \"1.0\"\nserde = { version = \"1.0\", features = [\"derive\"] }",
                "explanation": "Combined dependencies from both branches to support both OAuth and 2FA functionality. All dependencies are compatible and necessary for the merged authentication system.",
                "confidence_score": 95,
                "requires_testing": true
            }
        ],
        "summary": {
            "total_files_with_conflicts": 2,
            "total_conflicts_resolved": 2,
            "conflicts_by_type": {"ContentConflict": 2},
            "resolution_strategies_used": {"ManualMerge": 2},
            "high_risk_resolutions": 0,
            "requires_manual_review": false,
            "estimated_test_time_minutes": 45,
            "overall_confidence_score": 90
        },
        "post_resolution_actions": [
            "cargo check",
            "cargo test",
            "cargo clippy",
            "git add .",
            "git commit -m 'Resolve merge conflicts: integrate OAuth and 2FA systems'"
        ]
    }"#;

    println!("\n=== Example AI Response ===");
    println!("{}", example_response);

    // Parse the response into a ConflictResolution object
    match ConflictResolution::from_json(example_response, "ConflictResolverAgent") {
        Ok(resolution) => {
            println!("\n=== Parsed Conflict Resolution ===");
            println!("Merge Message: {}", resolution.merge_commit_message);
            println!("Resolver: {}", resolution.resolver);
            println!("Source Branch: {} -> Target Branch: {}", 
                resolution.branch_info.source_branch, 
                resolution.branch_info.target_branch
            );
            println!("Overall Confidence: {}/100", resolution.summary.overall_confidence_score);
            
            println!("\n=== Conflict Details ===");
            for (i, conflict) in resolution.conflicts.iter().enumerate() {
                println!("{}. {} ({})", 
                    i + 1,
                    conflict.file_path,
                    format!("{:?}", conflict.conflict_type)
                );
                println!("   Strategy: {:?}", conflict.resolution_strategy);
                println!("   Confidence: {}/100", conflict.confidence_score);
                println!("   Testing Required: {}", conflict.requires_testing);
                println!("   Explanation: {}", conflict.explanation);
                println!();
            }

            println!("=== Resolution Analysis ===");
            let content_conflicts = resolution.get_conflicts_by_type(ConflictType::ContentConflict);
            println!("Content conflicts: {}", content_conflicts.len());
            
            let high_risk = resolution.get_high_risk_conflicts();
            println!("High-risk resolutions: {}", high_risk.len());
            
            let needs_testing = resolution.get_conflicts_requiring_testing();
            println!("Conflicts requiring testing: {}", needs_testing.len());
            
            if resolution.requires_manual_review() {
                println!("⚠️  Manual review recommended");
            } else {
                println!("✅ Resolution looks good for automated merge");
            }

            println!("\n=== Post-Resolution Actions ===");
            for (i, action) in resolution.post_resolution_actions.iter().enumerate() {
                println!("{}. {}", i + 1, action);
            }
        }
        Err(e) => {
            println!("Failed to parse conflict resolution: {}", e);
        }
    }
}
