//! Verified discovery and bounded execution for application-private engines.
//!
//! This is process hygiene, not an operating-system sandbox. A registered executable still has
//! the same filesystem and network permissions as the user who launched the application.

use crate::{read_bounded, Error, ErrorKind, Job, Result};
use command_group::{CommandGroup, GroupChild};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Write},
    ops::{Deref, DerefMut},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const MANIFEST_LIMIT: u64 = 1024 * 1024;
const MAX_ENGINES: usize = 128;
const MAX_ARGUMENTS: usize = 256;
const MAX_ARGUMENT_BYTES: usize = 256 * 1024;
const MAX_ENVIRONMENT_BYTES: usize = 64 * 1024;
const MAX_PIPE_BYTES: usize = 64 * 1024 * 1024;
const MAX_ENGINE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_TIMEOUT_MS: u64 = 24 * 60 * 60 * 1000;

/// Versioned inventory stored at `engines/manifest.json` beside an application executable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineManifest {
    /// Manifest schema version. Version 1 is currently supported.
    pub schema_version: u32,
    /// Engines bundled with this application.
    pub engines: Vec<EngineSpec>,
}

/// A bounded, hash-pinned private engine declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineSpec {
    /// Stable lowercase ASCII identifier.
    pub id: String,
    /// Engine release or adapter version.
    pub version: String,
    /// Relative executable path from the application directory.
    pub executable: PathBuf,
    /// Expected lowercase SHA-256 hex digest of the executable.
    pub sha256: String,
    /// SPDX expression or concise license reference for bundled notices.
    pub license: String,
    /// Operations, formats and limitations exposed by this engine.
    pub capabilities: Vec<crate::Capability>,
    /// Maximum runtime in milliseconds, from 1 through 86,400,000.
    pub timeout_ms: u64,
    /// Maximum request bytes sent to standard input.
    pub max_stdin_bytes: usize,
    /// Maximum captured standard output bytes.
    pub max_stdout_bytes: usize,
    /// Maximum captured standard error bytes.
    pub max_stderr_bytes: usize,
    /// Additional environment variable names this engine may receive.
    pub environment_allowlist: Vec<String>,
    /// Optional arguments that make the engine report whether it is healthy.
    pub health_check_args: Option<Vec<String>>,
}

/// Arguments, input bytes and explicitly permitted environment values for one engine call.
#[derive(Debug, Clone, Default)]
pub struct EngineInvocation {
    /// Structured arguments passed directly without a shell.
    pub args: Vec<String>,
    /// Bounded protocol input written to standard input.
    pub stdin: Vec<u8>,
    /// Additional values whose keys appear in the engine's allowlist.
    pub environment: BTreeMap<String, String>,
}

/// Bounded output and process exit status returned by a private engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineOutput {
    /// Process exit code, or -1 when the platform does not provide one.
    pub exit_code: i32,
    /// Captured standard output bytes.
    pub stdout: Vec<u8>,
    /// Captured standard error bytes.
    pub stderr: Vec<u8>,
}

/// An executable whose descriptor and SHA-256 digest have been checked.
#[derive(Debug, Clone)]
pub struct ResolvedEngine {
    spec: EngineSpec,
    executable: PathBuf,
}

impl ResolvedEngine {
    /// Return the verified descriptor.
    pub fn spec(&self) -> &EngineSpec {
        &self.spec
    }

    /// Return the absolute executable path that was verified.
    pub fn executable(&self) -> &Path {
        &self.executable
    }

