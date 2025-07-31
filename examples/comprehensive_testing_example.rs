/// Comprehensive Testing Agents Example
/// 
/// This example demonstrates all the testing agents:
/// - Unit Testing Agent
/// - Integration Testing Agent  
/// - E2E Testing Agent
/// - Performance Testing Agent
/// 
/// Each agent produces JSON actions to create, run, and fix tests.

use orchy::prompts::Prompts;

fn main() {
    println!("=== COMPREHENSIVE TESTING AGENTS ===\n");

    // Example 1: Unit Testing Agent
    example_unit_testing();
    
    // Example 2: Integration Testing Agent
    example_integration_testing();
    
    // Example 3: E2E Testing Agent
    example_e2e_testing();
    
    // Example 4: Performance Testing Agent
    example_performance_testing();
    
    println!("\n=== TESTING WORKFLOW SUMMARY ===");
    println!("🧪 Unit Testing: Tests individual functions and components");
    println!("🔗 Integration Testing: Tests component interactions and data flow");
    println!("🖱️  E2E Testing: Tests complete user workflows");
    println!("⚡ Performance Testing: Tests speed, load, and resource usage");
    println!("\n🎯 ALL TESTING AGENTS PRODUCE EXECUTABLE JSON ACTIONS!");
}

fn example_unit_testing() {
    println!("🧪 UNIT TESTING AGENT");
    println!("{}", "=".repeat(50));
    
    let tech_stack = "Vue 3, TypeScript, Vitest";
    let test_framework = "Vitest";
    
    let target_files = vec![
        ("src/utils/validation.ts".to_string(),
         r#"export function validateEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

export function validatePassword(password: string): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  
  if (password.length < 8) {
    errors.push('Password must be at least 8 characters');
  }
  
  if (!/[A-Z]/.test(password)) {
    errors.push('Password must contain at least one uppercase letter');
  }
  
  if (!/[0-9]/.test(password)) {
    errors.push('Password must contain at least one number');
  }
  
  return { valid: errors.length === 0, errors };
}

export function formatCurrency(amount: number, currency = 'USD'): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: currency,
  }).format(amount);
}"#.to_string()),
        ("src/components/TodoItem.vue".to_string(),
         r#"<template>
  <div class="todo-item" :class="{ completed: todo.completed }">
    <input 
      type="checkbox" 
      :checked="todo.completed" 
      @change="$emit('toggle', todo.id)"
    />
    <span class="todo-title">{{ todo.title }}</span>
    <button @click="$emit('delete', todo.id)" class="delete-btn">Delete</button>
  </div>
</template>

<script setup lang="ts">
interface Todo {
  id: string;
  title: string;
  completed: boolean;
}

defineProps<{
  todo: Todo;
}>();

defineEmits<{
  toggle: [id: string];
  delete: [id: string];
}>();
</script>"#.to_string()),
    ];
    
    let existing_tests = vec![];
    
    let prompt = Prompts::unit_testing_prompt(
        tech_stack,
        &target_files,
        test_framework,
        &existing_tests,
        None
    );
    
    println!("{}", prompt);
    println!("\n");
}

