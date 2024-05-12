use crate::util::NonEmptyString;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::env;
use std::marker::PhantomData;
use std::process::{Command, Stdio};
use strum::Display;

use crate::atlas_models::{MigrateApply, MigrateDown, SchemaApply};

pub struct Client {
    exec_path: NonEmptyString,
    working_dir: Option<String>,
}
impl Client {
    pub fn new(working_dir: Option<&str>, exec_path: &str) -> anyhow::Result<Self> {
        if exec_path.is_empty() {
            return Err(anyhow!("exec_path cannot be empty"));
        }

        let exec_path = match which::which(exec_path) {
            Err(e) => return Err(anyhow!("looking up atlas-cli: {}", e)),
            Ok(path) => path
                .to_str()
                .ok_or(anyhow!("path to atlas-cli is not valid utf-8"))?
                .to_string(),
        };

        if let Some(dir) = working_dir {
            if dir.is_empty() {
                return Err(anyhow!("working_dir cannot be empty when it is not None"));
            }

            if let Err(e) = std::fs::metadata(dir) {
                return Err(anyhow!(
                    "failed to initialize Atlas with working dir {}: {}",
                    dir,
                    e
                ));
            }
        }

        Ok(Self {
            exec_path: exec_path.try_into()?,
            working_dir: working_dir.map(|v| v.to_string()),
        })
    }

    pub fn with_work_dir(
        &mut self,
        dir: Option<&str>,
        f: fn(&mut Self) -> anyhow::Result<()>,
    ) -> anyhow::Result<()> {
        self.working_dir = dir.map(|v| v.to_string());
        f(self)
    }

    pub fn login(&self, params: LoginParams) -> anyhow::Result<()> {
        if params.token.is_empty() {
            return Err(anyhow!("token cannot be empty"));
        }

        self.run_command(vec!["login", "--token", &params.token])?;

        Ok(())
    }

    pub fn logout(&self) -> anyhow::Result<()> {
        self.run_command(vec!["logout"])?;

        Ok(())
    }

    pub fn migrate_push(&self, params: MigratePushParams) -> anyhow::Result<String> {
        let mut args = vec!["migrate", "push"];

        if let Some(ref dev_url) = params.dev_url {
            args.append(&mut vec!["--dev-url", dev_url.as_str()]);
        }

        if let Some(ref dir_url) = params.dir_url {
            args.append(&mut vec!["--dir", dir_url.as_str()])
        }

        if let Some(ref dir_format) = params.dir_format {
            args.append(&mut vec!["--dir-format", dir_format.as_str()])
        }

        if let Some(ref lock_timeout) = params.lock_timeout {
            args.append(&mut vec!["--lock-timeout", lock_timeout.as_str()])
        }

        let json: String;
        if let Some(ref context) = params.context {
            json = serde_json::to_string(context)
                .map_err(|e| anyhow!("failed to serialize run context: {}", e))?;

            args.append(&mut vec!["--config", &json])
        }

        if let Some(ref config_url) = params.config_url {
            args.append(&mut vec!["--config", config_url.as_str()])
        }

        if let Some(ref env) = params.env {
            args.append(&mut vec!["--env", env.as_str()])
        }

        let tag_arg: String;
        match params.tag {
            Some(ref tag) => {
                tag_arg = format!("{}:{}", &params.name, tag);
                args.push(&tag_arg);
            }
            None => args.push(&params.name),
        }

        self.run_command(args)
    }

    pub fn migrate_apply(&self, params: MigrateApplyParams) -> anyhow::Result<MigrateApply> {
        first_result(self.migrate_apply_slice(params))
    }

