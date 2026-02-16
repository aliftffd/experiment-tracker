use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};

use crate::platform;

/// Docker container state
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerState {
    Building,
    Running,
    Exited(i32), // exit code
    Failed(String),
}

/// A tracked Docker container
#[derive(Debug)]
pub struct TrackedContainer {
    pub container_id: String,
    pub run_id: i64,
    pub state: ContainerState,
    pub image: String,
    pub command: String,
    child: Option<Child>,
}

/// Docker availability info
#[derive(Debug, Clone)]
pub struct DockerInfo {
    pub installed: bool,
    pub running: bool,
    pub gpu_support: bool,
    pub version: String,
}

/// Manages Docker containers for experiment execution
pub struct DockerManager {
    docker_path: String,
    pub containers: HashMap<i64, TrackedContainer>, // run_id → container
}

impl DockerManager {
    /// Create a new Docker manager. Returns None if Docker is not available.
    pub fn new() -> Option<Self> {
        let path = platform::find_docker()?;
        Some(Self {
            docker_path: path.to_string_lossy().to_string(),
            containers: HashMap::new(),
        })
    }

    /// Check Docker installation and capabilities
    pub fn check_health(&self) -> DockerInfo {
        // Check version
        let version = Command::new(&self.docker_path)
            .args(["--version"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        let installed = !version.is_empty();

        // Check if daemon is running
        let running = Command::new(&self.docker_path)
            .args(["info"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        // Check GPU support
        let gpu_support = if running {
            Command::new(&self.docker_path)
                .args([
                    "run",
                    "--rm",
                    "--gpus",
                    "all",
                    "nvidia/cuda:12.1.0-base-ubuntu22.04",
                    "nvidia-smi",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        } else {
            false
        };

        DockerInfo {
            installed,
            running,
            gpu_support,
            version,
        }
    }

    /// Run a training container
    pub fn run_container(
        &mut self,
        run_id: i64,
        image: &str,
        command: &str,
        host_output_dir: &str,
        container_workdir: &str,
        use_gpu: bool,
        env_vars: &HashMap<String, String>,
    ) -> Result<String> {
        let host_path = std::path::Path::new(host_output_dir);
        let docker_host_path = platform::to_docker_path(host_path);

        let mut args = vec![
            "run".to_string(),
            "--rm".to_string(),
            "-d".to_string(), // detached mode
            "-v".to_string(),
            format!("{}:{}", docker_host_path, container_workdir),
        ];

        // GPU access
        if use_gpu {
            args.push("--gpus".to_string());
            args.push("all".to_string());
        }

        // Environment variables
        for (key, value) in env_vars {
            args.push("-e".to_string());
            args.push(format!("{}={}", key, value));
        }

        // Image and command
        args.push(image.to_string());
        for part in command.split_whitespace() {
            args.push(part.to_string());
        }

        let output = Command::new(&self.docker_path)
            .args(&args)
            .output()
            .with_context(|| "Failed to start Docker container")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Docker run failed: {}", stderr.trim());
        }

        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

        self.containers.insert(
            run_id,
            TrackedContainer {
                container_id: container_id.clone(),
                run_id,
                state: ContainerState::Running,
                image: image.to_string(),
                command: command.to_string(),
                child: None,
            },
        );

        Ok(container_id)
    }

    /// Check status of all tracked containers
    pub fn poll_containers(&mut self) -> Vec<(i64, ContainerState)> {
        let mut updates = Vec::new();

        let run_ids: Vec<i64> = self.containers.keys().cloned().collect();

        for run_id in run_ids {
            if let Some(container) = self.containers.get(&run_id) {
                if container.state != ContainerState::Running {
                    continue; // already finished
                }

                let container_id = container.container_id.clone();

                // Check if container is still running
                if let Ok(output) = Command::new(&self.docker_path)
                    .args(["inspect", "--format", "{{.State.Status}}", &container_id])
                    .output()
                {
                    let status = String::from_utf8_lossy(&output.stdout).trim().to_string();

                    let new_state = match status.as_str() {
                        "running" => ContainerState::Running,
                        "exited" => {
                            // Get exit code
                            let exit_code = self.get_exit_code(&container_id).unwrap_or(-1);
                            ContainerState::Exited(exit_code)
                        }
                        other => ContainerState::Failed(format!("Unexpected state: {}", other)),
                    };

                    if new_state != ContainerState::Running {
                        updates.push((run_id, new_state.clone()));
                        if let Some(c) = self.containers.get_mut(&run_id) {
                            c.state = new_state;
                        }
                    }
                }
            }
        }

        updates
    }

    /// Stop a running container
    pub fn stop_container(&mut self, run_id: i64) -> Result<()> {
        if let Some(container) = self.containers.get_mut(&run_id) {
            Command::new(&self.docker_path)
                .args(["stop", &container.container_id])
                .output()
                .with_context(|| "Failed to stop container")?;

            container.state = ContainerState::Exited(137); // SIGKILL exit code
        }
        Ok(())
    }

    /// Kill a container immediately
    pub fn kill_container(&mut self, run_id: i64) -> Result<()> {
        if let Some(container) = self.containers.get_mut(&run_id) {
            Command::new(&self.docker_path)
                .args(["kill", &container.container_id])
                .output()
                .with_context(|| "Failed to kill container")?;

            container.state = ContainerState::Exited(137);
        }
        Ok(())
    }

    /// Get logs from a container
    pub fn get_logs(&self, run_id: i64, tail: usize) -> Result<String> {
        if let Some(container) = self.containers.get(&run_id) {
            let output = Command::new(&self.docker_path)
                .args(["logs", "--tail", &tail.to_string(), &container.container_id])
                .output()
                .with_context(|| "Failed to get container logs")?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            Ok(format!("{}{}", stdout, stderr))
        } else {
            anyhow::bail!("No container found for run {}", run_id)
        }
    }

    /// Build a Docker image from a Dockerfile
    pub fn build_image(&self, dockerfile_dir: &str, image_tag: &str) -> Result<Child> {
        let child = Command::new(&self.docker_path)
            .args(["build", "-t", image_tag, dockerfile_dir])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| "Failed to start Docker build")?;

        Ok(child)
    }

    /// List available Docker images
    pub fn list_images(&self) -> Result<Vec<String>> {
        let output = Command::new(&self.docker_path)
            .args(["images", "--format", "{{.Repository}}:{{.Tag}}"])
            .output()
            .with_context(|| "Failed to list Docker images")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let images: Vec<String> = stdout
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.contains("<none>"))
            .collect();

        Ok(images)
    }

    /// Stop all running containers (cleanup on exit)
    pub fn stop_all(&mut self) {
        let run_ids: Vec<i64> = self.containers.keys().cloned().collect();
        for run_id in run_ids {
            let _ = self.stop_container(run_id);
        }
    }

    /// Get exit code of a container
    fn get_exit_code(&self, container_id: &str) -> Result<i32> {
        let output = Command::new(&self.docker_path)
            .args(["inspect", "--format", "{{.State.ExitCode}}", container_id])
            .output()?;

        let code_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let code = code_str.parse::<i32>().unwrap_or(-1);
        Ok(code)
    }

    /// Check if a container is tracked and running for a given run
    pub fn is_running(&self, run_id: i64) -> bool {
        self.containers
            .get(&run_id)
            .map(|c| c.state == ContainerState::Running)
            .unwrap_or(false)
    }
}
