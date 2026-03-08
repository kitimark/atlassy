use atlassy_contracts::{ContractError, PipelineState};

#[derive(Debug, Clone)]
pub struct StateTracker {
    current: Option<PipelineState>,
}

impl StateTracker {
    pub fn new() -> Self {
        Self { current: None }
    }

    pub fn transition_to(&mut self, next: PipelineState) -> Result<(), ContractError> {
        let expected = PipelineState::expected_next(self.current)
            .map(|state| state.as_str().to_string())
            .unwrap_or_else(|| "<done>".to_string());
        if expected != next.as_str() {
            return Err(ContractError::InvalidTransition {
                expected,
                actual: next.as_str().to_string(),
            });
        }
        self.current = Some(next);
        Ok(())
    }
}

impl Default for StateTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_tracker_blocks_out_of_order_transitions() {
        let mut tracker = StateTracker::new();
        assert!(tracker.transition_to(PipelineState::Fetch).is_ok());
        let err = tracker.transition_to(PipelineState::Patch).unwrap_err();
        assert_eq!(
            err,
            ContractError::InvalidTransition {
                expected: "classify".to_string(),
                actual: "patch".to_string(),
            }
        );
    }
}
