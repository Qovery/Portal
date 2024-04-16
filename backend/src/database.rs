use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sqlx::{Executor, Pool, Postgres};
use sqlx::types::Uuid;

use crate::errors::QError;

const DB_SCHEMA: &str = r#"
-- create status enum if it doesn't exist
DO
$$
    BEGIN
        IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'status') THEN
            CREATE TYPE status AS ENUM (
                'QUEUED',
                'RUNNING',
                'SUCCESS',
                'FAILURE'
                );
        END IF;
    END
$$;

-- create a new flat table to store action runs
CREATE TABLE IF NOT EXISTS self_service_runs
(
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    created_at    TIMESTAMP        DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at    TIMESTAMP        DEFAULT CURRENT_TIMESTAMP NOT NULL,
    section_slug  VARCHAR(255)                               NOT NULL,
    action_slug   VARCHAR(255)                               NOT NULL,
    status        status                                     NOT NULL,
    input_payload JSONB DEFAULT '{}'::jsonb                  NOT NULL,
    tasks         JSONB DEFAULT '{}'::jsonb                  NOT NULL
);

CREATE INDEX IF NOT EXISTS self_service_runs_section_slug_idx ON self_service_runs (section_slug);
CREATE INDEX IF NOT EXISTS self_service_runs_action_slug_idx ON self_service_runs (action_slug);
"#;

#[derive(sqlx::FromRow)]
pub struct SelfServiceRun {
    id: Uuid,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
    section_slug: String,
    action_slug: String,
    status: Status,
    input_payload: serde_json::Value,
    tasks: serde_json::Value,
}

#[derive(sqlx::Type, Clone, Serialize, Deserialize, Debug)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    // update schema.sql when adding new status
    Queued,
    Running,
    Success,
    Failure,
}

impl SelfServiceRun {
    pub fn to_json(&self) -> SelfServiceRunJson {
        SelfServiceRunJson {
            id: self.id.to_string(),
            created_at: self.created_at.to_string(),
            updated_at: self.updated_at.to_string(),
            section_slug: self.section_slug.clone(),
            action_slug: self.action_slug.clone(),
            status: self.status.clone(),
            input_payload: self.input_payload.clone(),
            tasks: self.tasks.clone(),
        }
    }

    pub fn id(&self) -> String {
        self.id.to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SelfServiceRunJson {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub section_slug: String,
    pub action_slug: String,
    pub status: Status,
    pub input_payload: serde_json::Value,
    pub tasks: serde_json::Value,
}

/// Initialize the database by creating the tables
pub async fn init_database(pg_pool: &Pool<Postgres>) -> Result<(), QError> {
    // execute SQL schema
    let _ = pg_pool.execute(DB_SCHEMA).await?;

    Ok(())
}

pub async fn list_self_service_runs_by_section_and_action_slugs(
    pg_pool: &Pool<Postgres>,
    section_slug: &str,
    action_slug: &str,
) -> Result<Vec<SelfServiceRun>, QError> {
    Ok(
        sqlx::query_as::<_, SelfServiceRun>(
            r#"
            SELECT *
            FROM self_service_runs
            WHERE section_slug = $1 AND action_slug = $2
            ORDER BY created_at DESC
        "#
        )
            .bind(section_slug)
            .bind(action_slug)
            .fetch_all(pg_pool)
            .await?
    )
}

pub async fn list_self_service_runs_by_section_slug(
    pg_pool: &Pool<Postgres>,
    section_slug: &str,
) -> Result<Vec<SelfServiceRun>, QError> {
    Ok(
        sqlx::query_as::<_, SelfServiceRun>(
            r#"
            SELECT *
            FROM self_service_runs
                WHERE section_slug = $1
            ORDER BY created_at DESC
        "#
        )
            .bind(section_slug)
            .fetch_all(pg_pool)
            .await?
    )
}

pub async fn list_self_service_runs(
    pg_pool: &Pool<Postgres>,
) -> Result<Vec<SelfServiceRun>, QError> {
    Ok(
        sqlx::query_as::<_, SelfServiceRun>(
            r#"
            SELECT *
            FROM self_service_runs
            ORDER BY created_at DESC
        "#
        )
            .fetch_all(pg_pool)
            .await?
    )
}

pub async fn insert_self_service_run(
    pg_pool: &Pool<Postgres>,
    section_slug: &str,
    action_slug: &str,
    status: Status,
    input_payload: &serde_json::Value,
    tasks: &serde_json::Value,
) -> Result<SelfServiceRun, QError> {
    Ok(
        sqlx::query_as::<_, SelfServiceRun>(
            r#"
            INSERT INTO self_service_runs (section_slug, action_slug, status, input_payload, tasks)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
        "#
        )
            .bind(section_slug)
            .bind(action_slug)
            .bind(status)
            .bind(input_payload)
            .bind(tasks)
            .fetch_one(pg_pool)
            .await?
    )
}

pub async fn update_self_service_run(
    pg_pool: &Pool<Postgres>,
    id: &str,
    status: Status,
    tasks: &serde_json::Value,
) -> Result<SelfServiceRun, QError> {
    Ok(
        sqlx::query_as::<_, SelfServiceRun>(
            r#"
            UPDATE self_service_runs
            SET status = $1, tasks = $2
            WHERE id = $3
            RETURNING *
        "#
        )
            .bind(status)
            .bind(tasks)
            .bind(Uuid::from_str(id).unwrap())
            .fetch_one(pg_pool)
            .await?
    )
}
