use std::collections::HashMap;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::error::ServerFnError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyCommand {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryRunRecord {
    pub id: i64,
    pub ts: i64,
    pub status: String,
    pub entry_room: String,
    pub requested_by: Option<String>,
    pub target_repo: Option<String>,
    pub source_brief: Option<String>,
    pub current_stage: Option<String>,
    pub active_room_ids: Vec<String>,
    pub artifact_ids: Vec<i64>,
    pub diagnostic_ids: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryArtifactRecord {
    pub id: i64,
    pub ts: i64,
    pub run_id: i64,
    pub artifact_type: String,
    pub version: u64,
    pub producer_stage_id: Option<i64>,
    pub input_artifact_ids: Vec<i64>,
    pub payload: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryStageRecord {
    pub id: i64,
    pub ts: i64,
    pub status: String,
    pub run_id: i64,
    pub stage_name: String,
    pub depends_on: Vec<i64>,
    pub input_artifact_ids: Vec<i64>,
    pub output_artifact_ids: Vec<i64>,
    pub source_artifact_id: Option<i64>,
    pub source_artifact_version: Option<u64>,
    pub superseded_by_artifact_id: Option<i64>,
    pub assigned_room: Option<String>,
    pub summary: Option<String>,
    pub verification_summary: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryIssueRecord {
    pub id: i64,
    pub ts: i64,
    pub status: String,
    pub run_id: i64,
    pub planner_key: Option<String>,
    pub title: String,
    pub kind: String,
    pub stage_id: Option<i64>,
    pub depends_on: Vec<i64>,
    pub input_artifact_ids: Vec<i64>,
    pub output_artifact_ids: Vec<i64>,
    pub source_artifact_id: Option<i64>,
    pub source_artifact_version: Option<u64>,
    pub superseded_by_artifact_id: Option<i64>,
    pub acceptance_criteria: Vec<String>,
    pub verification_requirements: Vec<String>,
    pub assigned_room: Option<String>,
    pub branch_name: Option<String>,
    pub worktree_path: Option<String>,
    pub summary: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryCheckpointRecord {
    pub id: i64,
    pub ts: i64,
    pub run_id: i64,
    pub status: String,
    pub branch_name: String,
    pub worktree_path: String,
    pub base_ref: String,
    pub included_issue_ids: Vec<i64>,
    pub merged_issue_ids: Vec<i64>,
    pub conflict_issue_ids: Vec<i64>,
    pub head_sha: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryRunStatus {
    pub run_id: i64,
    pub status: String,
    pub current_stage: Option<String>,
    pub stage_counts: HashMap<String, usize>,
    pub issue_counts: HashMap<String, usize>,
    pub latest_checkpoint_status: Option<String>,
    pub latest_checkpoint_id: Option<i64>,
    pub latest_verification_success: Option<bool>,
    pub latest_verification_artifact_id: Option<i64>,
    pub latest_gate_verdicts: HashMap<String, String>,
    pub blockers: Vec<String>,
    pub diagnostics: Vec<String>,
    pub next_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryRunStartRequest {
    pub entry_room: String,
    pub requested_by: Option<String>,
    pub target_repo: Option<String>,
    pub source_brief: Option<String>,
    pub planner_actor: Option<String>,
    pub planner_llm: Option<String>,
    pub verify_commands: Option<Vec<VerifyCommand>>,
    pub max_steps: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryRunStartResponse {
    pub run_id: i64,
    pub status: String,
    pub planning_rooms: Vec<String>,
    pub stage_ids: Vec<i64>,
    pub issue_ids: Vec<i64>,
    pub actions: Vec<String>,
    pub completed: bool,
    pub failed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryRunOrchestrateResponse {
    pub run_id: i64,
    pub status: String,
    pub actions: Vec<String>,
    pub completed: bool,
    pub failed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorySnapshot {
    pub run: FactoryRunRecord,
    pub status: FactoryRunStatus,
    pub stages: Vec<FactoryStageRecord>,
    pub issues: Vec<FactoryIssueRecord>,
    pub artifacts: Vec<FactoryArtifactRecord>,
    pub checkpoints: Vec<FactoryCheckpointRecord>,
    pub surface_gaps: Vec<String>,
}

#[server]
pub async fn factory_list_runs() -> Result<Vec<FactoryRunRecord>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::list_runs().await.map_err(ServerFnError::new);
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "factory_list_runs is only available on the server",
    ))
}

#[server]
pub async fn factory_get_run(run_id: i64) -> Result<FactoryRunRecord, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::get_run(run_id).await.map_err(ServerFnError::new);
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "factory_get_run is only available on the server",
    ))
}

#[server]
pub async fn factory_get_run_status(run_id: i64) -> Result<FactoryRunStatus, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::get_run_status(run_id)
            .await
            .map_err(ServerFnError::new);
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "factory_get_run_status is only available on the server",
    ))
}

#[server]
pub async fn factory_get_run_snapshot(run_id: i64) -> Result<FactorySnapshot, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::get_run_snapshot(run_id)
            .await
            .map_err(ServerFnError::new);
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "factory_get_run_snapshot is only available on the server",
    ))
}

