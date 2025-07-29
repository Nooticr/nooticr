pub struct Prompts {}

impl Prompts {
    pub fn idea_breakdown_user_prompt(idea: &str, context: &str) -> String {
        format!(
            r#"Break down this software development idea into specific, well-structured tasks:

            IDEA: {}

            ADDITIONAL CONTEXT:
            {}

            TASK BREAKDOWN REQUIREMENTS:
            - Create 6-12 actionable tasks with clear separation of concerns
            - Start with "Research and Planning" for architecture and requirements analysis
            - Separate frontend, backend, and database tasks clearly
            - Each task should be specific, measurable, and technology-focused
            - Include proper dependency ordering (planning → backend → frontend → integration)
            - Consider authentication, data persistence, API design, and user interface
            - Include testing and documentation as separate tasks
            - Be specific about technology stack in descriptions (React/TypeScript for frontend, Rust/Node.js for backend, PostgreSQL for database)

            EXAMPLE QUALITY BREAKDOWN FOR TODO APP:
            1. Research and Planning (High priority, complexity 3)
            2. Database Schema Design (High priority, complexity 4)
            3. Backend API Development (High priority, complexity 6)
            4. Authentication System (High priority, complexity 5)
            5. Frontend Components (Medium priority, complexity 5)
            6. User Interface Integration (Medium priority, complexity 4)
            7. Testing and Quality Assurance (Medium priority, complexity 4)
            8. Documentation and Deployment (Low priority, complexity 3)

            Ensure each task has clear acceptance criteria and appropriate technology tags. return everythin in a json
            in this format (important)

             [{
                "id": "unique_task_id",
                "title": "Task title",
                "description": "Task description",
                "priority": "High/Medium/Low/Critical",
                "complexity": 1-10,
                "tags": ["tag1", "tag2", "tag3"]
                "depends_on": ["task_id1", "task_id2"]
             }]
            "#,
            idea, context
        )
    }

    pub fn feature_development_user_prompt(
        task_description: &str,
        codebase_context: &str,
        tech_stack: &str,
    ) -> String {
        format!(
            r#"Implement this feature based on the requirements and codebase context:

                    TASK: {}

                    CODEBASE CONTEXT:
                    {}

                    TECHNOLOGY STACK: {}

                    Requirements:
                    1. Follow existing code patterns and conventions
                    2. Implement complete functionality with error handling
                    3. Include appropriate tests if needed
                    4. Add meaningful comments for complex logic
                    5. Ensure code is production-ready

                    Please provide complete implementation with file paths."#,
            task_description, codebase_context, tech_stack
        )
    }

    pub fn code_review_user_prompt(code: &str, requirements: &str, context: &str) -> String {
        format!(
            r#"Review this code implementation against the requirements:

            CODE:
            {}

            REQUIREMENTS:
            {}

            CONTEXT:
            {}

            Please provide a thorough review focusing on:
            1. Does the code meet the stated requirements?
            2. Are there any bugs or edge cases missed?
            3. Is the code maintainable and well-structured?
            4. Are there security or performance concerns?
            5. Is error handling appropriate?
            6. Would you approve this code for production?"#,
            code, requirements, context
        )
    }

    pub fn conflict_resolution_user_prompt(
        conflict_content: &str,
        context: &str,
        branch_info: &str,
    ) -> String {
        format!(
            r#"Resolve this Git merge conflict:

                CONFLICT CONTENT:
                {}

                BRANCH CONTEXT:
                {}

                ADDITIONAL INFO:
                {}

                Requirements:
                1. Preserve functionality from both branches where possible
                2. Maintain consistent code style
                3. Ensure the result compiles and works correctly
                4. Remove all conflict markers (<<<<<<< ======= >>>>>>>)
                5. Provide clean, production-ready code

                Please provide the complete resolved file content."#,
            conflict_content, context, branch_info
        )
    }
}
