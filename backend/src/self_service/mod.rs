use std::time::Duration;

use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process;
use tokio::time::timeout;
use tracing::debug;

use crate::database::insert_self_service_run_log;
use crate::yaml_config::{
    ExternalCommand, SelfServiceSectionActionYamlConfig, SelfServiceSectionYamlConfig, YamlConfig,
};
use std::process::Stdio;

pub mod controllers;
pub mod services;

#[derive(Serialize, Deserialize)]
pub struct ResultsResponse<T> {
    message: Option<String>,
    results: Vec<T>,
}

#[derive(Serialize, Deserialize)]
pub struct ResultResponse<T> {
    message: Option<String>,
    result: Option<T>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JobResponse {
    message: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ExecValidateScriptRequest {
    payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct JobOutputResult {
    pub one_liner_command: String,
    pub output: serde_json::Value,
    pub execution_time_in_millis: u128,
}

fn find_self_service_section_by_slug<'a>(
    sections: &'a Vec<SelfServiceSectionYamlConfig>,
    section_slug: &str,
) -> Option<&'a SelfServiceSectionYamlConfig> {
    sections.iter().find(|section| section.slug == section_slug)
}

fn find_self_service_action_by_slug<'a>(
    section: &'a SelfServiceSectionYamlConfig,
    action_slug: &str,
) -> Option<&'a SelfServiceSectionActionYamlConfig> {
    section
        .actions
        .as_ref()
        .unwrap()
        .iter()
        .find(|action| action.slug == action_slug)
}

/// Extract the job output from the environment variable TORII_JSON_OUTPUT and reset it to an empty JSON object
fn consume_job_output_result_from_json_output_env(
    action_slug: &str,
    execution_time: u128,
) -> JobOutputResult {
    let job_output_result = match std::env::var("TORII_JSON_OUTPUT") {
        Ok(json_output) => JobOutputResult {
            one_liner_command: action_slug.to_string(),
            output: serde_json::from_str(json_output.as_str()).unwrap_or(serde_json::json!({})),
            execution_time_in_millis: execution_time,
        },
        Err(_) => JobOutputResult {
            one_liner_command: action_slug.to_string(),
            output: serde_json::json!({}),
            execution_time_in_millis: execution_time,
        },
    };

    // reset the environment variable
    std::env::set_var("TORII_JSON_OUTPUT", "{}");

    job_output_result
}

fn check_json_payload_against_yaml_config_fields(
    section_slug: &str,
    action_slug: &str,
    json_payload: &serde_json::Value,
    yaml_config: &YamlConfig,
) -> Result<(), String> {
    let section =
        match find_self_service_section_by_slug(&yaml_config.self_service.sections, section_slug) {
            Some(section) => section,
            None => return Err(format!("Self service section '{}' not found", section_slug)),
        };

    let action = match find_self_service_action_by_slug(section, action_slug) {
        Some(action) => action,
        None => return Err(format!("Action '{}' not found", action_slug)),
    };

    let fields = match action.fields.as_ref() {
        Some(fields) => fields,
        None => return Err(format!("Action '{}' has no fields", action_slug)),
    };

    for field in fields {
        let field_value = match json_payload.get(field.slug.as_str()) {
            Some(field_value) => field_value,
            None => return Err(format!("Field '{}' not found in payload", field.slug)),
        };

        if field.required.unwrap_or(false) && field_value.is_null() {
            return Err(format!("Field '{}' is required", field.slug));
        }
    }

    Ok(())
}

