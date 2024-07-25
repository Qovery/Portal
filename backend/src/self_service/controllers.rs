use std::sync::Arc;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::{debug_handler, Extension, Json};
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, info};

use crate::database;
use crate::database::{
    insert_self_service_run, list_logs_by_self_service_run_id, SelfServiceRunJson,
    SelfServiceRunLogJson, Status,
};
use crate::self_service::services::BackgroundWorkerTask;
use crate::self_service::ResultResponse;
use crate::self_service::{
    check_json_payload_against_yaml_config_fields, execute_command,
    find_self_service_section_by_slug, get_self_service_section_and_action,
    ExecValidateScriptRequest, JobResponse, ResultsResponse,
};
use crate::yaml_config::{
    SelfServiceSectionActionYamlConfig, SelfServiceSectionYamlConfig, YamlConfig,
};

#[debug_handler]
pub async fn list_self_service_sections(
    Extension(yaml_config): Extension<Arc<YamlConfig>>,
) -> (
    StatusCode,
    Json<ResultsResponse<SelfServiceSectionYamlConfig>>,
) {
    (
        StatusCode::OK,
        Json(ResultsResponse {
            message: None,
            results: yaml_config.self_service.sections.clone(),
        }),
    )
}

#[debug_handler]
pub async fn get_self_service_runs_by_id(
    Extension(pg_pool): Extension<Arc<sqlx::PgPool>>,
    Path(id): Path<String>,
) -> (StatusCode, Json<ResultResponse<SelfServiceRunJson>>) {
    match database::get_self_service_runs(&pg_pool, &id).await {
        Ok(self_service) => (
            StatusCode::OK,
            Json(ResultResponse {
                message: None,
                result: Some(self_service.to_json()),
            }),
        ),
        Err(err) => {
            error!("Failed to get self service: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResultResponse {
                    message: Some(err.to_string()),
                    result: None,
                }),
            )
        }
    }
}

#[debug_handler]
pub async fn list_self_service_section_actions(
    Extension(yaml_config): Extension<Arc<YamlConfig>>,
    Path(section_slug): Path<String>,
) -> (
    StatusCode,
    Json<ResultsResponse<SelfServiceSectionActionYamlConfig>>,
) {
    let section = match find_self_service_section_by_slug(
        &yaml_config.self_service.sections,
        section_slug.as_str(),
    ) {
        Some(section) => section,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ResultsResponse {
                    message: Some(format!("Self service section '{}' not found", section_slug)),
                    results: vec![],
                }),
            )
        }
    };

    (
        StatusCode::OK,
        Json(ResultsResponse {
            message: None,
            results: section.actions.clone().unwrap_or(vec![]),
        }),
    )
}

#[debug_handler]
pub async fn list_self_service_section_runs_by_section_and_action_slugs(
    Extension(pg_pool): Extension<Arc<sqlx::PgPool>>,
    Path((section_slug, action_slug)): Path<(String, String)>,
) -> (StatusCode, Json<ResultsResponse<SelfServiceRunJson>>) {
    match database::list_self_service_runs_by_section_and_action_slugs(
        &pg_pool,
        &section_slug,
        &action_slug,
    )
    .await
    {
        Ok(action_execution_statuses) => (
            StatusCode::OK,
            Json(ResultsResponse {
                message: None,
                results: action_execution_statuses
                    .iter()
                    .map(|x| x.to_json())
                    .collect(),
            }),
        ),
        Err(err) => {
            error!("failed to list action execution statuses: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResultsResponse {
                    message: Some(err.to_string()),
                    results: vec![],
                }),
            )
        }
    }
}

#[debug_handler]
pub async fn list_self_service_section_runs_by_section_slug(
    Extension(pg_pool): Extension<Arc<sqlx::PgPool>>,
    Path(section_slug): Path<String>,
) -> (StatusCode, Json<ResultsResponse<SelfServiceRunJson>>) {
    match database::list_self_service_runs_by_section_slug(&pg_pool, &section_slug).await {
        Ok(action_execution_statuses) => (
            StatusCode::OK,
            Json(ResultsResponse {
                message: None,
                results: action_execution_statuses
                    .iter()
                    .map(|x| x.to_json())
                    .collect(),
            }),
        ),
        Err(err) => {
            error!("failed to list action execution statuses: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResultsResponse {
                    message: Some(err.to_string()),
                    results: vec![],
                }),
            )
        }
    }
}

#[debug_handler]
pub async fn list_self_service_section_runs(
    Extension(pg_pool): Extension<Arc<sqlx::PgPool>>,
) -> (StatusCode, Json<ResultsResponse<SelfServiceRunJson>>) {
    match database::list_self_service_runs(&pg_pool).await {
        Ok(self_service_runs) => (
            StatusCode::OK,
            Json(ResultsResponse {
                message: None,
                results: self_service_runs.iter().map(|x| x.to_json()).collect(),
            }),
        ),
        Err(err) => {
            error!("failed to list action execution statuses: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResultsResponse {
                    message: Some(err.to_string()),
                    results: vec![],
                }),
            )
        }
    }
}

