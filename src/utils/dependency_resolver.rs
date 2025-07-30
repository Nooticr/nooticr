use crate::models::task::Task;
use crate::error::{OrchestratorError, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;
use tracing::debug;

/// Utility for resolving task dependencies and determining execution order
pub struct DependencyResolver;

impl DependencyResolver {
    /// Sort tasks in dependency order using topological sorting
    /// Returns tasks in the order they should be executed
    pub fn sort_tasks_by_dependencies(tasks: Vec<Task>) -> Result<Vec<Task>> {
        debug!("🔍 Starting dependency resolution for {} tasks", tasks.len());
        
        // Create maps for easier lookup
        let mut task_map: HashMap<Uuid, Task> = HashMap::new();
        let mut dependency_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut dependents_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        
        // Build the maps
        debug!("📋 Building dependency maps...");
        for task in tasks {
            let task_id = task.id;
            debug!("   📝 Task: {} ({})", task.title, task_id);
            debug!("      - Dependencies: {:?}", task.depends_on);
            debug!("      - Priority: {:?}", task.priority);
            debug!("      - Complexity: {:?}", task.estimated_complexity);
            
            // Store dependencies for this task
            dependency_map.insert(task_id, task.depends_on.clone());
            
            // Build reverse dependency map (who depends on this task)
            for &dep_id in &task.depends_on {
                dependents_map.entry(dep_id).or_insert_with(Vec::new).push(task_id);
            }
            
            task_map.insert(task_id, task);
        }
        
        debug!("🔗 Dependency analysis:");
        for (task_id, deps) in &dependency_map {
            if let Some(task) = task_map.get(task_id) {
                if deps.is_empty() {
                    debug!("   🟢 {} has no dependencies (can start immediately)", task.title);
                } else {
                    debug!("   🔵 {} depends on {} tasks:", task.title, deps.len());
                    for dep_id in deps {
                        if let Some(dep_task) = task_map.get(dep_id) {
                            debug!("      ⬅️  {}", dep_task.title);
                        } else {
                            debug!("      ❌ Unknown dependency: {}", dep_id);
                        }
                    }
                }
            }
        }
        
        // Perform topological sort using Kahn's algorithm
        debug!("🔄 Performing topological sort...");
        let mut sorted_tasks = Vec::new();
        let mut in_degree: HashMap<Uuid, usize> = HashMap::new();
        let mut queue = VecDeque::new();
        
        // Calculate in-degree for each task
        debug!("📊 Calculating in-degrees...");
        for (&task_id, dependencies) in &dependency_map {
            let degree = dependencies.len();
            in_degree.insert(task_id, degree);
            debug!("   📈 {} has in-degree: {}", task_map.get(&task_id).unwrap().title, degree);
            
            if degree == 0 {
                queue.push_back(task_id);
                debug!("      ➡️  Added to initial queue (no dependencies)");
            }
        }
        
        debug!("🚀 Starting topological sort with {} tasks in initial queue", queue.len());
        let mut processing_round = 1;
        
        while let Some(current_id) = queue.pop_front() {
            let current_task = task_map.remove(&current_id).unwrap();
            debug!("🔄 Round {}: Processing task '{}'", processing_round, current_task.title);
            debug!("   📋 Task details:");
            debug!("      - ID: {}", current_task.id);
            debug!("      - Priority: {:?}", current_task.priority);
            debug!("      - Complexity: {:?}", current_task.estimated_complexity);
            debug!("      - Tags: {:?}", current_task.tags);
            
            sorted_tasks.push(current_task);
            
            // Check all tasks that depend on this one
            if let Some(dependent_ids) = dependents_map.get(&current_id) {
                debug!("   ⬇️  Updating {} dependent tasks:", dependent_ids.len());
                for &dependent_id in dependent_ids {
                    if let Some(degree) = in_degree.get_mut(&dependent_id) {
                        *degree -= 1;
                        let dependent_task = task_map.get(&dependent_id).unwrap();
                        debug!("      📉 {} in-degree reduced to {}", dependent_task.title, *degree);
                        
                        if *degree == 0 {
                            queue.push_back(dependent_id);
                            debug!("         ✅ All dependencies satisfied - added to queue");
                        } else {
                            debug!("         ⏳ Still waiting for {} dependencies", *degree);
                        }
                    }
                }
            } else {
                debug!("   ℹ️  No tasks depend on this one");
            }
            
            processing_round += 1;
        }
        
        // Check for circular dependencies
        if !task_map.is_empty() {
            debug!("❌ Circular dependency detected! Remaining tasks:");
            for (id, task) in &task_map {
                debug!("   🔄 {} (dependencies: {:?})", task.title, dependency_map.get(id).unwrap_or(&vec![]));
            }
            return Err(OrchestratorError::validation("Circular dependency detected in tasks"));
        }
        
        debug!("✅ Dependency resolution complete!");
        debug!("📋 Final execution order ({} tasks):", sorted_tasks.len());
        for (index, task) in sorted_tasks.iter().enumerate() {
            debug!("   {}. {} (Priority: {:?}, Complexity: {:?})", 
                   index + 1, task.title, task.priority, task.estimated_complexity);
        }
        
        Ok(sorted_tasks)
    }
    
    /// Check if all dependencies of a task are satisfied (completed)
    pub fn are_dependencies_satisfied(task: &Task, completed_tasks: &HashSet<Uuid>) -> bool {
        debug!("🔍 Checking dependencies for task: {}", task.title);
        debug!("   📋 Task has {} dependencies", task.depends_on.len());
        debug!("   ✅ {} tasks are completed", completed_tasks.len());
        
        for (index, &dep_id) in task.depends_on.iter().enumerate() {
            let is_completed = completed_tasks.contains(&dep_id);
            debug!("   {}. Dependency {} is {}", 
                   index + 1, dep_id, 
                   if is_completed { "✅ COMPLETED" } else { "⏳ PENDING" });
            
            if !is_completed {
                debug!("   ❌ Not all dependencies satisfied");
                return false;
            }
        }
        
        debug!("   ✅ All dependencies satisfied!");
        true
    }
    
    /// Get tasks that can be executed next (no pending dependencies)
    pub fn get_ready_tasks<'a>(tasks: &'a [Task], completed_tasks: &HashSet<Uuid>) -> Vec<&'a Task> {
        debug!("🔍 Finding ready tasks from {} total tasks", tasks.len());
        debug!("   ✅ {} tasks already completed", completed_tasks.len());
        
        let ready_tasks: Vec<&Task> = tasks
            .iter()
            .filter(|task| {
                let is_completed = completed_tasks.contains(&task.id);
                if is_completed {
                    return false;
                }
                Self::are_dependencies_satisfied(task, completed_tasks)
            })
            .collect();
        
        debug!("🎯 Found {} ready tasks:", ready_tasks.len());
        for (index, task) in ready_tasks.iter().enumerate() {
            debug!("   {}. {} (Priority: {:?})", index + 1, task.title, task.priority);
        }
        
        ready_tasks
    }
    
    /// Validate that all task dependencies exist in the task list
    pub fn validate_dependencies(tasks: &[Task]) -> Result<()> {
        debug!("🔍 Validating task dependencies...");
        let task_ids: HashSet<Uuid> = tasks.iter().map(|t| t.id).collect();
        
        for task in tasks {
            debug!("   📝 Validating task: {}", task.title);
            for &dep_id in &task.depends_on {
                if !task_ids.contains(&dep_id) {
                    debug!("   ❌ Invalid dependency: {} not found in task list", dep_id);
                    return Err(OrchestratorError::validation(&format!(
                        "Task '{}' depends on non-existent task: {}",
                        task.title, dep_id
                    )));
                } else {
                    debug!("   ✅ Dependency {} exists", dep_id);
                }
            }
        }
        
        debug!("✅ All task dependencies are valid");
        Ok(())
    }
    
    /// Get dependency depth for each task (for prioritization)
    pub fn calculate_dependency_depths(tasks: &[Task]) -> HashMap<Uuid, usize> {
        debug!("📊 Calculating dependency depths...");
        let mut depths = HashMap::new();
        let task_map: HashMap<Uuid, &Task> = tasks.iter().map(|t| (t.id, t)).collect();
        
        for task in tasks {
            let depth = Self::calculate_task_depth(task, &task_map, &mut HashMap::new());
            depths.insert(task.id, depth);
            debug!("   📏 Task '{}' has depth: {}", task.title, depth);
        }
        
        debug!("✅ Dependency depths calculated");
        depths
    }
    
    /// Recursively calculate depth of a single task
    fn calculate_task_depth(
        task: &Task, 
        task_map: &HashMap<Uuid, &Task>, 
        memo: &mut HashMap<Uuid, usize>
    ) -> usize {
        if let Some(&cached_depth) = memo.get(&task.id) {
            return cached_depth;
        }
        
        let depth = if task.depends_on.is_empty() {
            0
        } else {
            task.depends_on
                .iter()
                .filter_map(|&dep_id| task_map.get(&dep_id))
                .map(|dep_task| Self::calculate_task_depth(dep_task, task_map, memo))
                .max()
                .unwrap_or(0) + 1
        };
        
        memo.insert(task.id, depth);
        depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::task::{Task, TaskInput};
    use crate::enums::Priority;
    
    #[test]
    fn test_sort_tasks_simple_dependency() {
        let task1 = Task::new("Task 1", "First task", Priority::High);
        let task1_id = task1.id;
        
        let mut task2 = Task::new("Task 2", "Second task", Priority::Medium);
        task2.depends_on = vec![task1_id];
        
        let tasks = vec![task2.clone(), task1.clone()]; // Intentionally out of order
        let sorted = DependencyResolver::sort_tasks_by_dependencies(tasks).unwrap();
        
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].id, task1_id); // Task 1 should come first
        assert_eq!(sorted[1].id, task2.id); // Task 2 should come second
    }
    
    #[test]
    fn test_sort_tasks_complex_dependencies() {
        let task_a = Task::new("Task A", "Independent task", Priority::High);
        let task_b = Task::new("Task B", "Independent task", Priority::High);
        let task_a_id = task_a.id;
        let task_b_id = task_b.id;
        
        let mut task_c = Task::new("Task C", "Depends on A and B", Priority::Medium);
        task_c.depends_on = vec![task_a_id, task_b_id];
        
        let mut task_d = Task::new("Task D", "Depends on C", Priority::Low);
        task_d.depends_on = vec![task_c.id];
        
        let tasks = vec![task_d.clone(), task_c.clone(), task_b.clone(), task_a.clone()];
        let sorted = DependencyResolver::sort_tasks_by_dependencies(tasks).unwrap();
        
        assert_eq!(sorted.len(), 4);
        
        // A and B should come first (order doesn't matter between them)
        let first_two: HashSet<Uuid> = sorted[0..2].iter().map(|t| t.id).collect();
        assert!(first_two.contains(&task_a_id));
        assert!(first_two.contains(&task_b_id));
        
        // C should come third
        assert_eq!(sorted[2].id, task_c.id);
        
        // D should come last
        assert_eq!(sorted[3].id, task_d.id);
    }
    
    #[test]
    fn test_circular_dependency_detection() {
        let mut task1 = Task::new("Task 1", "First task", Priority::High);
        let mut task2 = Task::new("Task 2", "Second task", Priority::Medium);
        
        // Create circular dependency
        task1.depends_on = vec![task2.id];
        task2.depends_on = vec![task1.id];
        
        let tasks = vec![task1, task2];
        let result = DependencyResolver::sort_tasks_by_dependencies(tasks);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular dependency"));
    }
    
    #[test]
    fn test_validate_dependencies() {
        let task1 = Task::new("Task 1", "First task", Priority::High);
        let task1_id = task1.id;
        
        let mut task2 = Task::new("Task 2", "Second task", Priority::Medium);
        task2.depends_on = vec![task1_id];
        
        // Valid dependencies
        let tasks = vec![task1, task2];
        assert!(DependencyResolver::validate_dependencies(&tasks).is_ok());
        
        // Invalid dependency
        let task3 = Task::new("Task 3", "Third task", Priority::Low);
        let mut task4 = Task::new("Task 4", "Fourth task", Priority::Medium);
        task4.depends_on = vec![Uuid::new_v4()]; // Non-existent dependency
        
        let invalid_tasks = vec![task3, task4];
        assert!(DependencyResolver::validate_dependencies(&invalid_tasks).is_err());
    }
    
    #[test]
    fn test_get_ready_tasks() {
        let task1 = Task::new("Task 1", "Independent", Priority::High);
        let task2 = Task::new("Task 2", "Independent", Priority::High);
        let task1_id = task1.id;
        let task2_id = task2.id;
        
        let mut task3 = Task::new("Task 3", "Depends on 1", Priority::Medium);
        task3.depends_on = vec![task1_id];
        
        let mut task4 = Task::new("Task 4", "Depends on 1 and 2", Priority::Low);
        task4.depends_on = vec![task1_id, task2_id];
        
        let tasks = vec![task1, task2, task3, task4];
        
        // Initially, only independent tasks are ready
        let completed = HashSet::new();
        let ready = DependencyResolver::get_ready_tasks(&tasks, &completed);
        assert_eq!(ready.len(), 2);
        
        // After completing task1, task3 becomes ready
        let mut completed = HashSet::new();
        completed.insert(task1_id);
        let ready = DependencyResolver::get_ready_tasks(&tasks, &completed);
        assert_eq!(ready.len(), 2); // task2 and task3
        
        // After completing both task1 and task2, task4 becomes ready
        completed.insert(task2_id);
        let ready = DependencyResolver::get_ready_tasks(&tasks, &completed);
        assert_eq!(ready.len(), 2); // task2 and task3
    }
}