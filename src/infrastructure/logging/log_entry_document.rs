use chrono::{DateTime, Utc};
use mongodb::bson::{oid::ObjectId, serde_helpers::chrono_datetime_as_bson_datetime};
use serde::{Deserialize, Serialize};

use crate::domain::{
    entity::log_entry::LogEntry,
    value_object::{
        analysis_status::AnalysisStatus, architecture_layer::ArchitectureLayer,
        http_context::HttpContext, log_id::LogId, log_level::LogLevel, log_metadata::LogMetadata,
        operation::Operation, request_id::RequestId, user_context::UserContext, user_id::UserId,
    },
};
use crate::shared::error::infrastructure_error::InfrastructureError;

#[derive(Serialize, Deserialize, Debug)]
pub struct LogEntryDocument {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub mongo_id: Option<ObjectId>,

    pub log_id: String,
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub request_id: String,
    pub architecture_layer: String,
    pub operation: String,

    pub user_context: Option<UserContextDocument>,
    pub http_context: HttpContextDocument,
    pub metadata: serde_json::Value,

    pub tags: Vec<String>,
    pub analysis_status: String,
    pub related_log_ids: Vec<String>,

    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub indexed_at: DateTime<Utc>,
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HttpContextDocument {
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserContextDocument {
    pub user_id: String,
    pub username: Option<String>,
    pub role: Option<String>,
}

impl From<LogEntry> for LogEntryDocument {
    fn from(entry: LogEntry) -> Self {
        let now = Utc::now();
        
        Self {
            mongo_id: None,
            log_id: entry.id().to_string(),
            timestamp: entry.timestamp(),
            level: entry.level().to_string(),
            message: entry.message().to_string(),
            request_id: entry.request_id().to_string(),
            architecture_layer: entry.architecture_layer().to_string(),
            operation: entry.operation().to_string(),
            user_context: entry.user_context().as_ref().map(|ctx| UserContextDocument {
                user_id: ctx.user_id().value().to_string(),
                username: ctx.username().map(|s| s.to_string()),
                role: ctx.role().map(|s| s.to_string()),
            }),
            http_context: HttpContextDocument {
                method: entry.http_context().method().to_string(),
                path: entry.http_context().path().to_string(),
                status_code: entry.http_context().status_code(),
                response_time_ms: entry.http_context().response_time_ms(),
                user_agent: entry.http_context().user_agent().map(|s| s.to_string()),
                ip_address: entry.http_context().ip_address().map(|s| s.to_string()),
            },
            metadata: entry.metadata().to_json(),
            tags: entry.tags().to_vec(),
            analysis_status: entry.analysis_status().to_string(),
            related_log_ids: entry
                .related_log_ids()
                .iter()
                .map(|id| id.to_string())
                .collect(),
            indexed_at: now,
            created_at: entry.timestamp(),
            updated_at: now,
        }
    }
}

impl TryFrom<LogEntryDocument> for LogEntry {
    type Error = InfrastructureError;

    fn try_from(doc: LogEntryDocument) -> Result<Self, Self::Error> {
        let log_id = LogId::from_string(&doc.log_id)
            .map_err(|e| InfrastructureError::Deserialization(e.to_string()))?;

        let level = LogLevel::from_string(&doc.level)
            .map_err(|e| InfrastructureError::Deserialization(e.to_string()))?;

        let request_id = RequestId::from_string(&doc.request_id)
            .map_err(|e| InfrastructureError::Deserialization(e.to_string()))?;

        let architecture_layer = ArchitectureLayer::from_string(&doc.architecture_layer)
            .map_err(|e| InfrastructureError::Deserialization(e.to_string()))?;

        let operation = Operation::from_string(&doc.operation)
            .map_err(|e| InfrastructureError::Deserialization(e.to_string()))?;

        let user_context = doc
            .user_context
            .map(|ctx| {
                let user_id = UserId::new(ctx.user_id);
                UserContext::new(user_id, ctx.username, ctx.role)
            });

        let http_context = HttpContext::new(
            doc.http_context.method,
            doc.http_context.path,
            doc.http_context.status_code,
            doc.http_context.response_time_ms,
            doc.http_context.user_agent,
            doc.http_context.ip_address,
        );

        let metadata = LogMetadata::from_json(doc.metadata)
            .map_err(|e| InfrastructureError::Deserialization(e.to_string()))?;

        let analysis_status = AnalysisStatus::from_string(&doc.analysis_status)
            .map_err(|e| InfrastructureError::Deserialization(e.to_string()))?;

        let related_log_ids = doc
            .related_log_ids
            .into_iter()
            .map(|id| LogId::from_string(&id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| InfrastructureError::Deserialization(e.to_string()))?;

        LogEntry::reconstruct(
            log_id,
            doc.timestamp,
            level,
            doc.message,
            request_id,
            user_context,
            http_context,
            architecture_layer,
            operation,
            metadata,
            doc.tags,
            analysis_status,
            related_log_ids,
        )
        .map_err(|e| InfrastructureError::Deserialization(e.to_string()))
    }
}