#[debug_handler]
pub async fn list_self_service_section_run_logs(
    Extension(pg_pool): Extension<Arc<sqlx::PgPool>>,
    Path(task_id): Path<String>,
) -> (StatusCode, Json<ResultsResponse<SelfServiceRunLogJson>>) {
    info!("List logs from self service run id={}", task_id);

    match list_logs_by_self_service_run_id(&pg_pool, &task_id).await {
        Ok(logs) => (
            StatusCode::OK,
            Json(ResultsResponse {
                message: None,
                results: logs.iter().map(|x| x.to_json()).collect(),
            }),
        ),
        Err(err) => {
            error!("failed to list action execution statuses: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ResultsResponse {
                    message: Some(err.to_string()),
                    results: vec![],
                }),
            )
        }
    }
}

#[debug_handler]
pub async fn exec_self_service_section_action_validate_scripts(
    Extension(yaml_config): Extension<Arc<YamlConfig>>,
    Path((section_slug, action_slug)): Path<(String, String)>,
    Extension(pg_pool): Extension<Arc<sqlx::PgPool>>,
    Json(req): Json<ExecValidateScriptRequest>,
) -> (StatusCode, Json<JobResponse>) {
    debug!("validate");

    let _ = match check_json_payload_against_yaml_config_fields(
        section_slug.as_str(),
        action_slug.as_str(),
        &req.payload,
        &yaml_config,
    ) {
        Ok(x) => x,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(JobResponse { message: Some(err) }),
            )
        }
    };

    let (_, action) = match get_self_service_section_and_action(
        &yaml_config,
        section_slug.as_str(),
        action_slug.as_str(),
    ) {
        Ok((section, action)) => (section, action),
        Err(err) => return err,
    };

    for cmd in action.validate.as_ref().unwrap_or(&vec![]) {
        let _ = match execute_command(
            pg_pool.to_owned(),
            cmd,
            req.payload.to_string().as_str(),
            "uuid",
        )
        .await
        {
            Ok(_) => (),
            Err(err) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(JobResponse { message: Some(err) }),
                )
            }
        };
    }

    (StatusCode::OK, Json(JobResponse { message: None }))
}

