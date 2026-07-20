use palette_protocol::{WorkflowAction, WorkflowDefinition};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WorkflowValidationError {
    #[error("workflow id cannot be empty")]
    EmptyId,
    #[error("workflow name cannot be empty")]
    EmptyName,
    #[error("workflow must contain at least one action")]
    NoActions,
    #[error("workflow exceeds the maximum of 64 actions")]
    TooManyActions,
    #[error("action {index} has an empty required value")]
    EmptyActionValue { index: usize },
    #[error("action {index} has a non-finite parameter value")]
    NonFiniteParameter { index: usize },
}

pub fn validate_workflow(workflow: &WorkflowDefinition) -> Result<(), WorkflowValidationError> {
    if workflow.id.trim().is_empty() {
        return Err(WorkflowValidationError::EmptyId);
    }
    if workflow.name.trim().is_empty() {
        return Err(WorkflowValidationError::EmptyName);
    }
    if workflow.actions.is_empty() {
        return Err(WorkflowValidationError::NoActions);
    }
    if workflow.actions.len() > 64 {
        return Err(WorkflowValidationError::TooManyActions);
    }
    for (index, action) in workflow.actions.iter().enumerate() {
        let empty = match action {
            WorkflowAction::LoadItem { item_id, .. } => item_id.trim().is_empty(),
            WorkflowAction::InsertNative { name, .. } | WorkflowAction::RenameTrack { name } => {
                name.trim().is_empty()
            }
            WorkflowAction::SetParameter {
                device_name,
                parameter_name,
                value,
            } => {
                if !value.is_finite() {
                    return Err(WorkflowValidationError::NonFiniteParameter { index });
                }
                device_name.trim().is_empty() || parameter_name.trim().is_empty()
            }
            _ => false,
        };
        if empty {
            return Err(WorkflowValidationError::EmptyActionValue { index });
        }
    }
    Ok(())
}