    /// Run this engine with structured arguments, a private temporary directory, bounded pipes,
    /// environment allowlisting, a wall-clock timeout and process-tree cancellation.
    pub fn run(&self, invocation: EngineInvocation, job: &Job) -> Result<EngineOutput> {
        validate_invocation(&self.spec, &invocation)?;
        verify_executable(&self.spec, &self.executable)?;

        let private_dir = tempfile::tempdir()?;
        let mut command = Command::new(&self.executable);
        command
            .args(&invocation.args)
            .current_dir(private_dir.path())
            .env_clear()
            .env("HOME", private_dir.path())
            .env("TMPDIR", private_dir.path())
            .env("TMP", private_dir.path())
            .env("TEMP", private_dir.path())
            .env("LANG", "C")
            .env("LC_ALL", "C")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        if let Some(system_root) = std::env::var_os("SystemRoot") {
            command.env("SystemRoot", system_root);
        }
        for (key, value) in invocation.environment {
            command.env(key, value);
        }
        #[cfg(windows)]
        let process = {
            let mut group = command.group();
            group.kill_on_drop(true);
            group.spawn()
        };
        #[cfg(not(windows))]
        let process = command.group().spawn();
        let mut child = ManagedGroup {
            child: process
                .map_err(|error| Error::engine(format!("Could not start engine: {error}")))?,
            reaped: false,
        };
        let overflow = Arc::new(AtomicBool::new(false));
        let stdout = spawn_reader(
            child.inner().stdout.take().ok_or_else(|| {
                Error::new(ErrorKind::Internal, "Engine stdout pipe was not created")
            })?,
            self.spec.max_stdout_bytes,
            overflow.clone(),
        )?;
        let stderr = spawn_reader(
            child.inner().stderr.take().ok_or_else(|| {
                Error::new(ErrorKind::Internal, "Engine stderr pipe was not created")
            })?,
            self.spec.max_stderr_bytes,
            overflow.clone(),
        )?;
        let mut stdin =
            child.inner().stdin.take().ok_or_else(|| {
                Error::new(ErrorKind::Internal, "Engine stdin pipe was not created")
            })?;
        let input = invocation.stdin;
        let writer = thread::Builder::new()
            .name("localwright-engine-input".into())
            .spawn(move || {
                let result = stdin.write_all(&input);
                drop(stdin);
                result
            })
            .map_err(|error| {
                Error::engine(format!("Could not start engine input worker: {error}"))
            })?;

        let start = Instant::now();
        let deadline = Duration::from_millis(self.spec.timeout_ms);
        let termination = loop {
            if let Err(error) = job.check() {
                terminate_group(&mut child);
                break Some(error);
            }
            if overflow.load(Ordering::Relaxed) {
                terminate_group(&mut child);
                break Some(Error::new(
                    ErrorKind::Limit,
                    "Engine output exceeded its declared byte limit",
                ));
            }
            if start.elapsed() >= deadline {
                terminate_group(&mut child);
                break Some(Error::new(
                    ErrorKind::Engine,
                    format!("Engine exceeded its {} ms timeout", self.spec.timeout_ms),
                ));
            }
            match child.try_wait() {
                Ok(Some(_)) => break None,
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(error) => {
                    terminate_group(&mut child);
                    break Some(Error::engine(format!("Could not monitor engine: {error}")));
                }
            }
        };
        let status = child.wait().map_err(|error| {
            Error::engine(format!("Could not reap engine process group: {error}"))
        })?;
        let writer_result = writer
            .join()
            .map_err(|_| Error::new(ErrorKind::Internal, "Engine input worker panicked"))?;
        let stdout = join_reader(stdout)?;
        let stderr = join_reader(stderr)?;
        drop(private_dir);

        if let Some(error) = termination {
            return Err(error);
        }
        writer_result
            .map_err(|error| Error::engine(format!("Engine did not accept its input: {error}")))?;
        if overflow.load(Ordering::Relaxed) {
            return Err(Error::new(
                ErrorKind::Limit,
                "Engine output exceeded its declared byte limit",
            ));
        }
        if !status.success() {
            let detail = String::from_utf8_lossy(&stderr);
            let detail = detail.trim();
            let message = if detail.is_empty() {
                format!("Engine exited with status {status}")
            } else {
                format!("Engine exited with status {status}: {detail}")
            };
            return Err(Error::engine(message));
        }
        Ok(EngineOutput {
            exit_code: status.code().unwrap_or(-1),
            stdout,
            stderr,
        })
    }
}

/// Registered, currently available engines and explicit reasons unavailable entries were refused.
#[derive(Debug, Default)]
pub struct EngineRuntime {
    app_root: PathBuf,
    available: BTreeMap<String, ResolvedEngine>,
    unavailable: BTreeMap<String, String>,
}

