use orchy::models::code_review::*;
use orchy::prompts::Prompts;

fn main() {
    // Example of the enhanced code review functionality
    println!("=== Enhanced Code Review Example ===\n");

    // Example files to review
    let files_and_code = vec![
        (
            "src/auth.rs".to_string(),
            r#"
use std::collections::HashMap;

pub fn authenticate_user(username: &str, password: &str) -> bool {
    let query = format!("SELECT * FROM users WHERE username = '{}' AND password = '{}'", username, password);
    // Execute query...
    true
}

pub fn get_user_data(user_id: i32) -> Option<String> {
    let data = unsafe { 
        std::ptr::read(user_id as *const String)
    };
    Some(data)
}
"#.to_string(),
        ),
        (
            "src/utils.rs".to_string(),
            r#"
pub fn process_data(data: Vec<u8>) -> Result<String, Box<dyn std::error::Error>> {
    let result = String::from_utf8(data)?;
    Ok(result)
}

pub fn calculate_score(values: &[i32]) -> f64 {
    let sum: i32 = values.iter().sum();
    sum as f64 / values.len() as f64
}
"#.to_string(),
        ),
    ];

    // Generate the enhanced code review prompt
    let prompt = Prompts::code_review_user_prompt(
        &files_and_code,
        "Implement secure user authentication and data processing utilities",
        "This is a web application handling sensitive user data",
        "pr-123",
    );

    println!("Generated Enhanced Code Review Prompt:");
    println!("{}", prompt);

    // Example of what the AI would return (in JSON format)
    let example_response = r#"{
        "id": null,
        "pull_request_id": "pr-123",
        "approved": false,
        "overall_comment": "The code has critical security vulnerabilities that must be addressed before approval. The authentication function is vulnerable to SQL injection, and there's unsafe memory access in the user data retrieval.",
        "comments": [
            {
                "file_path": "src/auth.rs",
                "line_start": 5,
                "line_end": 5,
                "feedback_type": "Security",
                "severity": "Critical",
                "message": "SQL injection vulnerability: user input is directly concatenated into SQL query without sanitization",
                "suggested_change": "Use parameterized queries or an ORM like sqlx: 'SELECT * FROM users WHERE username = $1 AND password = $2'",
                "code_snippet": "let query = format!(\"SELECT * FROM users WHERE username = '{}' AND password = '{}'\", username, password);"
            },
            {
                "file_path": "src/auth.rs",
                "line_start": 11,
                "line_end": 13,
                "feedback_type": "Security",
                "severity": "Critical",
                "message": "Unsafe memory access: reading arbitrary memory location based on user_id",
                "suggested_change": "Use proper database queries or safe data structures instead of unsafe pointer operations",
                "code_snippet": "let data = unsafe { \n    std::ptr::read(user_id as *const String)\n};"
            },
            {
                "file_path": "src/utils.rs",
                "line_start": 8,
                "line_end": 10,
                "feedback_type": "Issue",
                "severity": "Major",
                "message": "Division by zero potential: if values slice is empty, this will panic",
                "suggested_change": "Add check for empty slice: if values.is_empty() { return 0.0; }",
                "code_snippet": "let sum: i32 = values.iter().sum();\nsum as f64 / values.len() as f64"
            },
            {
                "file_path": "src/utils.rs",
                "line_start": 2,
                "line_end": 4,
                "feedback_type": "Praise",
                "severity": "Info",
                "message": "Good use of proper error handling with Result type",
                "suggested_change": null,
                "code_snippet": "pub fn process_data(data: Vec<u8>) -> Result<String, Box<dyn std::error::Error>>"
            }
        ],
        "summary": {
            "total_files_reviewed": 2,
            "total_lines_reviewed": 15,
            "issues_found": 3,
            "suggestions_made": 1,
            "security_concerns": 2,
            "performance_concerns": 0,
            "test_coverage_adequate": false,
            "overall_quality_score": 25
        }
    }"#;

    println!("\n=== Example AI Response ===");
    println!("{}", example_response);

    // Parse the response into a CodeReview object
    match CodeReview::from_json(example_response, "CodeReviewAgent") {
        Ok(review) => {
            println!("\n=== Parsed Code Review ===");
            println!("Pull Request: {}", review.pull_request_id);
            println!("Reviewer: {}", review.reviewer);
            println!("Approved: {}", review.approved);
            println!("Overall Comment: {}", review.overall_comment);
            println!("Quality Score: {}/100", review.summary.overall_quality_score);
            
            println!("\n=== Review Comments ===");
            for (i, comment) in review.comments.iter().enumerate() {
                println!("{}. [{}] {} ({}:{})", 
                    i + 1,
                    format!("{:?}", comment.severity),
                    comment.message,
                    comment.file_path,
                    comment.line_start.map_or("N/A".to_string(), |l| l.to_string())
                );
            }

            println!("\n=== Security Analysis ===");
            let security_comments = review.get_comments_by_type(ReviewFeedbackType::Security);
            println!("Security concerns found: {}", security_comments.len());
            
            let blocking_comments = review.get_blocking_comments();
            println!("Blocking issues: {}", blocking_comments.len());
            
            if review.has_unresolved_blocking_comments() {
                println!("❌ This PR cannot be merged due to unresolved blocking issues");
            } else {
                println!("✅ No blocking issues found");
            }
        }
        Err(e) => {
            println!("Failed to parse code review: {}", e);
        }
    }
}