    pub fn migrate_apply_slice(
        &self,
        params: MigrateApplyParams,
    ) -> anyhow::Result<Vec<MigrateApply>> {
        let mut args = vec!["migrate", "apply", "--format", "{{ json . }}"];

        if let Some(ref env) = params.env {
            args.append(&mut vec!["--env", env.as_str()]);
        }

        if let Some(ref config_url) = params.config_url {
            args.append(&mut vec!["--config", config_url.as_str()])
        }

        let json: String;
        if let Some(ref ctx) = params.context {
            json = serde_json::to_string(ctx)
                .map_err(|e| anyhow!("failed to serialize DeployRunContext: {}", e))?;

            args.append(&mut vec!["--context", &json])
        }

        if let Some(ref url) = params.url {
            args.append(&mut vec!["--url", url.as_str()])
        }

        if let Some(ref dir_url) = params.dir_url {
            args.append(&mut vec!["--dir", dir_url.as_str()])
        }

        if params.allow_dirty {
            args.append(&mut vec!["--allow-dirty"])
        }

        if params.dry_run {
            args.append(&mut vec!["--dry-run"])
        }

        if let Some(ref revisions_schema) = params.revisions_schema {
            args.append(&mut vec!["--revisions-schema", revisions_schema.as_str()])
        }

        if let Some(ref baseline_version) = params.baseline_version {
            args.append(&mut vec!["baseline", baseline_version.as_str()])
        }

        if let Some(ref tx_mode) = params.tx_mode {
            args.append(&mut vec!["--tx-mode", tx_mode.as_str()])
        }

        let exec_order_str: String;
        if let Some(ref exec_order) = params.exec_order {
            exec_order_str = exec_order.to_string();
            args.append(&mut vec!["--exec-order", &exec_order_str])
        }

        let amount_str: String;
        if params.amount > 0 {
            amount_str = params.amount.to_string();
            args.append(&mut vec![&amount_str])
        }

        let var_args = params.vars.as_args();

        args.append(&mut var_args.iter().map(|s| s.as_str()).collect::<Vec<&str>>());

        let res_str = self.run_command(args)?;

        serde_json::from_str(&res_str).map_err(|e| {
            anyhow!(
                "failed to deserialize run_command response {}: {}",
                res_str,
                e
            )
        })
    }

    pub fn migrate_down(&self, params: MigrateDownParams) -> anyhow::Result<MigrateDown> {
        let mut args = vec!["migrate", "down", "--format", "{{ json .}}"];

        if let Some(ref env) = params.env {
            args.append(&mut vec!["--env", env.as_str()]);
        }

        if let Some(ref config_url) = params.config_url {
            args.append(&mut vec!["--config", config_url.as_str()]);
        }

        if let Some(ref dev_url) = params.dev_url {
            args.append(&mut vec!["--dev-url", dev_url.as_str()]);
        }

        let ctx_json: String;
        if let Some(ref ctx) = params.context {
            ctx_json = serde_json::to_string(ctx)
                .map_err(|e| anyhow!("failed to serialize DeployRunContext: {}", e))?;

            args.append(&mut vec!["--context", &ctx_json]);
        }

        if let Some(ref url) = params.url {
            args.append(&mut vec!["--url", url.as_str()]);
        }

        if let Some(ref dir_url) = params.dir_url {
            args.append(&mut vec!["--dir", dir_url.as_str()]);
        }

        if let Some(ref revisions_schema) = params.revisions_schema {
            args.append(&mut vec!["--revisions-schema", revisions_schema.as_str()]);
        }

        if let Some(ref to_version) = params.to_version {
            args.append(&mut vec!["--to-version", to_version.as_str()]);
        }

        if let Some(ref to_tag) = params.to_tag {
            args.append(&mut vec!["--to-tag", to_tag.as_str()]);
        }

        let amount_str: String;
        if params.amount > 0 {
            amount_str = params.amount.to_string();
            args.push(&amount_str);
        }

        let var_args = params.vars.as_args();

        args.append(&mut var_args.iter().map(|s| s.as_str()).collect::<Vec<&str>>());

        // TODO: result should be stderr if present

        let result_json = self.run_command(args)?;
        first_result(
            serde_json::from_str(&result_json)
                .map_err(|e| anyhow!("failed to deserialize MigrateDown: {}", e)),
        )
    }

    pub fn schema_apply(&self, params: SchemaApplyParams) -> anyhow::Result<SchemaApply> {
        first_result(self.schema_apply_slice(params))
    }