async fn execute_command<T>(
    pg_pool: Arc<sqlx::PgPool>,
    external_command: &T,
    json_payload: &str,
    task_id: &str,
) -> Result<JobOutputResult, String>
where
    T: ExternalCommand,
{
    let cmd_one_line = external_command.get_command().join(" ");

    debug!(
        "executing validate script '{}' with payload '{}'",
        &cmd_one_line, json_payload
    );

    if external_command.get_command().len() == 1 {
        return Err(format!(
            "Validate script '{}' is invalid. \
                Be explicit on the command to execute, e.g. 'python examples/validation_script.py'",
            external_command.get_command()[0]
        ));
    }

    let mut cmd = process::Command::new(&external_command.get_command()[0]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    for arg in external_command.get_command()[1..].iter() {
        cmd.arg(arg);
    }

    cmd.arg(json_payload);

    // start execution timer
    let start = std::time::Instant::now();

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(err) => {
            return Err(format!(
                "Validate script '{}' failed: {}",
                &cmd_one_line, err
            ))
        }
    };

    let stdout = child
        .stdout
        .take()
        .expect("child did not have a handle to stdout");
    let stderr = child
        .stderr
        .take()
        .expect("child did not have a handle to stderr");

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        match insert_self_service_run_log(&pg_pool, task_id, line.clone(), false).await {
                            Ok(x) => x,
                            Err(_err) => break,
                        };
                        println!("Stdout: {}", line.clone())
                    },
                    Err(_) => break,
                    _ => (),
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        match insert_self_service_run_log(&pg_pool, task_id, line.clone(), true).await {
                            Ok(x) => x,
                            Err(_err) => break,
                        };
                        println!("Stderr: {}", line.clone())
                    },
                    Err(_) => break,
                    _ => (),
                }
            }
            result = child.wait() => {
                match result {
                    Ok(exit_code) => println!("33 Child process exited with {}", exit_code),
                    _ => (),
                }
                break
            }
        };
    }

    let exit_status = match timeout(Duration::from_secs(external_command.get_timeout()), child.wait()).await {
        Ok(exit_status) => exit_status,
        Err(_) => return Err(match child.kill().await {
            Ok(_) => format!(
                "Validate script '{}' timed out after {} seconds",
                &cmd_one_line,
                external_command.get_timeout()
            ),
            Err(err) => format!(
                "Validate script '{}' timed out after {} seconds, but failed to kill the process: {}",
                &cmd_one_line, external_command.get_timeout(), err
            )
        })
    }.unwrap();

    if !exit_status.success() {
        return Err(format!(
            "Validate script '{}' failed: {:?}",
            &cmd_one_line, exit_status
        ));
    }

    Ok(consume_job_output_result_from_json_output_env(
        cmd_one_line.as_str(),
        start.elapsed().as_millis(),
    ))
}

fn get_self_service_section_and_action<'a>(
    yaml_config: &'a YamlConfig,
    section_slug: &str,
    action_slug: &str,
) -> Result<
    (
        &'a SelfServiceSectionYamlConfig,
        &'a SelfServiceSectionActionYamlConfig,
    ),
    (StatusCode, Json<JobResponse>),
> {
    let section =
        match find_self_service_section_by_slug(&yaml_config.self_service.sections, section_slug) {
            Some(section) => section,
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(JobResponse {
                        message: Some(format!("Self service section '{}' not found", section_slug)),
                    }),
                ))
            }
        };

    let action = match find_self_service_action_by_slug(section, action_slug) {
        Some(action) => action,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(JobResponse {
                    message: Some(format!("Action '{}' not found", action_slug)),
                }),
            ))
        }
    };

    Ok((section, action))
}

#[cfg(test)]
mod tests {
    use crate::self_service::{
        find_self_service_action_by_slug, find_self_service_section_by_slug,
    };
    use crate::yaml_config::{SelfServiceSectionActionYamlConfig, SelfServiceSectionYamlConfig};

    #[test]
    fn test_find_section_by_slug() {
        let sections = vec![
            SelfServiceSectionYamlConfig {
                slug: "section-1".to_string(),
                name: "Section 1".to_string(),
                description: None,
                actions: None,
            },
            SelfServiceSectionYamlConfig {
                slug: "section-2".to_string(),
                name: "Section 2".to_string(),
                description: None,
                actions: None,
            },
        ];

        assert_eq!(
            find_self_service_section_by_slug(&sections, "section-1"),
            Some(&sections[0])
        );
        assert_eq!(
            find_self_service_section_by_slug(&sections, "section-2"),
            Some(&sections[1])
        );
        assert_eq!(
            find_self_service_section_by_slug(&sections, "section-3"),
            None
        );
    }

    #[test]
    fn test_find_action_by_slug() {
        let section = SelfServiceSectionYamlConfig {
            slug: "section-1".to_string(),
            name: "Section 1".to_string(),
            description: None,
            actions: Some(vec![
                SelfServiceSectionActionYamlConfig {
                    slug: "action-1".to_string(),
                    name: "Action 1".to_string(),
                    description: None,
                    icon: None,
                    icon_color: None,
                    fields: None,
                    validate: None,
                    post_validate: None,
                },
                SelfServiceSectionActionYamlConfig {
                    slug: "action-2".to_string(),
                    name: "Action 2".to_string(),
                    description: None,
                    icon: None,
                    icon_color: None,
                    fields: None,
                    validate: None,
                    post_validate: None,
                },
            ]),
        };

        assert_eq!(
            find_self_service_action_by_slug(&section, "action-1"),
            Some(&section.actions.as_ref().unwrap()[0])
        );
        assert_eq!(
            find_self_service_action_by_slug(&section, "action-2"),
            Some(&section.actions.as_ref().unwrap()[1])
        );
        assert_eq!(find_self_service_action_by_slug(&section, "action-3"), None);
    }
}
