//! Stack scaffolder trait and table-driven implementation.

use dare_core::{CoreResult, ProjectRoot};

use crate::types::{
    ScaffoldPlan, ScaffoldRequest, StackMetadata, ValidationReport, SCHEMA_VERSION,
};

/// Domain trait for per-stack scaffolding (BLUEPRINT-046 §0.5).
pub trait StackScaffolder: Send + Sync {
    fn id(&self) -> &'static str;
    fn metadata(&self) -> &StackMetadata;
    fn plan(&self, root: &ProjectRoot, req: &ScaffoldRequest) -> CoreResult<ScaffoldPlan>;
    fn validate(&self, root: &ProjectRoot) -> CoreResult<ValidationReport>;
}

/// Table-driven scaffolder shared by all 11 stacks in mp046-001.
pub struct GenericScaffolder {
    id: &'static str,
    metadata: StackMetadata,
}

impl GenericScaffolder {
    pub(crate) fn new(id: &'static str, metadata: StackMetadata) -> Self {
        Self { id, metadata }
    }

    /// Expected AX + meta paths for validate (honest empty-project report).
    fn expected_paths(&self) -> Vec<String> {
        let mut paths = vec![
            ".env.example".to_string(),
            "Dockerfile".to_string(),
            "README.md".to_string(),
            "dare.config.json".to_string(),
            "docker-compose.yml".to_string(),
            "llms.txt".to_string(),
            "openapi.json".to_string(),
            self.metadata.rate_limit_rel.clone(),
        ];
        paths.sort();
        paths
    }
}

impl StackScaffolder for GenericScaffolder {
    fn id(&self) -> &'static str {
        self.id
    }

    fn metadata(&self) -> &StackMetadata {
        &self.metadata
    }

    fn plan(&self, _root: &ProjectRoot, req: &ScaffoldRequest) -> CoreResult<ScaffoldPlan> {
        Ok(ScaffoldPlan {
            schema_version: SCHEMA_VERSION,
            stack_id: self.id.to_string(),
            project_name: req.project_name.clone(),
            items: vec![],
        })
    }

    fn validate(&self, _root: &ProjectRoot) -> CoreResult<ValidationReport> {
        let missing = self.expected_paths();
        Ok(ValidationReport {
            stack_id: self.id.to_string(),
            ok: false,
            missing,
            secret_hits: vec![],
        })
    }
}