    pub fn schema_apply_slice(
        &self,
        params: SchemaApplyParams,
    ) -> anyhow::Result<Vec<SchemaApply>> {
        let mut args = vec!["schema", "apply", "--format", "{{ json .}}"];

        if let Some(ref env) = params.env {
            args.append(&mut vec!["--env", env.as_str()]);
        }

        if let Some(ref config_url) = params.config_url {
            args.append(&mut vec!["--config", config_url.as_str()]);
        }

        if let Some(ref url) = params.url {
            args.append(&mut vec!["--url", url.as_str()]);
        }

        if let Some(ref to) = params.to {
            args.append(&mut vec!["--to", to.as_str()]);
        }

        if params.dry_run {
            args.push("--dry-run");
        } else {
            args.push("--auto-approve");
        }

        if let Some(ref tx_mode) = params.tx_mode {
            args.append(&mut vec!["--tx-mode", tx_mode.as_str()]);
        }

        if let Some(ref dev_url) = params.dev_url {
            args.append(&mut vec!["--dev-url", dev_url.as_str()]);
        }

        let schema_joined: String;
        if !params.schema.is_empty() {
            schema_joined = params
                .schema
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(",");

            args.append(&mut vec!["--schema", &schema_joined]);
        }

        let exclude_joined: String;
        if !params.exclude.is_empty() {
            exclude_joined = params
                .exclude
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(",");

            args.append(&mut vec!["--exclude", &exclude_joined]);
        }

        let var_args = params.vars.as_args();

        args.append(&mut var_args.iter().map(|s| s.as_str()).collect::<Vec<&str>>());

        let result = self.run_command(args)?;

        serde_json::from_str(&result).map_err(|e| {
            anyhow!(
                "failed to deserialize command result {} to Vec<SchemaApply>: {}",
                result,
                e
            )
        })
    }

    pub fn schema_inspect(&self, params: SchemaInspectParams) -> anyhow::Result<String> {
        let mut args = vec!["schema", "inspect"];

        if let Some(ref env) = params.env {
            args.append(&mut vec!["--env", env.as_str()]);
        }

        if let Some(ref config_url) = params.config_url {
            args.append(&mut vec!["--config", config_url.as_str()]);
        }

        if let Some(ref url) = params.url {
            args.append(&mut vec!["--url", url.as_str()]);
        }

        if let Some(ref dev_url) = params.dev_url {
            args.append(&mut vec!["--dev-url", dev_url.as_str()]);
        }

        if let Some(ref format) = params.format {
            match format.as_str() {
                "sql" => args.append(&mut vec!["format", "{{ sql .}}"]),
                other => args.append(&mut vec!["--format", other]),
            }
        }

        let schema_joined: String;
        if !params.schema.is_empty() {
            schema_joined = params
                .schema
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(",");

            args.append(&mut vec!["--schema", &schema_joined]);
        }

        let exclude_joined: String;
        if !params.exclude.is_empty() {
            exclude_joined = params
                .exclude
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(",");

            args.append(&mut vec!["--exclude", &exclude_joined]);
        }

        self.run_command(args)
    }

    fn run_command(&self, args: Vec<&str>) -> anyhow::Result<String> {
        let mut cmd = Command::new(self.exec_path.as_str());
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        if let Some(dir) = &self.working_dir {
            cmd.current_dir(dir);
        }

        // set if not already set
        if env::var("ATLAS_NO_UPDATE_NOTIFIER").is_err() {
            cmd.env("ATLAS_NO_UPDATE_NOTIFIER", "1");
        }

        let output = cmd
            .output()
            .map_err(|e| anyhow!("failed to run cmd: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8(output.stderr)
                .map_err(|e| anyhow!("stderr included non-utf8 chars: {}", e))?
                .trim()
                .to_string();

            return Err(anyhow!(
                "cmd had non-zero exit status {}: {}",
                output.status,
                stderr,
            ));
        }

        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| anyhow!("stdout included non-utf8 chars: {}", e))?
            .trim()
            .to_string();

        Ok(stdout)
    }
}

#[derive(Debug)]
pub struct LoginParams {
    pub token: String,
}

#[derive(Debug)]
pub struct MigratePushParams {
    pub name: String,
    pub tag: Option<String>,
    pub dev_url: Option<NonEmptyString>,
    pub dir_url: Option<NonEmptyString>,
    pub dir_format: Option<NonEmptyString>,
    pub lock_timeout: Option<NonEmptyString>,
    pub context: Option<RunContext>,
    pub config_url: Option<NonEmptyString>,
    pub env: Option<NonEmptyString>,
    pub vars: Vars,
}

#[derive(Debug, Display, Deserialize, Serialize)]
pub enum TriggerType {
    #[serde(rename = "CLI")]
    #[strum(serialize = "CLI")]
    Cli,

    #[serde(rename = "KUBERNETES")]
    #[strum(serialize = "KUBERNETES")]
    Kubernetes,

    #[serde(rename = "TERRAFORM")]
    #[strum(serialize = "TERRAFORM")]
    Terraform,