fn example_integration_testing() {
    println!("🔗 INTEGRATION TESTING AGENT");
    println!("{}", "=".repeat(50));
    
    let tech_stack = "Node.js, Express, PostgreSQL, Jest";
    let test_framework = "Jest with Supertest";
    
    let application_files = vec![
        ("src/routes/todos.js".to_string(),
         r#"const express = require('express');
const { TodoService } = require('../services/TodoService');
const router = express.Router();

router.get('/', async (req, res) => {
  try {
    const todos = await TodoService.getAll();
    res.json(todos);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

router.post('/', async (req, res) => {
  try {
    const { title } = req.body;
    if (!title) {
      return res.status(400).json({ error: 'Title is required' });
    }
    const todo = await TodoService.create({ title, completed: false });
    res.status(201).json(todo);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

module.exports = router;"#.to_string()),
        ("src/services/TodoService.js".to_string(),
         r#"const { TodoRepository } = require('../repositories/TodoRepository');

class TodoService {
  static async getAll() {
    return await TodoRepository.findAll();
  }
  
  static async create(todoData) {
    if (!todoData.title || todoData.title.trim() === '') {
      throw new Error('Title cannot be empty');
    }
    return await TodoRepository.create(todoData);
  }
  
  static async update(id, updates) {
    const todo = await TodoRepository.findById(id);
    if (!todo) {
      throw new Error('Todo not found');
    }
    return await TodoRepository.update(id, updates);
  }
}

module.exports = { TodoService };"#.to_string()),
    ];
    
    let integration_scenarios = vec![
        "Test API endpoints with database operations".to_string(),
        "Test authentication middleware integration".to_string(),
        "Test error handling across service layers".to_string(),
        "Test transaction rollback on failures".to_string(),
    ];
    
    let prompt = Prompts::integration_testing_prompt(
        tech_stack,
        &application_files,
        test_framework,
        &integration_scenarios,
        None
    );
    
    println!("{}", prompt);
    println!("\n");
}

fn example_e2e_testing() {
    println!("🖱️  E2E TESTING AGENT");
    println!("{}", "=".repeat(50));
    
    let tech_stack = "Vue 3, Vite, Playwright";
    let application_url = "http://localhost:3000";
    let test_framework = "Playwright";
    
    let user_workflows = vec![
        "User can register a new account".to_string(),
        "User can login with valid credentials".to_string(),
        "User can add, edit, and delete todos".to_string(),
        "User can filter todos by status (all, active, completed)".to_string(),
        "User can mark todos as completed/uncompleted".to_string(),
        "User can logout and session is cleared".to_string(),
    ];
    
    let prompt = Prompts::e2e_testing_prompt(
        tech_stack,
        application_url,
        &user_workflows,
        test_framework,
        None
    );
    
    println!("{}", prompt);
    println!("\n");
}

fn example_performance_testing() {
    println!("⚡ PERFORMANCE TESTING AGENT");
    println!("{}", "=".repeat(50));
    
    let tech_stack = "Vue 3, Node.js, PostgreSQL";
    let application_url = "http://localhost:3000";
    
    let performance_targets = vec![
        ("Page Load Time".to_string(), "< 2 seconds".to_string()),
        ("API Response Time".to_string(), "< 200ms".to_string()),
        ("Lighthouse Performance Score".to_string(), "> 90".to_string()),
        ("Concurrent Users".to_string(), "100 users".to_string()),
        ("Error Rate".to_string(), "< 1%".to_string()),
        ("Memory Usage".to_string(), "< 512MB".to_string()),
    ];
    
    let test_scenarios = vec![
        "Normal load: 10 concurrent users for 5 minutes".to_string(),
        "Peak load: 50 concurrent users for 2 minutes".to_string(),
        "Stress test: 100 concurrent users for 1 minute".to_string(),
        "Spike test: Sudden increase from 10 to 100 users".to_string(),
        "Endurance test: 20 users for 30 minutes".to_string(),
    ];
    
    let prompt = Prompts::performance_testing_prompt(
        tech_stack,
        application_url,
        &performance_targets,
        &test_scenarios,
        None
    );
    
    println!("{}", prompt);
    println!("\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_testing_prompt() {
        let tech_stack = "Vue 3, TypeScript";
        let test_framework = "Vitest";
        let target_files = vec![];
        let existing_tests = vec![];
        
        let prompt = Prompts::unit_testing_prompt(tech_stack, &target_files, test_framework, &existing_tests, None);
        
        assert!(prompt.contains("UnitTesting agent"));
        assert!(prompt.contains("MUST RETURN JSON ACTIONS"));
        assert!(prompt.contains("UNIT TESTING ACTION EXAMPLES"));
        assert!(prompt.contains("Function Testing"));
        assert!(prompt.contains("Edge Cases"));
    }

    #[test]
    fn test_integration_testing_prompt() {
        let tech_stack = "Node.js, Express";
        let test_framework = "Jest";
        let app_files = vec![];
        let scenarios = vec!["API testing".to_string()];
        
        let prompt = Prompts::integration_testing_prompt(tech_stack, &app_files, test_framework, &scenarios, None);
        
        assert!(prompt.contains("IntegrationTesting agent"));
        assert!(prompt.contains("MUST RETURN JSON ACTIONS"));
        assert!(prompt.contains("INTEGRATION TESTING ACTION EXAMPLES"));
        assert!(prompt.contains("API Testing"));
        assert!(prompt.contains("Database Testing"));
    }

    #[test]
    fn test_e2e_testing_prompt() {
        let tech_stack = "Vue 3";
        let app_url = "http://localhost:3000";
        let test_framework = "Playwright";
        let workflows = vec!["User login".to_string()];
        
        let prompt = Prompts::e2e_testing_prompt(tech_stack, app_url, &workflows, test_framework, None);
        
        assert!(prompt.contains("E2ETesting agent"));
        assert!(prompt.contains("MUST RETURN JSON ACTIONS"));
        assert!(prompt.contains("E2E TESTING ACTION EXAMPLES"));
        assert!(prompt.contains("User Authentication"));
        assert!(prompt.contains("Navigation"));
    }

    #[test]
    fn test_performance_testing_prompt() {
        let tech_stack = "Vue 3";
        let app_url = "http://localhost:3000";
        let targets = vec![("Load Time".to_string(), "< 2s".to_string())];
        let scenarios = vec!["Load test".to_string()];
        
        let prompt = Prompts::performance_testing_prompt(tech_stack, app_url, &targets, &scenarios, None);
        
        assert!(prompt.contains("PerformanceTesting agent"));
        assert!(prompt.contains("MUST RETURN JSON ACTIONS"));
        assert!(prompt.contains("PERFORMANCE TESTING ACTION EXAMPLES"));
        assert!(prompt.contains("Load Testing"));
        assert!(prompt.contains("Response Time"));
    }
}