impl EngineRuntime {
    /// Discover and hash-verify descriptors at `<app_root>/engines/manifest.json`.
    pub fn discover(app_root: &Path) -> Result<Self> {
        let app_root = fs::canonicalize(app_root)?;
        if !app_root.is_dir() {
            return Err(Error::input("Application engine root must be a directory"));
        }
        let manifest_path = app_root.join("engines").join("manifest.json");
        let manifest_meta = fs::symlink_metadata(&manifest_path)?;
        if !manifest_meta.is_file() || manifest_meta.file_type().is_symlink() {
            return Err(Error::input(
                "Engine manifest must be a regular non-symlink file",
            ));
        }
        let bytes = read_bounded(&manifest_path, MANIFEST_LIMIT)?;
        let manifest: EngineManifest = serde_json::from_slice(&bytes)?;
        if manifest.schema_version != 1 || manifest.engines.len() > MAX_ENGINES {
            return Err(Error::input(
                "Unsupported engine manifest version or engine count",
            ));
        }
        let mut runtime = Self {
            app_root: app_root.clone(),
            ..Self::default()
        };
        let mut ids = BTreeSet::new();
        for spec in manifest.engines {
            validate_spec(&spec)?;
            if !ids.insert(spec.id.clone()) {
                return Err(Error::input(format!("Duplicate engine ID {:?}", spec.id)));
            }
            match resolve_relative(&app_root, spec.clone()) {
                Ok(engine) => {
                    runtime.available.insert(spec.id, engine);
                }
                Err(error) => {
                    runtime.unavailable.insert(spec.id, error.message);
                }
            }
        }
        Ok(runtime)
    }

    /// Discover engines relative to the running application's executable directory.
    pub fn discover_current_executable() -> Result<Self> {
        let executable = std::env::current_exe()?;
        let root = executable
            .parent()
            .ok_or_else(|| Error::input("Current executable has no parent directory"))?;
        Self::discover(root)
    }

    /// Verify an explicitly trusted absolute executable against a declared descriptor.
    ///
    /// The path is never searched through `PATH`. The caller is responsible for deciding which
    /// absolute paths it trusts; the descriptor digest is still checked before each launch.
    pub fn resolve_trusted_path(spec: EngineSpec, executable: &Path) -> Result<ResolvedEngine> {
        validate_spec(&spec)?;
        if !executable.is_absolute() {
            return Err(Error::input(
                "Trusted engine executable path must be absolute",
            ));
        }
        verify_executable(&spec, executable)?;
        Ok(ResolvedEngine {
            spec,
            executable: executable.to_owned(),
        })
    }

    /// Return the application directory used for relative discovery.
    pub fn app_root(&self) -> &Path {
        &self.app_root
    }

    /// List verified engines in stable identifier order.
    pub fn available(&self) -> impl Iterator<Item = &ResolvedEngine> {
        self.available.values()
    }

    /// List unavailable engine IDs and the verification reason.
    pub fn unavailable(&self) -> &BTreeMap<String, String> {
        &self.unavailable
    }

    /// Return the verified engine descriptor, if available.
    pub fn get(&self, id: &str) -> Option<&ResolvedEngine> {
        self.available.get(id)
    }

    /// Launch an available engine by registered ID.
    pub fn run(&self, id: &str, invocation: EngineInvocation, job: &Job) -> Result<EngineOutput> {
        let engine = self.available.get(id).ok_or_else(|| {
            Error::engine(
                self.unavailable
                    .get(id)
                    .map(|reason| format!("Engine {id:?} is unavailable: {reason}"))
                    .unwrap_or_else(|| format!("Engine {id:?} is not registered")),
            )
        })?;
        engine.run(invocation, job)
    }

    /// Execute descriptor-supplied health-check arguments using the same bounds as normal jobs.
    pub fn health_check(&self, id: &str, job: &Job) -> Result<EngineOutput> {
        let engine = self.available.get(id).ok_or_else(|| {
            Error::engine(format!("Engine {id:?} is unavailable or not registered"))
        })?;
        let args =
            engine.spec.health_check_args.clone().ok_or_else(|| {
                Error::input("This engine does not declare health-check arguments")
            })?;
        engine.run(
            EngineInvocation {
                args,
                ..EngineInvocation::default()
            },
            job,
        )
    }
}

