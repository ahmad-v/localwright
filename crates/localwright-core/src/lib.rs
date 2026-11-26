//! General-purpose contracts for offline desktop tools. No GUI or engine dependency.
#![forbid(unsafe_code)]
pub mod figure;
use serde::{Deserialize, Serialize};
use std::{fs::{self, File}, io::{Read, Write}, path::{Path, PathBuf}, sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}}};

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all="snake_case")]
pub enum ErrorKind { Input, Io, Limit, Cancelled, Engine, Collision, Internal }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error { pub kind: ErrorKind, pub message: String }
impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self { Self { kind, message: message.into() } }
    pub fn input(message: impl Into<String>) -> Self { Self::new(ErrorKind::Input, message) }
    pub fn engine(message: impl Into<String>) -> Self { Self::new(ErrorKind::Engine, message) }
}
impl std::fmt::Display for Error { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.message) } }
impl std::error::Error for Error {}
impl From<std::io::Error> for Error { fn from(e: std::io::Error) -> Self { Self::new(ErrorKind::Io, e.to_string()) } }
impl From<serde_json::Error> for Error { fn from(e: serde_json::Error) -> Self { Self::input(e.to_string()) } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability { pub format: String, pub direction: String, pub operation: String, pub engine: String, pub limitations: String, pub fixture: String }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engine { pub id: String, pub version: String, pub license: String, pub targets: Vec<String>, pub requirements: String, pub capabilities: Vec<Capability> }
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Request { pub operation: String, pub input: PathBuf, pub output: PathBuf, pub parameter: String, pub draft: String }
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Outcome { pub preview: String, pub warnings: Vec<String>, pub written: Option<PathBuf>, pub draft: Option<String> }
#[derive(Debug, Clone, Copy)]
pub struct Operation { pub id: &'static str, pub label: &'static str, pub hint: &'static str }
pub type Handler = fn(Request, Job) -> Result<Outcome>;
static NEXT: AtomicU64 = AtomicU64::new(1);
#[derive(Debug, Clone)]
pub struct Job { pub id: u64, cancelled: Arc<AtomicBool>, done: Arc<AtomicU64>, total: Arc<AtomicU64> }
impl Default for Job { fn default() -> Self { Self::new() } }
impl Job {
    pub fn new() -> Self { Self { id: NEXT.fetch_add(1, Ordering::Relaxed), cancelled: Arc::new(AtomicBool::new(false)), done: Arc::new(AtomicU64::new(0)), total: Arc::new(AtomicU64::new(0)) } }
    pub fn cancel(&self) { self.cancelled.store(true, Ordering::Relaxed); }
    pub fn check(&self) -> Result<()> { if self.cancelled.load(Ordering::Relaxed) { Err(Error::new(ErrorKind::Cancelled, "Job cancelled")) } else { Ok(()) } }
    pub fn progress(&self, done: u64, total: u64) { self.total.store(total, Ordering::Relaxed); self.done.store(done, Ordering::Relaxed); }
    pub fn fraction(&self) -> Option<f32> { let total=self.total.load(Ordering::Relaxed); (total>0).then(|| (self.done.load(Ordering::Relaxed) as f32 / total as f32).clamp(0.0,1.0)) }
}
/// Read only a regular file, reject terminal symlinks and cap bytes even if it grows.
/// This is not a sandbox: parent links and concurrent pathname substitution remain risks.
pub fn read_bounded(path: &Path, max: u64) -> Result<Vec<u8>> {
    let m=fs::symlink_metadata(path)?;
    if !m.is_file() || m.file_type().is_symlink() { return Err(Error::input("Expected a regular, non-symlink file")); }
    if m.len()>max { return Err(Error::new(ErrorKind::Limit,"Input exceeds byte limit")); }
    let mut file=File::open(path)?;
    if !file.metadata()?.is_file() { return Err(Error::input("Not a regular file")); }
    let mut bytes=Vec::new(); (&mut file).take(max.saturating_add(1)).read_to_end(&mut bytes)?;
    if bytes.len() as u64>max { return Err(Error::new(ErrorKind::Limit,"Input grew beyond limit")); }
    Ok(bytes)
}
/// Publish a new output only. Temporary sibling is removed on failure. Never overwrites.
pub fn write_new(path: &Path, build: impl FnOnce(&mut File)->Result<()>) -> Result<()> {
    if path.file_name().is_none() { return Err(Error::input("Choose an output filename")); }
    let parent=path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut temp=tempfile::NamedTempFile::new_in(parent)?;
    build(temp.as_file_mut())?; temp.as_file_mut().flush()?; temp.as_file().sync_all()?;
    temp.persist_noclobber(path).map_err(|e| Error::new(if e.error.kind()==std::io::ErrorKind::AlreadyExists {ErrorKind::Collision} else {ErrorKind::Io},e.error.to_string()))?;
    Ok(())
}
pub fn write_bytes(path: &Path, bytes: &[u8]) -> Result<()> { write_new(path, |f| { f.write_all(bytes)?; Ok(()) }) }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Settings { pub schema_version: u32, pub theme: String, pub font_size: u16 }
impl Default for Settings { fn default()->Self { Self { schema_version:1, theme:"Modern Light".into(),font_size:16 } } }
impl Settings {
    pub fn parse(bytes: &[u8]) -> Result<Self> { if bytes.len()>65536 {return Err(Error::input("Settings too large"));} let s:Self=serde_json::from_slice(bytes)?; if s.schema_version!=1 || !(12..=48).contains(&s.font_size) || s.theme.len()>64 { return Err(Error::input("Invalid settings")); } Ok(s) }
    pub fn save_new(&self,path:&Path)->Result<()> { let bytes=serde_json::to_vec_pretty(self)?; Self::parse(&bytes)?; write_bytes(path,&bytes) }
}
/// Shared non-GUI CLI for reproducible fixture testing, using the same operation handler.
pub fn cli(args: Vec<String>, operations: &[Operation], handler: Handler) -> Result<()> {
    if args.is_empty() || args[0]=="--help" { for op in operations { println!("{}: {}",op.id,op.hint); } println!("Usage: --cli OP INPUT OUTPUT PARAMETER (use empty strings for unused arguments)"); return Ok(()); }
    if !operations.iter().any(|op|op.id==args[0]) {return Err(Error::input("Unknown operation"));}
    let get=|i:usize|args.get(i).cloned().unwrap_or_default();
    let r=handler(Request{operation:get(0),input:get(1).into(),output:get(2).into(),parameter:get(3),draft:String::new()},Job::new())?;
    println!("{}",serde_json::to_string_pretty(&r)?); Ok(())
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn no_overwrite_and_failed_write() { let d=tempfile::tempdir().unwrap(); let p=d.path().join("é.txt"); write_bytes(&p,b"original").unwrap(); assert_eq!(write_bytes(&p,b"bad").unwrap_err().kind,ErrorKind::Collision); assert_eq!(read_bounded(&p,8).unwrap(),b"original"); assert!(read_bounded(&p,2).is_err()); let q=d.path().join("failed"); assert!(write_new(&q,|_|Err(Error::input("failure"))).is_err()); assert!(!q.exists()); }
 #[test] fn cancellation_and_settings() { let j=Job::new(); let other=j.clone(); j.progress(1,2); assert_eq!(j.fraction(),Some(0.5)); other.cancel(); assert_eq!(j.check().unwrap_err().kind,ErrorKind::Cancelled); assert!(Settings::parse(br#"{"schema_version":2,"theme":"x","font_size":16}"#).is_err()); assert_eq!(Settings::parse(&serde_json::to_vec(&Settings::default()).unwrap()).unwrap(),Settings::default()); }
}