#[debug_handler]
pub async fn exec_self_service_section_action_post_validate_scripts(
    Extension(yaml_config): Extension<Arc<YamlConfig>>,
    Extension(tx): Extension<Sender<BackgroundWorkerTask>>,
    Extension(pg_pool): Extension<Arc<sqlx::PgPool>>,
    Path((section_slug, action_slug)): Path<(String, String)>,
    Json(req): Json<ExecValidateScriptRequest>,
) -> (StatusCode, Json<JobResponse>) {
    info!("Self service execute");

    let _ = match check_json_payload_against_yaml_config_fields(
        section_slug.as_str(),
        action_slug.as_str(),
        &req.payload,
        &yaml_config,
    ) {
        Ok(x) => x,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(JobResponse { message: Some(err) }),
            )
        }
    };

    let service = match get_self_service_section_and_action(
        &yaml_config,
        section_slug.as_str(),
        action_slug.as_str(),
    ) {
        Ok((_, service)) => service,
        Err(err) => return err,
    };

    let self_service_run = match insert_self_service_run(
        &pg_pool,
        &section_slug,
        &action_slug,
        Status::Queued,
        &req.payload,
        &serde_json::Value::Array(vec![]),
    )
    .await
    {
        Ok(self_service_run) => self_service_run,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JobResponse {
                    message: Some(err.to_string()),
                }),
            )
        }
    };

    info!("Insertion OK - {}", self_service_run.id());

    // execute post validate scripts
    let _ = tx
        .send(BackgroundWorkerTask::new(
            self_service_run.id(),
            service.clone(),
            req,
        ))
        .await
        .unwrap_or_else(|err| {
            error!("failed to send task to background worker: {}", err);
            // TODO change catalog execution status to Failure
        });

    (
        StatusCode::CREATED,
        Json(JobResponse {
            message: Some("workflow executed".to_string()),
        }),
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::extract::Path;
    use axum::http::StatusCode;
    use axum::{Extension, Json};

    use crate::self_service::controllers::exec_self_service_section_action_validate_scripts;
    use crate::self_service::ExecValidateScriptRequest;
    use crate::yaml_config::ActionFieldType::Text;
    use crate::yaml_config::{
        SelfServiceSectionActionFieldYamlConfig, SelfServiceSectionActionPostValidateYamlConfig,
        SelfServiceSectionActionValidateYamlConfig, SelfServiceSectionActionYamlConfig,
        SelfServiceSectionYamlConfig, SelfServiceYamlConfig, YamlConfig,
    };

    fn get_yaml_config() -> YamlConfig {
        YamlConfig {
            self_service: SelfServiceYamlConfig {
                sections: vec![SelfServiceSectionYamlConfig {
                    slug: "section-1".to_string(),
                    name: "Section 1".to_string(),
                    description: None,
                    actions: Some(vec![SelfServiceSectionActionYamlConfig {
                        slug: "action-1".to_string(),
                        name: "Action 1".to_string(),
                        description: None,
                        icon: None,
                        icon_color: None,
                        fields: Some(vec![
                            SelfServiceSectionActionFieldYamlConfig {
                                slug: "field-1".to_string(),
                                title: "Field 1".to_string(),
                                description: None,
                                placeholder: None,
                                type_: Text,
                                default: None,
                                required: Some(true),
                                autocomplete_fetcher: None,
                            },
                            SelfServiceSectionActionFieldYamlConfig {
                                slug: "field-2".to_string(),
                                title: "Field 2".to_string(),
                                description: None,
                                placeholder: None,
                                type_: Text,
                                default: None,
                                required: None,
                                autocomplete_fetcher: None,
                            },
                        ]),
                        validate: Some(vec![SelfServiceSectionActionValidateYamlConfig {
                            timeout: None,
                            command: vec![
                                "python3".to_string(),
                                "examples/validation_script_ok.py".to_string(),
                            ],
                        }]),
                        post_validate: Some(vec![SelfServiceSectionActionPostValidateYamlConfig {
                            timeout: None,
                            command: vec![
                                "python3".to_string(),
                                "examples/validation_script_ok.py".to_string(),
                            ],
                            output_model: None,
                        }]),
                    }]),
                }],
            },
        }
    }

    #[tokio::test]
    async fn test_exec_self_service_action_validate_scripts_ok() {
        let yaml_config = Arc::from(get_yaml_config());

        let (status_code, job_response) = exec_self_service_section_action_validate_scripts(
            Extension(yaml_config),
            Path(("section-1".to_string(), "action-1".to_string())),
            Json(ExecValidateScriptRequest {
                payload: serde_json::json!({
                    "field-1": "value-1",
                    "field-2": "value-2",
                }),
            }),
        )
        .await;

        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(job_response.message, None);
    }

    #[tokio::test]
    async fn test_exec_self_service_action_validate_scripts_ko() {
        let mut yaml_config = get_yaml_config();

        // add a failing validation script
        yaml_config.self_service.sections[0]
            .actions
            .as_mut()
            .unwrap()[0]
            .validate
            .as_mut()
            .unwrap()
            .push(SelfServiceSectionActionValidateYamlConfig {
                timeout: None,
                command: vec![
                    "python3".to_string(),
                    "examples/validation_script_ko.py".to_string(),
                ],
            });

        let (status_code, job_response) = exec_self_service_section_action_validate_scripts(
            Extension(Arc::from(yaml_config)),
            Path(("section-1".to_string(), "action-1".to_string())),
            Json(ExecValidateScriptRequest {
                payload: serde_json::json!({
                    "field-1": "value-1",
                    "field-2": "value-2",
                }),
            }),
        )
        .await;

        assert_eq!(status_code, StatusCode::BAD_REQUEST);
        assert_eq!(job_response.message.as_ref().unwrap().is_empty(), false);
    }

    #[tokio::test]
    async fn test_exec_self_service_action_validate_scripts_timeout() {
        // FIXME this test does not work because of tokio::test which is single threaded and does not allow to kill the child process
        // let mut yaml_config = get_yaml_config();
        //
        // // add a failing validation script
        // yaml_config.catalogs[0].services.as_mut().unwrap()[0].validate.as_mut().unwrap().push(CatalogServiceValidateYamlConfig {
        //     timeout: Some(1), // 1 second
        //     command: vec![
        //         "python3".to_string(),
        //         "examples/validation_script_ok.py".to_string(), // this is > 1 second
        //     ],
        // });
        //
        // let x = exec_catalog_service_validate_scripts(
        //     Extension(Arc::from(yaml_config)),
        //     Path(("catalog-1".to_string(), "service-1".to_string())),
        //     Json(ExecValidateScriptRequest {
        //         payload: serde_json::json!({
        //         "field-1": "value-1",
        //         "field-2": "value-2",
        //     })
        //     }),
        // ).await;
        //
        // assert_eq!(x.0, StatusCode::INTERNAL_SERVER_ERROR);
        // assert_eq!(x.1.message.as_ref().unwrap().is_empty(), false);
    }
}