#[server]
pub async fn factory_start_run(
    request: FactoryRunStartRequest,
) -> Result<FactoryRunStartResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::start_run(request).await.map_err(ServerFnError::new);
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "factory_start_run is only available on the server",
    ))
}

#[server]
pub async fn factory_orchestrate_run(
    run_id: i64,
    verify_commands: Option<Vec<VerifyCommand>>,
    max_steps: Option<u64>,
) -> Result<FactoryRunOrchestrateResponse, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::orchestrate_run(run_id, verify_commands, max_steps)
            .await
            .map_err(ServerFnError::new);
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "factory_orchestrate_run is only available on the server",
    ))
}

#[cfg(feature = "ssr")]
mod ssr {
    use prost_types::Struct;
    use serde::de::DeserializeOwned;

    use crate::auth::user::require_current_user;
    use crate::net::prior::client::{
        PriorSession, connect_session, disconnect_with_result, i64_number_value, struct_from_vec,
    };

    use super::*;

    pub async fn list_runs() -> Result<Vec<FactoryRunRecord>, String> {
        let actor = require_current_user()?.sub;
        let mut session = connect_session(&actor, None).await?;
        let result = request_items::<FactoryRunRecord>(
            &mut session,
            "factory:run:list",
            struct_from_vec(Vec::new()),
        )
        .await;
        disconnect_with_result(&mut session, result).await
    }

    pub async fn get_run(run_id: i64) -> Result<FactoryRunRecord, String> {
        let actor = require_current_user()?.sub;
        let mut session = connect_session(&actor, None).await?;
        let result = request_one::<FactoryRunRecord>(
            &mut session,
            "factory:run:get",
            struct_from_vec(vec![("id", i64_number_value(run_id)?)]),
        )
        .await;
        disconnect_with_result(&mut session, result).await
    }

    pub async fn get_run_status(run_id: i64) -> Result<FactoryRunStatus, String> {
        let actor = require_current_user()?.sub;
        let mut session = connect_session(&actor, None).await?;
        let result = request_one::<FactoryRunStatus>(
            &mut session,
            "factory:run:status",
            struct_from_vec(vec![("id", i64_number_value(run_id)?)]),
        )
        .await;
        disconnect_with_result(&mut session, result).await
    }

    pub async fn get_run_snapshot(run_id: i64) -> Result<FactorySnapshot, String> {
        let actor = require_current_user()?.sub;
        let mut session = connect_session(&actor, None).await?;

        let run = request_one::<FactoryRunRecord>(
            &mut session,
            "factory:run:get",
            struct_from_vec(vec![("id", i64_number_value(run_id)?)]),
        )
        .await?;
        let status = request_one::<FactoryRunStatus>(
            &mut session,
            "factory:run:status",
            struct_from_vec(vec![("id", i64_number_value(run_id)?)]),
        )
        .await?;
        let stages = request_items::<FactoryStageRecord>(
            &mut session,
            "factory:stage:list",
            struct_from_vec(vec![("run_id", i64_number_value(run_id)?)]),
        )
        .await?;
        let issues = request_items::<FactoryIssueRecord>(
            &mut session,
            "factory:issue:list",
            struct_from_vec(vec![("run_id", i64_number_value(run_id)?)]),
        )
        .await?;
        let artifacts = request_items::<FactoryArtifactRecord>(
            &mut session,
            "factory:artifact:list",
            struct_from_vec(vec![("run_id", i64_number_value(run_id)?)]),
        )
        .await?;
        let checkpoints = request_items::<FactoryCheckpointRecord>(
            &mut session,
            "factory:checkpoint:list",
            struct_from_vec(vec![("run_id", i64_number_value(run_id)?)]),
        )
        .await?;

        let snapshot = FactorySnapshot {
            run,
            status,
            stages,
            issues,
            artifacts,
            checkpoints,
            surface_gaps: vec![
                "factory gate records are not exposed via a list/get syscall; only latest verdict summaries are available through factory:run:status".into(),
                "review bundles are emitted via factory:review:bundle, but existing review bundle records are not listable from the current public factory surface".into(),
            ],
        };

        disconnect_with_result(&mut session, Ok(snapshot)).await
    }

