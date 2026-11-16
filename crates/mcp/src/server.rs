//! The rmcp service: registers the six tools and translates between protocol types and
//! the handlers in `tools.rs`.
//!
//! Handlers are synchronous and can take seconds, so each call runs on a blocking
//! thread. The runner slot sits behind a mutex that is never held across an await.

use crate::errors::{ErrorCategory, ToolError};
use crate::runner::RunnerSlot;
use crate::schemas::{
    ExplainVerdictArgs, FindGhostArgs, FitComplexityArgs, RunReferenceArgs, ShrinkInputArgs,
    ValidatePackArgs,
};
use crate::tools;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, ContentBlock, Implementation, InitializeResult, ServerCapabilities,
};
use rmcp::{tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// The server's instructions. They name the sandbox-only code tools.
const INSTRUCTIONS: &str = "Counterspell pack tools. validate_pack, explain_verdict and fit_complexity are pure and never run code. run_reference, find_ghost_input and shrink_input run pack code only in the separate rootless sandbox, and they fail closed when it is not configured.";

/// The MCP server. It holds the workspace root, the pinned image digest and the runner slot.
#[derive(Clone)]
pub struct CounterspellServer {
    root: Arc<PathBuf>,
    image_digest: Arc<Option<String>>,
    runner: Arc<Mutex<RunnerSlot>>,
}

/// Turns a finished handler call into a tool result. Errors keep their stable category.
fn to_result(outcome: Result<Result<Value, ToolError>, tokio::task::JoinError>) -> CallToolResult {
    match outcome {
        Ok(Ok(value)) => CallToolResult::success(vec![ContentBlock::text(value.to_string())]),
        Ok(Err(error)) => {
            CallToolResult::error(vec![ContentBlock::text(error.to_json().to_string())])
        }
        Err(_) => {
            let error = ToolError {
                category: ErrorCategory::Internal,
                message: "the tool task stopped unexpectedly".to_string(),
            };
            CallToolResult::error(vec![ContentBlock::text(error.to_json().to_string())])
        }
    }
}

#[tool_router]
impl CounterspellServer {
    /// Builds the server. `image_digest` is `None` when the pinned catalogue is unreadable.
    #[must_use]
    pub fn new(root: PathBuf, image_digest: Option<String>, slot: RunnerSlot) -> Self {
        Self {
            root: Arc::new(root),
            image_digest: Arc::new(image_digest),
            runner: Arc::new(Mutex::new(slot)),
        }
    }

    /// Checks a pack's files without running any code.
    #[tool(
        name = "validate_pack",
        description = "Static checks on a Grimoire pack: layout, metadata, schema, samples, hints and known ghosts. Never runs code.",
        annotations(read_only_hint = true)
    )]
    async fn validate_pack(
        &self,
        Parameters(args): Parameters<ValidatePackArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.root.clone();
        Ok(to_result(
            tokio::task::spawn_blocking(move || tools::validate_pack(&root, &args)).await,
        ))
    }

    /// Explains one observed failure with the engine's Bug Autopsy. Never runs code.
    #[tool(
        name = "explain_verdict",
        description = "Explains one observed failure (wrong, crash or timeout) with the Bug Autopsy and any authored known-ghost explanation. Never runs code.",
        annotations(read_only_hint = true)
    )]
    async fn explain_verdict(
        &self,
        Parameters(args): Parameters<ExplainVerdictArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.root.clone();
        Ok(to_result(
            tokio::task::spawn_blocking(move || tools::explain_verdict(&root, &args)).await,
        ))
    }

    /// Fits supplied timing samples to a growth class. Never runs code.
    #[tool(
        name = "fit_complexity",
        description = "Fits caller-supplied timing samples to a growth class and compares it with a declared class. Never runs code and never benchmarks.",
        annotations(read_only_hint = true)
    )]
    async fn fit_complexity(
        &self,
        Parameters(args): Parameters<FitComplexityArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.root.clone();
        Ok(to_result(
            tokio::task::spawn_blocking(move || tools::fit_complexity(&root, &args)).await,
        ))
    }

    /// Runs the reference on one input inside the sandbox.
    #[tool(
        name = "run_reference",
        description = "Runs the trusted reference solution on one input inside the separate rootless sandbox. Requires an approved sandbox; never runs on the host."
    )]
    async fn run_reference(
        &self,
        Parameters(args): Parameters<RunReferenceArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.root.clone();
        let runner = self.runner.clone();
        Ok(to_result(
            tokio::task::spawn_blocking(move || {
                let mut slot = lock(&runner, &root)?;
                tools::run_reference(&root, &mut slot, &args)
            })
            .await,
        ))
    }

    /// Hunts for a failing input inside the sandbox and returns a minimized counterexample.
    #[tool(
        name = "find_ghost_input",
        description = "Searches for a failing input for an incorrect program and shrinks it to a minimized counterexample found within the budget. Runs code only inside the separate rootless sandbox; never on the host."
    )]
    async fn find_ghost_input(
        &self,
        Parameters(args): Parameters<FindGhostArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.root.clone();
        let runner = self.runner.clone();
        let digest = self.image_digest.clone();
        Ok(to_result(
            tokio::task::spawn_blocking(move || {
                let digest = digest.as_deref().ok_or_else(|| {
                    ToolError::new(
                        ErrorCategory::SandboxNotConfigured,
                        "the pinned language image is unavailable",
                        &root,
                    )
                })?;
                let mut slot = lock(&runner, &root)?;
                tools::find_ghost_input(&root, &mut slot, digest, &args)
            })
            .await,
        ))
    }

    /// Shrinks a failing input inside the sandbox, keeping the same failure.
    #[tool(
        name = "shrink_input",
        description = "Shrinks a failing input while it keeps the same wrong-answer or crash failure, judging each probe in the separate rootless sandbox. Never runs on the host."
    )]
    async fn shrink_input(
        &self,
        Parameters(args): Parameters<ShrinkInputArgs>,
    ) -> Result<CallToolResult, McpError> {
        let root = self.root.clone();
        let runner = self.runner.clone();
        Ok(to_result(
            tokio::task::spawn_blocking(move || {
                let mut slot = lock(&runner, &root)?;
                tools::shrink_input(&root, &mut slot, &args)
            })
            .await,
        ))
    }
}

/// Locks the runner slot. A poisoned lock is an internal error, not a silent retry.
fn lock<'a>(
    runner: &'a Mutex<RunnerSlot>,
    root: &std::path::Path,
) -> Result<std::sync::MutexGuard<'a, RunnerSlot>, ToolError> {
    runner.lock().map_err(|_| {
        ToolError::new(
            ErrorCategory::Internal,
            "the runner lock is unavailable",
            root,
        )
    })
}

// The macro generates an async trait method. `get_info` is synchronous, so the lint is allowed here.
#[allow(clippy::unused_async_trait_impl)]
#[tool_handler]
impl ServerHandler for CounterspellServer {
    fn get_info(&self) -> InitializeResult {
        InitializeResult::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::from_build_env())
            .with_instructions(INSTRUCTIONS)
    }
}
