// Abstraction thực thi Process (ProcessRunner) cho AegisNode
// Đảm bảo không bao giờ sử dụng /bin/sh -c hoặc nối chuỗi shell thô (tránh lỗ hổng Command Injection)
// Tích hợp Timeout và Capture đầy đủ stdout/stderr

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use aegis_core::{AegisError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Yêu cầu thực thi lệnh hệ thống an toàn
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessRequest {
    pub program: String,
    pub args: Vec<String>,
    pub timeout_seconds: Option<u64>,
}

impl ProcessRequest {
    pub fn new(program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
            timeout_seconds: Some(10), // Mặc định timeout 10 giây
        }
    }
}

/// Kết quả trả về sau khi thực thi lệnh hệ thống
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessOutput {
    pub status_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl ProcessOutput {
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            status_code: 0,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    pub fn failure(code: i32, stderr: impl Into<String>) -> Self {
        Self {
            status_code: code,
            stdout: String::new(),
            stderr: stderr.into(),
        }
    }

    pub fn is_success(&self) -> bool {
        self.status_code == 0
    }
}

/// Interface thực thi Process bất đồng bộ
#[async_trait]
pub trait ProcessRunner: Send + Sync {
    async fn run(&self, request: ProcessRequest) -> Result<ProcessOutput>;
}

/// Implementation mặc định sử dụng tokio::process::Command có Timeout
pub struct DefaultProcessRunner;

impl DefaultProcessRunner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultProcessRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProcessRunner for DefaultProcessRunner {
    async fn run(&self, request: ProcessRequest) -> Result<ProcessOutput> {
        let timeout_secs = request.timeout_seconds.unwrap_or(10);
        let mut cmd = tokio::process::Command::new(&request.program);
        cmd.args(&request.args);

        let output_future = cmd.output();

        match tokio::time::timeout(Duration::from_secs(timeout_secs), output_future).await {
            Ok(Ok(output)) => {
                let status_code = output.status.code().unwrap_or(-1);
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                Ok(ProcessOutput {
                    status_code,
                    stdout,
                    stderr,
                })
            }
            Ok(Err(e)) => Err(AegisError::Internal(format!(
                "Failed to execute process '{}': {e}",
                request.program
            ))),
            Err(_) => Err(AegisError::Timeout(format!(
                "Process '{}' execution timed out after {timeout_secs}s",
                request.program
            ))),
        }
    }
}

/// Mock ProcessRunner phục vụ unit tests và kiểm thử không cần root / Kernel
#[derive(Default)]
pub struct MockProcessRunner {
    mock_responses: Mutex<HashMap<String, ProcessOutput>>,
}

impl MockProcessRunner {
    pub fn new() -> Self {
        Self {
            mock_responses: Mutex::new(HashMap::new()),
        }
    }

    pub fn register_response(&self, program: &str, output: ProcessOutput) {
        let mut guard = self.mock_responses.lock().unwrap();
        guard.insert(program.to_string(), output);
    }

    pub fn set_response(&self, program: &str, output: ProcessOutput) {
        self.register_response(program, output);
    }
}

#[async_trait]
impl ProcessRunner for MockProcessRunner {
    async fn run(&self, request: ProcessRequest) -> Result<ProcessOutput> {
        let guard = self.mock_responses.lock().unwrap();
        if let Some(resp) = guard.get(&request.program) {
            Ok(resp.clone())
        } else {
            Ok(ProcessOutput::success(""))
        }
    }
}
