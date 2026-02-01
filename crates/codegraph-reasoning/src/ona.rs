//! ONA (OpenNARS for Applications) client
//!
//! Communicates with ONA process via stdin/stdout.

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use tracing::{debug, info};

use crate::narsese::NarseseStatement;

/// Client for communicating with ONA process
pub struct OnaClient {
    process: Arc<Mutex<Option<Child>>>,
    host: String,
    port: u16,
}

impl OnaClient {
    /// Create a new ONA client
    pub fn new() -> Self {
        let host = env::var("ONA_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port: u16 = env::var("ONA_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(50000);

        Self {
            process: Arc::new(Mutex::new(None)),
            host,
            port,
        }
    }

    /// Start ONA process locally (for testing)
    pub fn start_local(&self, ona_path: &str) -> Result<()> {
        let child = Command::new(ona_path)
            .arg("shell")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start ONA process")?;

        let mut proc = self.process.lock().unwrap();
        *proc = Some(child);

        info!("Started local ONA process");
        Ok(())
    }

    /// Load UI ontology into ONA
    pub fn load_ontology(&self) -> Result<String> {
        let ontology = include_str!("ontology.nal");
        self.execute_batch(ontology)
    }

    /// Input statements to ONA
    pub fn input_statements(&self, statements: &[NarseseStatement]) -> Result<()> {
        for stmt in statements {
            self.execute(&stmt.to_narsese())?;
        }
        Ok(())
    }

    /// Run inference steps
    pub fn step(&self, cycles: u32) -> Result<String> {
        self.execute(&format!("{}", cycles))
    }

    /// Execute a single command
    pub fn execute(&self, command: &str) -> Result<String> {
        debug!("ONA command: {}", command);

        let mut proc = self.process.lock().unwrap();
        if let Some(ref mut child) = *proc {
            // Write command
            if let Some(ref mut stdin) = child.stdin {
                writeln!(stdin, "{}", command)?;
                stdin.flush()?;
            }

            // Read response (with timeout simulation via non-blocking check)
            if let Some(ref mut stdout) = child.stdout {
                let reader = BufReader::new(stdout);
                let mut output = String::new();

                // Read available lines (this is simplified - real impl would use async/timeout)
                for line in reader.lines().take(100) {
                    if let Ok(l) = line {
                        output.push_str(&l);
                        output.push('\n');
                        if l.contains("done with") || l.is_empty() {
                            break;
                        }
                    }
                }

                return Ok(output);
            }
        }

        // Fallback: connect to remote ONA via TCP (when running in Docker)
        self.execute_remote(command)
    }

    /// Execute command on remote ONA (Docker container)
    fn execute_remote(&self, command: &str) -> Result<String> {
        use std::io::{Read, Write as IoWrite};
        use std::net::TcpStream;

        let addr = format!("{}:{}", self.host, self.port);
        debug!("Connecting to ONA at {}", addr);

        let mut stream =
            TcpStream::connect(&addr).context(format!("Failed to connect to ONA at {}", addr))?;

        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;

        writeln!(stream, "{}", command)?;
        stream.flush()?;

        let mut response = String::new();
        let mut buf = [0u8; 4096];

        loop {
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    response.push_str(&String::from_utf8_lossy(&buf[..n]));
                    if response.contains("done with") {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => break,
                Err(e) => return Err(e.into()),
            }
        }

        Ok(response)
    }

    /// Execute batch of commands
    fn execute_batch(&self, commands: &str) -> Result<String> {
        let mut output = String::new();
        for line in commands.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            let resp = self.execute(line)?;
            output.push_str(&resp);
        }
        Ok(output)
    }

    /// Query ONA for inference results
    pub fn query(&self, question: &str) -> Result<String> {
        // Questions in Narsese end with ?
        let q = if question.ends_with('?') {
            question.to_string()
        } else {
            format!("{}?", question)
        };
        self.execute(&q)
    }

    /// Input a single Narsese statement
    pub fn input_statement(&self, statement: &str) -> Result<String> {
        self.execute(statement)
    }

    /// Reset ONA to initial state
    pub fn reset(&self) -> Result<String> {
        self.execute("*reset")
    }

    /// Flush ONA buffers
    pub fn flush(&self) -> Result<String> {
        self.execute("*flush")
    }
}

impl Default for OnaClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for OnaClient {
    fn drop(&mut self) {
        if let Ok(mut proc) = self.process.lock() {
            if let Some(mut child) = proc.take() {
                let _ = child.kill();
            }
        }
    }
}
