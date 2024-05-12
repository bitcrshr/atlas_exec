use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::{
    macros::{date, time},
    Duration, PrimitiveDateTime,
};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct File {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub version: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct AppliedFile {
    #[serde(flatten)]
    pub file: File,

    #[serde(default = "default_time")]
    pub start: PrimitiveDateTime,

    #[serde(default = "default_time")]
    pub end: PrimitiveDateTime,

    pub skipped: isize,

    pub applied: Vec<String>,

    pub error: Option<SqlError>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct RevertedFile {
    #[serde(flatten)]
    pub file: File,

    #[serde(default = "default_time")]
    pub start: PrimitiveDateTime,

    #[serde(default = "default_time")]
    pub end: PrimitiveDateTime,

    pub skipped: isize,

    pub applied: Vec<String>,

    pub scope: String,

    pub error: Option<SqlError>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MigrateApply {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pending: Vec<File>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub applied: Vec<AppliedFile>,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub current: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub target: String,

    #[serde(default = "default_time")]
    pub start: PrimitiveDateTime,

    #[serde(default = "default_time")]
    pub end: PrimitiveDateTime,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub error: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MigrateDown {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub planned: Vec<File>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reverted: Vec<RevertedFile>,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub current: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub target: String,

    #[serde(skip_serializing_if = "isize_is_zero")]
    pub total: isize,

    #[serde(default = "default_time")]
    pub start: PrimitiveDateTime,

    #[serde(default = "default_time")]
    pub end: PrimitiveDateTime,

    #[serde(rename = "URL", skip_serializing_if = "String::is_empty")]
    pub url: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub status: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub error: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MigrateStatus {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub available: Vec<File>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pending: Vec<File>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub applied: Vec<Revision>,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub current: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub next: String,

    #[serde(skip_serializing_if = "isize_is_zero")]
    pub count: isize,

    #[serde(skip_serializing_if = "isize_is_zero")]
    pub total: isize,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub status: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub error: String,

    #[serde(rename = "SQL", skip_serializing_if = "String::is_empty")]
    pub sql: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SummaryReport {
    #[serde(rename = "URL", skip_serializing_if = "String::is_empty")]
    pub url: String,

    pub env: Env,

    pub schema: SummaryReportSchema,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<StepReport>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<FileReport>,
}
impl SummaryReport {
    pub fn diagnostics_count(&self) -> isize {
        let mut n: isize = 0;

        for f in &self.files {
            for r in &f.reports {
                n += r.diagnostics.len() as isize
            }
        }

        n
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Env {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub driver: String,

    #[serde(rename = "URL", skip_serializing_if = "String::is_empty")]
    pub url: String, // TODO: sqlclient.URL

    #[serde(skip_serializing_if = "String::is_empty")]
    pub dir: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SummaryReportSchema {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub current: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub desired: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct StepReport {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub text: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub error: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<FileReport>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct FileReport {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub text: String,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reports: Vec<Report>,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub error: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct StmtError {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub stmt: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Changes {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub applied: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pending: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<StmtError>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SchemaApply {
    #[serde(flatten)]
    pub env: Env,

    #[serde(skip_serializing_if = "changes_all_zero")]
    pub changes: Changes,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub error: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Revision {
    pub version: String,

    pub description: String,

    #[serde(rename = "Type")]
    pub typ: String,

    pub applied: isize,

    pub total: isize,

    #[serde(default = "default_time")]
    pub executed_at: PrimitiveDateTime,

    pub execution_time: Duration,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub error: String,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub error_stmt: String,

    pub operator_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Version {
    pub version: String,

    #[serde(rename = "SHA", skip_serializing_if = "String::is_empty")]
    pub sha: String,

    #[serde(skip_serializing_if = "bool_is_zero")]
    pub canary: bool,
}

#[derive(Debug, Error)]
#[error("{}", self.err_string())]
pub struct MigrateApplyError {
    pub result: Vec<MigrateApply>,
}
impl MigrateApplyError {
    pub fn new(result: Vec<MigrateApply>) -> Self {
        Self { result }
    }

    pub fn err_string(&self) -> String {
        match self.result.iter().last() {
            Some(last) => last.error.clone(),
            None => String::new(),
        }
    }
}

#[derive(Debug, Error)]
#[error("{}", self.err_string())]
pub struct SchemaApplyError {
    pub result: Vec<SchemaApply>,
}
impl SchemaApplyError {
    pub fn new(result: Vec<SchemaApply>) -> Self {
        Self { result }
    }

    pub fn err_string(&self) -> String {
        match self.result.iter().last() {
            Some(last) => last.error.clone(),
            None => String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Report {
    pub text: String,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<Diagnostic>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggested_fixes: Vec<SuggestedFix>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Diagnostic {
    pub pos: isize,

    pub text: String,

    pub code: String,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggested_fixes: Vec<SuggestedFix>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SuggestedFix {
    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_edit: Option<TextEdit>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct TextEdit {
    pub line: isize,

    pub end: isize,

    pub new_text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct URL {
    #[serde(flatten)]
    pub url: Option<url::Url>,

    #[serde(skip)]
    pub dsn: String,

    pub schema: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SqlError {
    #[serde(rename = "SQL")]
    pub sql: String,

    #[serde(rename = "Error")]
    pub error: String,
}

fn default_time() -> PrimitiveDateTime {
    PrimitiveDateTime::new(date!(0001 - 01 - 01), time!(0:00))
}

fn isize_is_zero(val: &isize) -> bool {
    *val == 0
}

fn changes_all_zero(changes: &Changes) -> bool {
    changes.applied.is_empty() && changes.pending.is_empty() && changes.error.is_none()
}

fn bool_is_zero(b: &bool) -> bool {
    !b
}