    pub async fn start_run(
        request: FactoryRunStartRequest,
    ) -> Result<FactoryRunStartResponse, String> {
        let actor = require_current_user()?.sub;
        let mut session = connect_session(&actor, None).await?;
        let requested_by = request.requested_by.unwrap_or(actor);
        let result = request_one::<FactoryRunStartResponse>(
            &mut session,
            "factory:run:start",
            serde_to_struct(&serde_json::json!({
                "entry_room": request.entry_room,
                "requested_by": requested_by,
                "target_repo": request.target_repo,
                "source_brief": request.source_brief,
                "planner_actor": request.planner_actor,
                "planner_llm": request.planner_llm,
                "verify_commands": request.verify_commands,
                "max_steps": request.max_steps,
            }))?,
        )
        .await;
        disconnect_with_result(&mut session, result).await
    }

    pub async fn orchestrate_run(
        run_id: i64,
        verify_commands: Option<Vec<VerifyCommand>>,
        max_steps: Option<u64>,
    ) -> Result<FactoryRunOrchestrateResponse, String> {
        let actor = require_current_user()?.sub;
        let mut session = connect_session(&actor, None).await?;
        let result = request_one::<FactoryRunOrchestrateResponse>(
            &mut session,
            "factory:run:orchestrate",
            serde_to_struct(&serde_json::json!({
                "run_id": run_id,
                "verify_commands": verify_commands,
                "max_steps": max_steps,
            }))?,
        )
        .await;
        disconnect_with_result(&mut session, result).await
    }

    async fn request_one<T: DeserializeOwned>(
        session: &mut PriorSession,
        syscall: &str,
        data: Struct,
    ) -> Result<T, String> {
        session.request_one(syscall, None, data, None).await
    }

    async fn request_items<T: DeserializeOwned>(
        session: &mut PriorSession,
        syscall: &str,
        data: Struct,
    ) -> Result<Vec<T>, String> {
        session.request_items(syscall, None, data, None).await
    }

    fn serde_to_struct(value: &serde_json::Value) -> Result<Struct, String> {
        let serde_json::Value::Object(map) = value else {
            return Err("expected object payload".into());
        };

        let fields = map
            .iter()
            .map(|(key, value)| (key.clone(), json_to_prost_value(value)))
            .collect();
        Ok(Struct { fields })
    }

    fn json_to_prost_value(value: &serde_json::Value) -> prost_types::Value {
        use prost_types::value::Kind;

        let kind = match value {
            serde_json::Value::Null => Kind::NullValue(0),
            serde_json::Value::Bool(value) => Kind::BoolValue(*value),
            serde_json::Value::Number(number) => Kind::NumberValue(number.as_f64().unwrap_or(0.0)),
            serde_json::Value::String(text) => Kind::StringValue(text.clone()),
            serde_json::Value::Array(values) => Kind::ListValue(prost_types::ListValue {
                values: values.iter().map(json_to_prost_value).collect(),
            }),
            serde_json::Value::Object(map) => Kind::StructValue(Struct {
                fields: map
                    .iter()
                    .map(|(key, value)| (key.clone(), json_to_prost_value(value)))
                    .collect(),
            }),
        };

        prost_types::Value { kind: Some(kind) }
    }
}
