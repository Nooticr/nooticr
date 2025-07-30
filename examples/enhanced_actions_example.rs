use orchy::enums::Action;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Enhanced Action System Example ===\n");

    // Example JSON with new actions that an LLM might generate
    let enhanced_actions_json = r#"[
        {
            "CreateDirectory": {
                "path": "temp_demo"
            }
        },
        {
            "Write": {
                "path": "temp_demo/sample.txt",
                "content": "Hello, World!\nThis is a sample file.\nWith multiple lines."
            }
        },
        {
            "Write": {
                "path": "temp_demo/config.json",
                "content": "{\"name\": \"demo\", \"version\": \"1.0.0\"}"
            }
        },
        {
            "CreateDirectory": {
                "path": "temp_demo/subdirectory"
            }
        },
        {
            "Write": {
                "path": "temp_demo/subdirectory/nested.txt",
                "content": "This is a nested file."
            }
        },
        {
            "Backup": {
                "path": "temp_demo/sample.txt",
                "backup_suffix": ".backup"
            }
        },
        {
            "Append": {
                "path": "temp_demo/sample.txt",
                "content": "\nAppended line 1\nAppended line 2"
            }
        },
        {
            "ListDirectory": {
                "path": "temp_demo",
                "recursive": true
            }
        },
        {
            "Grep": {
                "pattern": "sample",
                "path": "temp_demo",
                "recursive": true,
                "case_sensitive": false
            }
        },
        {
            "Copy": {
                "old_path": "temp_demo/sample.txt",
                "new_path": "temp_demo/sample_copy.txt"
            }
        },
        {
            "SetPermissions": {
                "path": "temp_demo/sample.txt",
                "permissions": "644"
            }
        },
        {
            "Archive": {
                "source_paths": ["temp_demo/sample.txt", "temp_demo/config.json"],
                "archive_path": "temp_demo/archive.tar.gz",
                "format": "tar.gz"
            }
        },
        {
            "RemoveDirectory": {
                "path": "temp_demo",
                "recursive": true
            }
        }
    ]"#;

    println!("1. PARSING ENHANCED ACTIONS");
    println!("{}", "=".repeat(50));
    
    // Parse the actions from JSON
    let actions = Action::from_json_array(enhanced_actions_json)?;
    println!("Successfully parsed {} actions from JSON", actions.len());

    println!("\n2. EXECUTING ENHANCED ACTIONS");
    println!("{}", "=".repeat(50));

    // Execute all actions
    for (i, action) in actions.iter().enumerate() {
        println!("Executing action {}: {:?}", i + 1, action);
        match action.execute().await {
            Ok(()) => println!("✅ Action {} completed successfully", i + 1),
            Err(e) => println!("❌ Action {} failed: {}", i + 1, e),
        }
        println!();
    }

    println!("3. DEMONSTRATING INDIVIDUAL NEW ACTIONS");
    println!("{}", "=".repeat(50));

    // Create a test directory for individual demonstrations
    let demo_dir = Action::CreateDirectory {
        path: "action_demo".to_string(),
    };
    demo_dir.execute().await?;

    // Demonstrate Grep action
    let sample_file = Action::Write {
        path: "action_demo/search_me.txt".to_string(),
        content: "This is a SAMPLE file\nWith some sample content\nAnd SAMPLE patterns to find".to_string(),
    };
    sample_file.execute().await?;

    println!("Demonstrating Grep (case-insensitive search for 'sample'):");
    let grep_action = Action::Grep {
        pattern: "sample".to_string(),
        path: "action_demo/search_me.txt".to_string(),
        recursive: false,
        case_sensitive: false,
    };
    grep_action.execute().await?;

    // Demonstrate Directory Listing
    println!("\nDemonstrating Directory Listing:");
    let list_action = Action::ListDirectory {
        path: "action_demo".to_string(),
        recursive: false,
    };
    list_action.execute().await?;

    // Demonstrate Append
    println!("\nDemonstrating Append:");
    let append_action = Action::Append {
        path: "action_demo/search_me.txt".to_string(),
        content: "\nThis line was appended!".to_string(),
    };
    append_action.execute().await?;

    // Demonstrate Backup
    println!("\nDemonstrating Backup:");
    let backup_action = Action::Backup {
        path: "action_demo/search_me.txt".to_string(),
        backup_suffix: Some(".bak".to_string()),
    };
    backup_action.execute().await?;

    // Demonstrate Symlink (Unix only)
    #[cfg(unix)]
    {
        println!("\nDemonstrating Symlink creation:");
        let symlink_action = Action::CreateSymlink {
            target: "search_me.txt".to_string(),
            link_path: "action_demo/link_to_search.txt".to_string(),
        };
        symlink_action.execute().await?;
    }

    // Clean up
    println!("\nCleaning up demo directory:");
    let cleanup_action = Action::RemoveDirectory {
        path: "action_demo".to_string(),
        recursive: true,
    };
    cleanup_action.execute().await?;

    println!("\n4. BATCH EXECUTION EXAMPLE");
    println!("{}", "=".repeat(50));

    // Demonstrate batch execution with error handling
    let batch_actions = vec![
        Action::CreateDirectory {
            path: "batch_demo".to_string(),
        },
        Action::Write {
            path: "batch_demo/file1.txt".to_string(),
            content: "File 1 content".to_string(),
        },
        Action::Write {
            path: "batch_demo/file2.txt".to_string(),
            content: "File 2 content".to_string(),
        },
        Action::ListDirectory {
            path: "batch_demo".to_string(),
            recursive: false,
        },
        Action::RemoveDirectory {
            path: "batch_demo".to_string(),
            recursive: true,
        },
    ];

    println!("Executing batch of {} actions:", batch_actions.len());
    Action::execute_batch(&batch_actions).await?;
    println!("✅ All batch actions completed successfully!");

    println!("\n=== Enhanced Action System Demo Complete! ===");
    println!("\nNew actions available for LLMs:");
    println!("• Grep - Search for patterns in files");
    println!("• CreateDirectory - Create directories");
    println!("• RemoveDirectory - Remove directories");
    println!("• ListDirectory - List directory contents");
    println!("• CreateSymlink - Create symbolic links");
    println!("• SetPermissions - Set file permissions");
    println!("• Append - Append to files");
    println!("• Backup - Create file backups");
    println!("• Download - Download files from URLs");
    println!("• Extract - Extract archives");
    println!("• Archive - Create archives");
    println!("• Watch - Watch files for changes");

    Ok(())
}
