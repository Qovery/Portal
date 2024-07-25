use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error, info};

use crate::database::{Status, update_self_service_run};
use crate::self_service::{execute_command, ExecValidateScriptRequest, JobOutputResult};
use crate::yaml_config::{SelfServiceSectionActionPostValidateYamlConfig, SelfServiceSectionActionYamlConfig};

#[derive(Serialize, Deserialize)]
pub struct BackgroundWorkerTask {
    pub execution_status_id: String,
    pub self_service_section_action_yaml_config: SelfServiceSectionActionYamlConfig,
    pub req: ExecValidateScriptRequest,
}

impl BackgroundWorkerTask {
    pub fn new(
        execution_status_id: String,
        self_service_section_action_yaml_config: SelfServiceSectionActionYamlConfig,
        req: ExecValidateScriptRequest,
    ) -> Self {
        debug!("BackgroundWorkerTask instantied");
        Self {
            execution_status_id,
            self_service_section_action_yaml_config,
            req,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TaskPayload {
    status: Status,
    message: Option<String>,
    post_validate_input: SelfServiceSectionActionPostValidateYamlConfig,
    post_validate_output: Option<JobOutputResult>,
}

pub async fn background_worker(mut rx: Receiver<BackgroundWorkerTask>, pg_pool: Arc<Pool<Postgres>>) {
    while let Some(task) = rx.recv().await {
        info!("task id {}", task.execution_status_id.as_str());

        let r = update_self_service_run(
            &pg_pool,
            task.execution_status_id.as_str(),
            Status::Running,
            &serde_json::Value::Array(vec![]),
        ).await;

        info!("{}", "2");


        if let Err(err) = r {
            error!("failed to update action execution status: {}", err);
            continue;
        }

        info!("{}", "3");

        let mut tasks = Vec::<TaskPayload>::new();

        info!("{}", "4");
        let mut last_task_value = serde_json::Value::Array(vec![]);
        info!("{}", "5");
        for cmd in task.self_service_section_action_yaml_config.post_validate.as_ref().unwrap_or(&vec![]) {
            info!("{}", cmd);
            let job_output_result = match execute_command(pg_pool.clone(), cmd, task.req.payload.to_string().as_str(), task.execution_status_id.as_str()).await {
                Ok(job_output_result) => job_output_result,
                Err(err) => {
                    let task_payload = TaskPayload {
                        status: Status::Failure,
                        message: Some(err.to_string()),
                        post_validate_input: cmd.clone(),
                        post_validate_output: None,
                    };

                    let _ = tasks.push(task_payload);

                    let _ = update_self_service_run(
                        &pg_pool,
                        task.execution_status_id.as_str(),
                        Status::Failure,
                        &serde_json::to_value(tasks.clone()).unwrap(),
                    ).await;

                    break;
                }
            };

            // TODO pass output to next command

            let task_payload = TaskPayload {
                status: Status::Success,
                message: None,
                post_validate_input: cmd.clone(),
                post_validate_output: Some(job_output_result),
            };

            let _ = tasks.push(task_payload);

            last_task_value = serde_json::to_value(tasks.clone()).unwrap();

            let _ = update_self_service_run(
                &pg_pool,
                task.execution_status_id.as_str(),
                Status::Running,
                &last_task_value,
            ).await;
        }

        let _ = update_self_service_run(
            &pg_pool,
            task.execution_status_id.as_str(),
            Status::Success,
            &last_task_value,
        ).await;
    }
}
