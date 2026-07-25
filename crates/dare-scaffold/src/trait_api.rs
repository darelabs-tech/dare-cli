//! Stack scaffolder trait and table-driven implementation.

use dare_core::{CoreResult, ProjectRoot};

use crate::plan::plan_scaffold;
use crate::types::{ScaffoldPlan, ScaffoldRequest, StackMetadata, ValidationReport};
use crate::validate::validate_stack_output;

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

}

impl StackScaffolder for GenericScaffolder {
    fn id(&self) -> &'static str {
        self.id
    }

    fn metadata(&self) -> &StackMetadata {
        &self.metadata
    }

    fn plan(&self, root: &ProjectRoot, req: &ScaffoldRequest) -> CoreResult<ScaffoldPlan> {
        plan_scaffold(root, req)
    }

    fn validate(&self, root: &ProjectRoot) -> CoreResult<ValidationReport> {
        validate_stack_output(root, self.id)
    }
}