fn validate_spec(spec: &EngineSpec) -> Result<()> {
    if !valid_id(&spec.id)
        || spec.version.trim().is_empty()
        || spec.version.len() > 128
        || spec.license.trim().is_empty()
        || spec.license.len() > 256
    {
        return Err(Error::input(
            "Invalid engine ID, version or license reference",
        ));
    }
    if !spec.executable.is_relative()
        || spec
            .executable
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(Error::input(
            "Engine executable must be a safe application-relative path",
        ));
    }
    if spec.sha256.len() != 64 || !spec.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(Error::input(
            "Engine SHA-256 must contain exactly 64 hexadecimal characters",
        ));
    }
    if !(1..=MAX_TIMEOUT_MS).contains(&spec.timeout_ms)
        || spec.max_stdin_bytes > MAX_PIPE_BYTES
        || spec.max_stdout_bytes > MAX_PIPE_BYTES
        || spec.max_stderr_bytes > MAX_PIPE_BYTES
        || spec.environment_allowlist.len() > 64
        || spec.capabilities.len() > 256
    {
        return Err(Error::input("Engine limits exceed supported bounds"));
    }
    let mut environment_names = BTreeSet::new();
    for name in &spec.environment_allowlist {
        if !valid_environment_name(name)
            || reserved_environment_name(name)
            || !environment_names.insert(name.to_ascii_uppercase())
        {
            return Err(Error::input(format!(
                "Invalid or reserved engine environment name {name:?}"
            )));
        }
    }
    if let Some(args) = &spec.health_check_args {
        validate_arguments(args)?;
    }
    Ok(())
}

fn valid_id(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && id.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
}

fn valid_environment_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(b'A'..=b'Z' | b'_'))
        && bytes.all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        && name.len() <= 128
}

fn reserved_environment_name(name: &str) -> bool {
    matches!(
        name.to_ascii_uppercase().as_str(),
        "PATH"
            | "HOME"
            | "TMP"
            | "TEMP"
            | "TMPDIR"
            | "SYSTEMROOT"
            | "LD_PRELOAD"
            | "LD_LIBRARY_PATH"
            | "DYLD_INSERT_LIBRARIES"
            | "DYLD_LIBRARY_PATH"
    )
}

fn validate_arguments(args: &[String]) -> Result<()> {
    if args.len() > MAX_ARGUMENTS
        || args
            .iter()
            .any(|arg| arg.len() > 64 * 1024 || arg.contains('\0'))
        || args.iter().map(String::len).sum::<usize>() > MAX_ARGUMENT_BYTES
    {
        return Err(Error::input("Engine arguments exceed supported bounds"));
    }
    Ok(())
}

fn validate_invocation(spec: &EngineSpec, invocation: &EngineInvocation) -> Result<()> {
    validate_arguments(&invocation.args)?;
    if invocation.stdin.len() > spec.max_stdin_bytes {
        return Err(Error::new(
            ErrorKind::Limit,
            "Engine input exceeds its declared byte limit",
        ));
    }
    let allowed = spec.environment_allowlist.iter().collect::<BTreeSet<_>>();
    let mut total = 0usize;
    for (key, value) in &invocation.environment {
        if !allowed.contains(key) || !valid_environment_name(key) || value.contains('\0') {
            return Err(Error::input(format!(
                "Environment variable {key:?} is not permitted"
            )));
        }
        total = total
            .checked_add(key.len())
            .and_then(|bytes| bytes.checked_add(value.len()))
            .filter(|bytes| *bytes <= MAX_ENVIRONMENT_BYTES)
            .ok_or_else(|| Error::new(ErrorKind::Limit, "Engine environment exceeds 64 KiB"))?;
    }
    Ok(())
}

fn resolve_relative(app_root: &Path, spec: EngineSpec) -> Result<ResolvedEngine> {
    let root = fs::canonicalize(app_root)?;
    let candidate = root.join(&spec.executable);
    let mut component_path = root.clone();
    for component in spec.executable.components() {
        if let Component::Normal(name) = component {
            component_path.push(name);
            if fs::symlink_metadata(&component_path)?
                .file_type()
                .is_symlink()
            {
                return Err(Error::input("Engine executable path contains a symlink"));
            }
        }
    }
    let executable = fs::canonicalize(&candidate)?;
    if !executable.starts_with(&root) {
        return Err(Error::input(
            "Engine executable resolves outside the application directory",
        ));
    }
    verify_executable(&spec, &executable)?;
    Ok(ResolvedEngine { spec, executable })
}

