//! Scheduler command types — minox publishes, nexox/argox consume.
//!
//! Topics: `minox/down/commands/{schedule-task, route-to-tier, throttle-node}`
//! Also: `nexox/down/commands/<service>/<cmd>` for operator commands from argox.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::validate::PayloadValidation;

/// minox scheduler assigns a task to a specific node + tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleDecision {
    pub task_id: Uuid,
    pub target_node: String,
    pub target_tier: u8,
    pub reason: String,
}

impl PayloadValidation for ScheduleDecision {
    fn is_valid(&self) -> bool {
        !self.target_node.is_empty() && (1..=4).contains(&self.target_tier)
    }
}

/// minox requests routing change for a node between tiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierRoute {
    pub node: String,
    pub from_tier: u8,
    pub to_tier: u8,
    pub reason: String,
}

impl PayloadValidation for TierRoute {
    fn is_valid(&self) -> bool {
        !self.node.is_empty()
            && (1..=4).contains(&self.from_tier)
            && (1..=4).contains(&self.to_tier)
    }
}

/// minox requests throttle action on a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThrottleCommand {
    pub node: String,
    pub action: ThrottleAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThrottleAction {
    ReduceFps { target_fps: u32 },
    PauseStream,
    ResumeStream,
}

impl PayloadValidation for ThrottleCommand {
    fn is_valid(&self) -> bool {
        !self.node.is_empty()
    }
}

/// Task descriptor for the scheduling pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDescriptor {
    pub task_id: Uuid,
    pub task_type: String,
    pub priority: TaskPriority,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    High,
    Normal,
    Low,
}

impl PayloadValidation for TaskDescriptor {
    fn is_valid(&self) -> bool {
        !self.task_type.is_empty()
    }
}

/// Operator command from argox UI → nexox supervisor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorCommand {
    pub service: String,
    pub command: String,
    pub args: serde_json::Value,
}

impl PayloadValidation for OperatorCommand {
    fn is_valid(&self) -> bool {
        !self.service.is_empty() && !self.command.is_empty()
    }
}
