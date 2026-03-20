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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryDashboardView {
    pub runs: Vec<FactoryRunDisplay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryRunDisplay {
    pub id: i64,
    pub title: String,
    pub repo_label: String,
    pub requested_by: String,
    pub status: String,
    pub current_stage: String,
    pub summary: String,
    pub active_room_count: usize,
    pub issue_count: usize,
    pub blocker_count: usize,
    pub needs_attention: bool,
    pub completed: bool,
    pub updates: Vec<FactoryRunUpdateDisplay>,
    pub lifecycle: Vec<FactoryLifecyclePhaseDisplay>,
    pub next_actions: Vec<String>,
    pub diagnostics: Vec<String>,
    pub room_labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryRunUpdateDisplay {
    pub kind: String,
    pub ts: i64,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryLifecyclePhaseDisplay {
    pub name: String,
    pub detail: String,
    pub state: String,
}

#[server]
pub async fn fetch_factory_dashboard() -> Result<FactoryDashboardView, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return ssr::dashboard_view().await.map_err(ServerFnError::new);
    }

    #[allow(unreachable_code)]
    Err(ServerFnError::new(
        "fetch_factory_dashboard is only available on the server",
    ))
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
    use std::cmp::Reverse;

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

    pub async fn dashboard_view() -> Result<FactoryDashboardView, String> {
        let actor = require_current_user()?.sub;
        let mut session = connect_session(&actor, None).await?;
        let run_records = request_items::<FactoryRunRecord>(
            &mut session,
            "factory:run:list",
            struct_from_vec(Vec::new()),
        )
        .await?;

        let mut runs = Vec::with_capacity(run_records.len());
        for run in run_records {
            let snapshot = get_run_snapshot_with_session(&mut session, run.id).await?;
            runs.push(build_run_display(snapshot));
        }

        runs.sort_by_key(|run| Reverse(run.id));
        disconnect_with_result(&mut session, Ok(FactoryDashboardView { runs })).await
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
        let snapshot = get_run_snapshot_with_session(&mut session, run_id).await?;
        disconnect_with_result(&mut session, Ok(snapshot)).await
    }

    async fn get_run_snapshot_with_session(
        session: &mut PriorSession,
        run_id: i64,
    ) -> Result<FactorySnapshot, String> {
        let run = request_one::<FactoryRunRecord>(
            session,
            "factory:run:get",
            struct_from_vec(vec![("id", i64_number_value(run_id)?)]),
        )
        .await?;
        let status = request_one::<FactoryRunStatus>(
            session,
            "factory:run:status",
            struct_from_vec(vec![("id", i64_number_value(run_id)?)]),
        )
        .await?;
        let stages = request_items::<FactoryStageRecord>(
            session,
            "factory:stage:list",
            struct_from_vec(vec![("run_id", i64_number_value(run_id)?)]),
        )
        .await?;
        let issues = request_items::<FactoryIssueRecord>(
            session,
            "factory:issue:list",
            struct_from_vec(vec![("run_id", i64_number_value(run_id)?)]),
        )
        .await?;
        let artifacts = request_items::<FactoryArtifactRecord>(
            session,
            "factory:artifact:list",
            struct_from_vec(vec![("run_id", i64_number_value(run_id)?)]),
        )
        .await?;
        let checkpoints = request_items::<FactoryCheckpointRecord>(
            session,
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
        Ok(snapshot)
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

    fn build_run_display(snapshot: FactorySnapshot) -> FactoryRunDisplay {
        let FactorySnapshot {
            run,
            status,
            mut stages,
            mut issues,
            artifacts: _artifacts,
            mut checkpoints,
            surface_gaps,
        } = snapshot;

        stages.sort_by_key(|stage| stage.ts);
        issues.sort_by_key(|issue| issue.ts);
        checkpoints.sort_by_key(|checkpoint| checkpoint.ts);

        let title = run_title(&run);
        let repo_label = run
            .target_repo
            .clone()
            .unwrap_or_else(|| run.entry_room.clone());
        let requested_by = run
            .requested_by
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let current_stage = status
            .current_stage
            .clone()
            .or_else(|| run.current_stage.clone())
            .unwrap_or_else(|| "awaiting stage".to_string());
        let blocker_count = status.blockers.len();
        let issue_count = issues.len();
        let needs_attention = blocker_count > 0
            || status.latest_verification_success == Some(false)
            || contains_attention_status(&run.status)
            || status
                .latest_checkpoint_status
                .as_deref()
                .is_some_and(contains_attention_status);
        let completed = contains_completed_status(&run.status)
            || status
                .latest_checkpoint_status
                .as_deref()
                .is_some_and(contains_completed_status)
            || current_stage.eq_ignore_ascii_case("complete");
        let summary = run_summary(&run, &status, issue_count, blocker_count);
        let updates =
            build_run_updates(&run, &status, &stages, &issues, &checkpoints, &surface_gaps);
        let lifecycle = build_lifecycle(&run, &status);

        FactoryRunDisplay {
            id: run.id,
            title,
            repo_label,
            requested_by,
            status: run.status,
            current_stage,
            summary,
            active_room_count: run.active_room_ids.len(),
            issue_count,
            blocker_count,
            needs_attention,
            completed,
            updates,
            lifecycle,
            next_actions: status.next_actions,
            diagnostics: status.diagnostics,
            room_labels: run.active_room_ids,
        }
    }

    fn build_run_updates(
        run: &FactoryRunRecord,
        status: &FactoryRunStatus,
        stages: &[FactoryStageRecord],
        issues: &[FactoryIssueRecord],
        checkpoints: &[FactoryCheckpointRecord],
        surface_gaps: &[String],
    ) -> Vec<FactoryRunUpdateDisplay> {
        let mut updates = Vec::new();

        if let Some(brief) = run.source_brief.as_deref().map(compact_text) {
            updates.push(FactoryRunUpdateDisplay {
                kind: "brief".into(),
                ts: run.ts,
                title: "Run requested".into(),
                body: brief,
            });
        }

        updates.extend(stages.iter().map(|stage| {
            FactoryRunUpdateDisplay {
                kind: "stage".into(),
                ts: stage.ts,
                title: format!("Stage: {} ({})", stage.stage_name, stage.status),
                body: stage
                    .summary
                    .as_deref()
                    .or(stage.verification_summary.as_deref())
                    .map_or_else(
                        || {
                            stage.assigned_room.as_deref().map_or_else(
                                || "Stage activity recorded.".to_string(),
                                |room| format!("Assigned execution room: {room}"),
                            )
                        },
                        compact_text,
                    ),
            }
        }));

        updates.extend(issues.iter().map(|issue| FactoryRunUpdateDisplay {
            kind: "issue".into(),
            ts: issue.ts,
            title: format!("Issue: {} ({})", issue.title, issue.status),
            body: issue.summary.as_deref().map_or_else(
                || {
                    issue.assigned_room.as_deref().map_or_else(
                        || "Issue queued in the factory.".to_string(),
                        |room| format!("Assigned issue room: {room}"),
                    )
                },
                compact_text,
            ),
        }));

        updates.extend(
            checkpoints
                .iter()
                .map(|checkpoint| FactoryRunUpdateDisplay {
                    kind: "checkpoint".into(),
                    ts: checkpoint.ts,
                    title: format!("Checkpoint {} ({})", checkpoint.id, checkpoint.status),
                    body: format!(
                        "branch {} merged {}/{} issue(s)",
                        checkpoint.branch_name,
                        checkpoint.merged_issue_ids.len(),
                        checkpoint.included_issue_ids.len()
                    ),
                }),
        );

        updates.extend(status.blockers.iter().enumerate().map(|(index, blocker)| {
            FactoryRunUpdateDisplay {
                kind: "blocker".into(),
                ts: run.ts + i64::try_from(index).unwrap_or(0),
                title: "Blocker".into(),
                body: blocker.clone(),
            }
        }));

        updates.extend(
            status
                .next_actions
                .iter()
                .enumerate()
                .map(|(index, action)| FactoryRunUpdateDisplay {
                    kind: "next_action".into(),
                    ts: run.ts + 10_000 + i64::try_from(index).unwrap_or(0),
                    title: "Next action".into(),
                    body: action.clone(),
                }),
        );

        updates.extend(surface_gaps.iter().enumerate().map(|(index, gap)| {
            FactoryRunUpdateDisplay {
                kind: "gap".into(),
                ts: run.ts + 20_000 + i64::try_from(index).unwrap_or(0),
                title: "Surface gap".into(),
                body: gap.clone(),
            }
        }));

        updates.sort_by_key(|update| update.ts);
        updates
    }

    fn build_lifecycle(
        run: &FactoryRunRecord,
        status: &FactoryRunStatus,
    ) -> Vec<FactoryLifecyclePhaseDisplay> {
        const PHASES: [(&str, &str); 7] = [
            ("interpret", "Normalize problem brief"),
            ("resolve", "Bind to repo context"),
            ("plan", "Generate stage graph"),
            ("execute", "Run issues"),
            ("verify", "Run tests"),
            ("gate", "Merge readiness"),
            ("complete", "PR + review"),
        ];

        let current_stage = status
            .current_stage
            .as_deref()
            .or(run.current_stage.as_deref())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let current_index = PHASES
            .iter()
            .position(|(phase, _)| current_stage.contains(phase));
        let run_complete = contains_completed_status(&run.status);

        PHASES
            .iter()
            .enumerate()
            .map(|(index, (name, detail))| {
                let state = if run_complete || current_stage == "complete" {
                    if *name == "complete" {
                        "active"
                    } else {
                        "done"
                    }
                } else if Some(index) < current_index {
                    "done"
                } else if Some(index) == current_index {
                    "active"
                } else {
                    "future"
                };

                FactoryLifecyclePhaseDisplay {
                    name: capitalize(name),
                    detail: (*detail).to_string(),
                    state: state.to_string(),
                }
            })
            .collect()
    }

    fn run_title(run: &FactoryRunRecord) -> String {
        run.source_brief
            .as_deref()
            .map(compact_text)
            .filter(|brief| !brief.is_empty())
            .unwrap_or_else(|| {
                run.target_repo
                    .clone()
                    .unwrap_or_else(|| format!("Factory run {}", run.id))
            })
    }

    fn run_summary(
        run: &FactoryRunRecord,
        status: &FactoryRunStatus,
        issue_count: usize,
        blocker_count: usize,
    ) -> String {
        if let Some(action) = status.next_actions.first() {
            return compact_text(action);
        }
        if let Some(blocker) = status.blockers.first() {
            return format!("Blocked: {}", compact_text(blocker));
        }
        if let Some(stage) = status
            .current_stage
            .as_deref()
            .or(run.current_stage.as_deref())
        {
            return format!("{issue_count} issue(s) across {stage}");
        }
        if blocker_count > 0 {
            return format!("{blocker_count} blocker(s) need attention");
        }
        format!("{} active room(s)", run.active_room_ids.len())
    }

    fn compact_text(value: &str) -> String {
        let mut single_line = value.split_whitespace().collect::<Vec<_>>().join(" ");
        if single_line.len() > 120 {
            single_line.truncate(117);
            single_line.push_str("...");
        }
        single_line
    }

    fn contains_attention_status(status: &str) -> bool {
        let status = status.to_ascii_lowercase();
        status.contains("failed")
            || status.contains("blocked")
            || status.contains("error")
            || status.contains("cancel")
    }

    fn contains_completed_status(status: &str) -> bool {
        let status = status.to_ascii_lowercase();
        status.contains("complete")
            || status.contains("ready")
            || status.contains("delivered")
            || status.contains("merged")
    }

    fn capitalize(value: &str) -> String {
        let mut chars = value.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    }
}