fn verify_executable(spec: &EngineSpec, executable: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(executable)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(Error::input(
            "Engine executable must be a regular non-symlink file",
        ));
    }
    let actual = sha256_file(executable)?;
    if !actual.eq_ignore_ascii_case(&spec.sha256) {
        return Err(Error::engine(format!(
            "SHA-256 verification failed for engine {:?}",
            spec.id
        )));
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String> {
    if fs::metadata(path)?.len() > MAX_ENGINE_BYTES {
        return Err(Error::new(
            ErrorKind::Limit,
            "Engine executable exceeds 2 GiB",
        ));
    }
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn spawn_reader<R: Read + Send + 'static>(
    mut reader: R,
    limit: usize,
    overflow: Arc<AtomicBool>,
) -> Result<JoinHandle<std::io::Result<Vec<u8>>>> {
    thread::Builder::new()
        .name("localwright-engine-output".into())
        .spawn(move || {
            let mut output = Vec::with_capacity(limit.min(64 * 1024));
            let mut buffer = [0u8; 16 * 1024];
            loop {
                let count = reader.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                let remaining = limit.saturating_sub(output.len());
                output.extend_from_slice(&buffer[..count.min(remaining)]);
                if count > remaining {
                    overflow.store(true, Ordering::Relaxed);
                }
            }
            Ok(output)
        })
        .map_err(|error| Error::engine(format!("Could not start engine output worker: {error}")))
}

fn join_reader(reader: JoinHandle<std::io::Result<Vec<u8>>>) -> Result<Vec<u8>> {
    reader
        .join()
        .map_err(|_| Error::new(ErrorKind::Internal, "Engine output worker panicked"))?
        .map_err(|error| Error::engine(format!("Could not read engine output: {error}")))
}

fn terminate_group(child: &mut ManagedGroup) {
    let _ = child.kill();
    let _ = child.wait();
}

struct ManagedGroup {
    child: GroupChild,
    reaped: bool,
}

impl Deref for ManagedGroup {
    type Target = GroupChild;

    fn deref(&self) -> &Self::Target {
        &self.child
    }
}

impl DerefMut for ManagedGroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.child
    }
}

impl ManagedGroup {
    fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        let status = self.child.wait()?;
        self.reaped = true;
        Ok(status)
    }
}

impl Drop for ManagedGroup {
    fn drop(&mut self) {
        if !self.reaped {
            let _ = self.child.kill();
            if self.child.wait().is_ok() {
                self.reaped = true;
            }
        }
    }
}

