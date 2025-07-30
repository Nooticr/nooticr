use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use crate::models::task::TaskInput;
use crate::enums::Action;

/// Response structure for idea breakdown prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeaBreakdownResponse {
    pub tasks: Vec<TaskInput>,
}

/// Response structure for feature development prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDevelopmentResponse {
    pub actions: Vec<Action>,
}

/// Response structure for CI/CD fix prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiCdFixResponse {
    pub id: Option<Uuid>,
    pub issue_analysis: IssueAnalysis,
    pub immediate_fixes: Vec<ImmediateFix>,
    pub pipeline_improvements: Vec<PipelineImprovement>,
    pub testing_strategy: TestingStrategy,
    pub monitoring_setup: MonitoringSetup,
    pub post_fix_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueAnalysis {
    pub root_cause: String,
    pub failure_type: String, // Configuration/Dependency/Code/Environment/Security
    pub affected_stages: Vec<String>,
    pub severity: String, // Critical/High/Medium/Low
    pub estimated_fix_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmediateFix {
    pub file_path: String,
    pub change_type: String, // Update/Add/Remove
    pub description: String,
    pub content: String,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineImprovement {
    pub improvement_type: String, // Performance/Security/Reliability/Monitoring
    pub description: String,
    pub implementation: String,
    pub benefits: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingStrategy {
    pub unit_tests: String,
    pub integration_tests: String,
    pub security_tests: String,
    pub performance_tests: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSetup {
    pub metrics_to_track: Vec<String>,
    pub alerting_rules: Vec<String>,
    pub logging_configuration: String,
}

/// Response structure for Docker deployment prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerDeploymentResponse {
    pub id: Option<Uuid>,
    pub deployment_strategy: DeploymentStrategy,
    pub docker_files: Vec<DockerFile>,
    pub kubernetes_manifests: Vec<KubernetesManifest>,
    pub configuration_files: Vec<ConfigurationFile>,
    pub security_configuration: SecurityConfiguration,
    pub monitoring_setup: DockerMonitoringSetup,
    pub deployment_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStrategy {
    pub container_architecture: String, // Single/Multi-container/Microservices
    pub orchestration_platform: String, // Docker Compose/Kubernetes/Docker Swarm
    pub scaling_approach: String, // Horizontal/Vertical/Auto
    pub environment_specific_configs: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerFile {
    pub file_path: String,
    pub content: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesManifest {
    pub file_path: String,
    pub content: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationFile {
    pub file_path: String,
    pub content: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfiguration {
    pub secrets_management: String,
    pub network_policies: String,
    pub rbac_configuration: String,
    pub security_scanning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerMonitoringSetup {
    pub logging_configuration: String,
    pub metrics_collection: String,
    pub health_checks: String,
    pub alerting_rules: String,
}

/// Response structure for QA analysis prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaAnalysisResponse {
    pub id: Option<Uuid>,
    pub overall_quality_score: u8,
    pub functional_analysis: FunctionalAnalysis,
    pub non_functional_analysis: NonFunctionalAnalysis,
    pub test_coverage_analysis: TestCoverageAnalysis,
    pub quality_metrics: QualityMetrics,
    pub critical_issues: Vec<CriticalIssue>,
    pub recommendations: Vec<Recommendation>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalAnalysis {
    pub requirements_coverage: RequirementsCoverage,
    pub user_workflow_testing: Vec<UserWorkflowTest>,
    pub edge_cases: Vec<EdgeCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementsCoverage {
    pub covered_requirements: Vec<String>,
    pub missing_requirements: Vec<String>,
    pub coverage_percentage: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWorkflowTest {
    pub scenario: String,
    pub status: String, // Pass/Fail/Partial
    pub issues_found: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCase {
    pub case: String,
    pub tested: bool,
    pub result: String, // Pass/Fail
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonFunctionalAnalysis {
    pub performance: PerformanceAnalysis,
    pub security: SecurityAnalysis,
    pub usability: UsabilityAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    pub response_times: String,
    pub throughput: String,
    pub resource_usage: String,
    pub bottlenecks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysis {
    pub vulnerabilities_found: Vec<String>,
    pub security_score: u8,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsabilityAnalysis {
    pub accessibility_score: u8,
    pub user_experience_issues: Vec<String>,
    pub improvement_suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCoverageAnalysis {
    pub current_coverage: CurrentCoverage,
    pub coverage_gaps: Vec<CoverageGap>,
    pub recommended_test_cases: Vec<RecommendedTestCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentCoverage {
    pub unit_tests: u8,
    pub integration_tests: u8,
    pub e2e_tests: u8,
    pub overall_coverage: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageGap {
    pub area: String,
    pub current_coverage: u8,
    pub recommended_tests: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedTestCase {
    pub test_type: String, // Unit/Integration/E2E
    pub description: String,
    pub priority: String, // High/Medium/Low
    pub implementation_effort: String, // Low/Medium/High
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub code_quality: CodeQuality,
    pub documentation: Documentation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQuality {
    pub maintainability_score: u8,
    pub technical_debt: String, // Low/Medium/High
    pub code_smells: Vec<String>,
    pub refactoring_recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Documentation {
    pub completeness_score: u8,
    pub accuracy_score: u8,
    pub missing_documentation: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalIssue {
    pub severity: String, // Critical/High/Medium/Low
    pub category: String, // Functional/Security/Performance/Usability
    pub description: String,
    pub impact: String,
    pub recommended_fix: String,
    pub priority: String, // Immediate/High/Medium/Low
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub category: String, // Testing/Performance/Security/Usability
    pub recommendation: String,
    pub implementation_effort: String, // Low/Medium/High
    pub expected_impact: String,
    pub timeline: String,
}

/// Response structure for API synchronization prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSynchronizationResponse {
    pub id: Option<Uuid>,
    pub synchronization_analysis: SynchronizationAnalysis,
    pub generated_code: Vec<GeneratedCode>,
    pub integration_fixes: Vec<IntegrationFix>,
    pub api_client_configuration: ApiClientConfiguration,
    pub testing_strategy: ApiTestingStrategy,
    pub documentation_updates: Vec<DocumentationUpdate>,
    pub migration_plan: MigrationPlan,
    pub monitoring_setup: ApiMonitoringSetup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynchronizationAnalysis {
    pub api_coverage: ApiCoverage,
    pub data_type_mismatches: Vec<DataTypeMismatch>,
    pub authentication_sync: AuthenticationSync,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCoverage {
    pub total_backend_endpoints: u32,
    pub frontend_integrated_endpoints: u32,
    pub missing_integrations: Vec<String>,
    pub outdated_integrations: Vec<String>,
    pub coverage_percentage: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTypeMismatch {
    pub endpoint: String,
    pub field: String,
    pub backend_type: String,
    pub frontend_type: String,
    pub severity: String, // High/Medium/Low
    pub fix_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationSync {
    pub backend_auth_method: String,
    pub frontend_implementation: String, // Correct/Incorrect/Missing
    pub token_refresh_logic: String, // Implemented/Missing
    pub security_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCode {
    pub file_path: String,
    pub content: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationFix {
    pub endpoint: String,
    pub issue: String,
    pub fix_type: String, // Add/Update/Remove
    pub implementation: String,
    pub testing_requirements: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiClientConfiguration {
    pub base_url_config: String,
    pub timeout_settings: String,
    pub retry_logic: String,
    pub error_handling: String,
    pub interceptors: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTestingStrategy {
    pub unit_tests: Vec<TestFile>,
    pub integration_tests: Vec<TestFile>,
    pub mock_data: Vec<MockDataFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFile {
    pub test_file: String,
    pub content: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockDataFile {
    pub file_path: String,
    pub content: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationUpdate {
    pub file_path: String,
    pub content: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub breaking_changes: Vec<BreakingChange>,
    pub backward_compatibility: String,
    pub rollout_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    pub change: String,
    pub impact: String,
    pub migration_steps: Vec<String>,
    pub timeline: String, // Immediate/Next release/Future
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMonitoringSetup {
    pub api_metrics: String,
    pub error_tracking: String,
    pub performance_monitoring: String,
    pub alerting_rules: String,
}

/// Response structure for performance optimization prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceOptimizationResponse {
    pub id: Option<Uuid>,
    pub performance_analysis: PerformanceAnalysisDetailed,
    pub optimization_recommendations: Vec<OptimizationRecommendation>,
    pub code_optimizations: Vec<CodeOptimization>,
    pub infrastructure_changes: Vec<InfrastructureChange>,
    pub monitoring_improvements: MonitoringImprovements,
    pub testing_strategy: PerformanceTestingStrategy,
    pub implementation_plan: ImplementationPlan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysisDetailed {
    pub current_metrics: PerformanceMetrics,
    pub target_metrics: PerformanceMetrics,
    pub bottleneck_analysis: Vec<BottleneckAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub response_time_avg: String,
    pub throughput: String,
    pub cpu_usage: String,
    pub memory_usage: String,
    pub database_query_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckAnalysis {
    pub bottleneck: String,
    pub impact: String, // High/Medium/Low
    pub root_cause: String,
    pub optimization_potential: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub category: String, // Database/Code/Infrastructure/Frontend
    pub priority: String, // Critical/High/Medium/Low
    pub description: String,
    pub implementation: String,
    pub expected_improvement: String,
    pub effort_required: String, // Low/Medium/High
    pub risk_level: String, // Low/Medium/High
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeOptimization {
    pub file_path: String,
    pub optimization_type: String, // Algorithm/Query/Caching/Memory
    pub current_code: String,
    pub optimized_code: String,
    pub explanation: String,
    pub performance_impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureChange {
    pub component: String, // Database/Cache/Load Balancer/CDN
    pub change_type: String, // Configuration/Addition/Upgrade
    pub description: String,
    pub implementation_steps: Vec<String>,
    pub cost_impact: String,
    pub maintenance_requirements: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringImprovements {
    pub new_metrics: Vec<String>,
    pub alerting_thresholds: HashMap<String, String>,
    pub dashboard_updates: String,
    pub profiling_setup: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTestingStrategy {
    pub load_testing: String,
    pub stress_testing: String,
    pub performance_regression_tests: String,
    pub benchmarking: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPlan {
    pub phase_1: ImplementationPhase,
    pub phase_2: ImplementationPhase,
    pub phase_3: ImplementationPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPhase {
    pub timeline: String,
    pub optimizations: Vec<String>,
    pub expected_improvement: String,
}