    #[serde(rename = "GITHUB_ACTION")]
    #[strum(serialize = "GITHUB_ACTION")]
    GithubAction,

    #[serde(rename = "CIRCLECI_ORB")]
    #[strum(serialize = "CIRCLECI_ORB")]
    CircleCiOrb,
}

#[derive(Debug, Display, Deserialize, Serialize)]
pub enum MigrateExecOrder {
    #[serde(rename = "linear")]
    #[strum(serialize = "linear")]
    Linear,

    #[serde(rename = "linear-skip")]
    #[strum(serialize = "linear-skip")]
    LinearSkip,

    #[serde(rename = "non-linear")]
    #[strum(serialize = "non-linear")]
    NonLinear,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployRunContext {
    pub trigger_type: TriggerType,
    pub trigger_version: String,
}

#[derive(Debug)]
pub struct MigrateApplyParams {
    pub env: Option<NonEmptyString>,
    pub config_url: Option<NonEmptyString>,
    pub context: Option<DeployRunContext>,
    pub dir_url: Option<NonEmptyString>,
    pub allow_dirty: bool,
    pub url: Option<NonEmptyString>,
    pub revisions_schema: Option<NonEmptyString>,
    pub baseline_version: Option<NonEmptyString>,
    pub tx_mode: Option<NonEmptyString>,
    pub exec_order: Option<MigrateExecOrder>,
    pub amount: u64,
    pub dry_run: bool,
    pub vars: Vars,
}

#[derive(Debug)]
pub struct MigrateDownParams {
    pub env: Option<NonEmptyString>,
    pub config_url: Option<NonEmptyString>,
    pub dev_url: Option<NonEmptyString>,
    pub context: Option<DeployRunContext>,
    pub dir_url: Option<NonEmptyString>,
    pub url: Option<NonEmptyString>,
    pub revisions_schema: Option<NonEmptyString>,
    pub amount: u64,
    pub to_version: Option<NonEmptyString>,
    pub to_tag: Option<NonEmptyString>,
    pub vars: Vars,
}

#[derive(Debug)]
pub struct MigrateStatusParams {
    pub env: String,
    pub config_url: String,
    pub dir_url: String,
    pub url: String,
    pub revisions_schema: String,
    pub vars: Vars,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunContext {
    pub repo: String,

    pub path: String,

    pub branch: String,

    pub commit: String,

    pub url: String,

    pub username: String,

    #[serde(rename = "userID")]
    pub user_id: String,

    pub scm_type: String,
}

#[derive(Debug)]
pub struct MigrateLintParams {
    pub env: String,
    pub config_url: String,
    pub dev_url: String,
    pub dir_url: String,
    pub context: RunContext,
    pub web: bool,
    pub latest: u64,
    pub vars: Vars,
    pub writer: PhantomData<u64>, // TODO: io.Writer
    pub base: String,
    pub format: String,
}

#[derive(Debug)]
pub struct SchemaApplyParams {
    pub env: Option<NonEmptyString>,
    pub config_url: Option<NonEmptyString>,
    pub dev_url: Option<NonEmptyString>,
    pub dry_run: bool,
    pub tx_mode: Option<NonEmptyString>,
    pub exclude: Vec<NonEmptyString>,
    pub schema: Vec<NonEmptyString>,
    pub to: Option<NonEmptyString>,
    pub url: Option<NonEmptyString>,
    pub vars: Vars,
}

#[derive(Debug)]
pub struct SchemaInspectParams {
    pub env: Option<NonEmptyString>,
    pub config_url: Option<NonEmptyString>,
    pub dev_url: Option<NonEmptyString>,
    pub exclude: Vec<NonEmptyString>,
    pub format: Option<NonEmptyString>,
    pub schema: Vec<NonEmptyString>,
    pub url: Option<NonEmptyString>,
    pub vars: Vars,
}

#[derive(Debug)]
pub struct Vars(std::collections::HashMap<String, String>);
impl Vars {
    pub fn as_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        for (k, v) in self.0.iter() {
            args.push("--var".into());
            args.push(format!("{}={}", k, v));
        }

        args
    }
}

fn first_result<T: Clone>(result: anyhow::Result<Vec<T>>) -> anyhow::Result<T> {
    match result {
        Err(e) => Err(e),
        Ok(v) => {
            if v.len() == 1 {
                return Ok(v[0].clone());
            }

            Err(anyhow!(
                "The command returned more than one result, use Slice function instead"
            ))
        }
    }
}