/// Stream-hash a regular file and return its lowercase SHA-256 digest.
pub fn sha256_path(path: &Path) -> Result<String> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(Error::input("Expected a regular non-symlink engine file"));
    }
    sha256_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_DIR: AtomicU64 = AtomicU64::new(1);

    fn temp_app() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let executable = std::env::current_exe().unwrap();
        let engine_dir = dir.path().join("engines");
        fs::create_dir(&engine_dir).unwrap();
        let file_name = format!("runtime-test-{}", NEXT_DIR.fetch_add(1, Ordering::Relaxed));
        let engine_path = engine_dir.join(file_name);
        fs::copy(executable, &engine_path).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&engine_path, fs::Permissions::from_mode(0o700)).unwrap();
        }
        (dir, engine_path)
    }

    fn spec(engine_path: &Path) -> EngineSpec {
        EngineSpec {
            id: "test-engine".into(),
            version: "1.0".into(),
            executable: engine_path
                .strip_prefix(engine_path.ancestors().nth(2).unwrap())
                .unwrap()
                .to_owned(),
            sha256: sha256_file(engine_path).unwrap(),
            license: "MIT".into(),
            capabilities: vec![],
            timeout_ms: 3000,
            max_stdin_bytes: 16,
            max_stdout_bytes: 1024,
            max_stderr_bytes: 1024,
            environment_allowlist: vec![
                "FIXTURE_VALUE".into(),
                "LOCALWRIGHT_ENGINE_TEST_CHILD".into(),
            ],
            health_check_args: Some(vec![
                "--exact".into(),
                "engine::tests::worker_fixture".into(),
                "--nocapture".into(),
            ]),
        }
    }

    #[test]
    fn worker_fixture() {
        if let Ok(mode) = std::env::var("LOCALWRIGHT_ENGINE_TEST_CHILD") {
            match mode.as_str() {
                "health" => println!("engine-ready"),
                "echo" => {
                    use std::io::Read;
                    let mut input = String::new();
                    std::io::stdin().read_to_string(&mut input).unwrap();
                    print!(
                        "{input}:{}",
                        std::env::var("FIXTURE_VALUE").unwrap_or_default()
                    );
                }
                "sleep" => thread::sleep(Duration::from_secs(5)),
                "overflow" => print!("{}", "x".repeat(4096)),
                _ => panic!("unknown fixture mode"),
            }
        }
    }

    fn invocation(mode: &str) -> EngineInvocation {
        EngineInvocation {
            args: vec![
                "--exact".into(),
                "engine::tests::worker_fixture".into(),
                "--nocapture".into(),
            ],
            environment: BTreeMap::from([
                ("FIXTURE_VALUE".into(), "fixture".into()),
                ("LOCALWRIGHT_ENGINE_TEST_CHILD".into(), mode.into()),
            ]),
            ..EngineInvocation::default()
        }
    }

    #[test]
    fn discovery_hashes_app_relative_engine_and_runs_with_controlled_environment() {
        let (dir, path) = temp_app();
        let mut descriptor = spec(&path);
        let health = invocation("health");
        descriptor.health_check_args = Some(health.args);
        let manifest = EngineManifest {
            schema_version: 1,
            engines: vec![descriptor],
        };
        fs::write(
            dir.path().join("engines/manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let runtime = EngineRuntime::discover(dir.path()).unwrap();
        assert!(runtime.unavailable().is_empty());
        let result = runtime
            .run(
                "test-engine",
                EngineInvocation {
                    args: invocation("echo").args,
                    stdin: b"hello".to_vec(),
                    environment: BTreeMap::from([
                        ("FIXTURE_VALUE".into(), "private".into()),
                        ("LOCALWRIGHT_ENGINE_TEST_CHILD".into(), "echo".into()),
                    ]),
                },
                &Job::new(),
            )
            .unwrap();
        assert!(String::from_utf8(result.stdout)
            .unwrap()
            .contains("hello:private"));
        let healthy = runtime.health_check("test-engine", &Job::new()).unwrap();
        assert_eq!(healthy.exit_code, 0);
    }

    #[test]
    fn rejects_path_escape_bad_digest_unallowlisted_environment_and_output_overflow() {
        let (_dir, path) = temp_app();
        let mut descriptor = spec(&path);
        descriptor.executable = PathBuf::from("../escape");
        assert!(validate_spec(&descriptor).is_err());
        descriptor.executable = path.file_name().unwrap().into();
        descriptor.sha256 = "0".repeat(64);
        assert!(verify_executable(&descriptor, &path).is_err());

        descriptor.sha256 = sha256_file(&path).unwrap();
        let engine = ResolvedEngine {
            spec: descriptor,
            executable: path,
        };
        let mut request = invocation("echo");
        request.environment.insert("PATH".into(), "/tmp".into());
        assert!(engine.run(request, &Job::new()).is_err());
        let mut bounded = engine.clone();
        bounded.spec.max_stdout_bytes = 32;
        assert_eq!(
            bounded
                .run(invocation("overflow"), &Job::new())
                .unwrap_err()
                .kind,
            ErrorKind::Limit
        );
    }

    #[test]
    fn job_cancellation_and_timeout_kill_engine_process_groups() {
        let (_dir, path) = temp_app();
        let mut descriptor = spec(&path);
        descriptor.executable = path.file_name().unwrap().into();
        let engine = ResolvedEngine {
            spec: descriptor.clone(),
            executable: path.clone(),
        };
        let job = Job::new();
        let cancel = job.clone();
        let thread = thread::spawn(move || {
            thread::sleep(Duration::from_millis(75));
            cancel.cancel();
        });
        assert_eq!(
            engine.run(invocation("sleep"), &job).unwrap_err().kind,
            ErrorKind::Cancelled
        );
        thread.join().unwrap();

        descriptor.timeout_ms = 75;
        let timed = ResolvedEngine {
            spec: descriptor,
            executable: path,
        };
        assert_eq!(
            timed
                .run(invocation("sleep"), &Job::new())
                .unwrap_err()
                .kind,
            ErrorKind::Engine
        );
    }
}
