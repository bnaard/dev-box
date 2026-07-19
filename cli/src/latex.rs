use crate::cli::OutputFormat;
use crate::config::{AiboxConfig, LatexDocument, LatexEngine};
use crate::runtime::{ContainerState, Runtime};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::fs;
use std::io::{self, Read};
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, UNIX_EPOCH};
use tiny_http::{Header, Response, Server, StatusCode};

const EMBEDPDF_VERSION: &str = "2.14.3";
const GUIDANCE_FILE: &str = "AIBOX-LATEX.md";
const GUIDANCE_HEADER: &str = "<!-- aibox-managed:latex-guidance -->";

pub fn sync_agent_guidance(config: &AiboxConfig, root: &Path) -> Result<()> {
    crate::sync_perimeter::check_perimeter(Path::new(GUIDANCE_FILE))?;
    let path = root.join(GUIDANCE_FILE);
    if config.latex.documents.is_empty() {
        if fs::read_to_string(&path).is_ok_and(|body| body.starts_with(GUIDANCE_HEADER)) {
            fs::remove_file(path)?;
        }
        return Ok(());
    }
    if path.exists()
        && fs::read_to_string(&path).is_ok_and(|body| !body.starts_with(GUIDANCE_HEADER))
    {
        bail!("Refusing to overwrite user-owned {}", path.display());
    }

    let mut body = format!(
        "{GUIDANCE_HEADER}\n# LaTeX Project Workflow\n\nUse the aibox commands below instead of invoking TeX engines directly. They apply the project's configured engine, options, output directories, and project-local TeX caches.\n\n"
    );
    body.push_str("| Document | Source | Build | Watch | Preview |\n|---|---|---|---|---|\n");
    for document in &config.latex.documents {
        body.push_str(&format!(
            "| `{}` | `{}` | `aibox latex build {}` | `aibox latex watch {}` | `aibox preview latex {}` |\n",
            document.name, document.source, document.name, document.name, document.name
        ));
    }
    body.push_str("\nRun `aibox latex status` before starting a watcher or preview so an existing process is reused. Build errors are summarized there from each document's TeX log. Preview binds to the configured address; loopback previews on a remote host require SSH port forwarding.\n");
    if fs::read_to_string(&path).ok().as_deref() != Some(body.as_str()) {
        fs::write(path, body)?;
    }
    Ok(())
}

pub fn cmd_build(config_path: &Option<String>, requested: Option<&str>) -> Result<()> {
    let (config, root) = load(config_path)?;
    let documents = select_documents(&config, requested.unwrap_or("all"))?;
    for document in documents {
        run_document(&config, &root, document, false)?;
        crate::output::ok(&format!("Built LaTeX document '{}'", document.name));
    }
    Ok(())
}

pub fn start_enabled_preview(config_path: &Option<String>, config: &AiboxConfig) -> Result<()> {
    if !config.latex.preview.enabled || config.latex.documents.is_empty() {
        return Ok(());
    }
    let name = config
        .latex
        .preview
        .document
        .as_deref()
        .unwrap_or(&config.latex.documents[0].name);
    let root = config_root(config_path)?;
    let pid_path = state_path(&root, "preview", name);
    if pid_is_running(&pid_path) {
        crate::output::info(&format!("LaTeX preview for '{name}' is already running"));
        return Ok(());
    }
    let _ = fs::remove_file(&pid_path);
    let log_path = root
        .join(".aibox")
        .join("latex")
        .join(format!("preview-{name}.log"));
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let stdout = fs::File::create(&log_path)?;
    let stderr = stdout.try_clone()?;
    let mut command = Command::new(std::env::current_exe()?);
    if let Some(path) = config_path {
        command.args(["--config", path]);
    }
    let mut child = command
        .args(["preview", "latex", name])
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .context("failed to start the LaTeX preview service")?;

    for _ in 0..20 {
        if pid_is_running(&pid_path) {
            crate::output::ok(&format!(
                "Started LaTeX preview for '{}' at {}",
                name,
                preview_url(config)
            ));
            return Ok(());
        }
        if let Some(status) = child.try_wait()? {
            let detail = fs::read_to_string(&log_path).unwrap_or_default();
            bail!(
                "LaTeX preview for '{}' exited with {}: {}",
                name,
                status,
                detail.trim()
            );
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    bail!(
        "Timed out starting LaTeX preview for '{}'; inspect {}",
        name,
        log_path.display()
    )
}

pub fn stop_previews(config_path: &Option<String>, config: &AiboxConfig) -> Result<()> {
    let root = config_root(config_path)?;
    for document in &config.latex.documents {
        let path = state_path(&root, "preview", &document.name);
        let Some(pid) = read_pid(&path) else {
            continue;
        };
        if preview_process_matches(pid, &document.name) && terminate_process(pid) {
            crate::output::ok(&format!("Stopped LaTeX preview for '{}'", document.name));
        }
        let _ = fs::remove_file(path);
    }
    Ok(())
}

pub fn cmd_watch(config_path: &Option<String>, name: &str) -> Result<()> {
    let (config, root) = load(config_path)?;
    let document = find_document(&config, name)?;
    if config.latex.engine == LatexEngine::Tectonic {
        bail!("aibox latex watch requires a latexmk engine; tectonic supports build only");
    }
    ensure_no_duplicate(&root, "watch", name)?;
    let _pid = PidGuard::create(&state_path(&root, "watch", name))?;
    crate::output::info(&format!(
        "Watching LaTeX document '{}' (Ctrl-C to stop)",
        name
    ));
    run_document(&config, &root, document, true)
}

pub fn cmd_status(config_path: &Option<String>, format: OutputFormat) -> Result<()> {
    let (config, root) = load(config_path)?;
    let rows: Vec<DocumentStatus> = config
        .latex
        .documents
        .iter()
        .map(|document| document_status(&config, &root, document))
        .collect();

    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&rows)?),
        OutputFormat::Yaml => print!("{}", serde_yaml::to_string(&rows)?),
        OutputFormat::Table => {
            if rows.is_empty() {
                crate::output::info("No LaTeX documents configured");
                return Ok(());
            }
            println!("DOCUMENT\tSOURCE\tPDF\tWATCH\tPREVIEW\tRECENT ERROR");
            for row in rows {
                println!(
                    "{}\t{}\t{}\t{}\t{}\t{}",
                    row.name,
                    row.source,
                    if row.pdf_exists { "ready" } else { "missing" },
                    if row.watching { "running" } else { "stopped" },
                    if row.preview_running {
                        row.preview_url.as_str()
                    } else {
                        "stopped"
                    },
                    row.recent_error.unwrap_or_else(|| "-".to_string())
                );
            }
        }
    }
    Ok(())
}

pub fn cmd_preview(config_path: &Option<String>, name: &str) -> Result<()> {
    let (config, root) = load(config_path)?;
    if !config.latex.preview.enabled {
        bail!("LaTeX preview is disabled; set latex.preview.enabled = true");
    }
    let document = find_document(&config, name)?.clone();
    if pid_is_running(&state_path(&root, "preview", name)) {
        crate::output::info(&format!(
            "LaTeX preview for '{}' is already running at {}",
            name,
            preview_url(&config)
        ));
        return Ok(());
    }
    ensure_no_duplicate(&root, "preview", name)?;
    let _pid = PidGuard::create(&state_path(&root, "preview", name))?;

    let ip: IpAddr = config.latex.preview.bind.parse()?;
    let address = SocketAddr::new(ip, config.latex.preview.port);
    let server = Server::http(address)
        .map_err(|error| anyhow::anyhow!("failed to bind LaTeX preview at {address}: {error}"))?;
    let actual = server.server_addr().to_ip().unwrap_or(address);
    let pdf = pdf_path(&root, &document);
    let revision = Arc::new((Mutex::new(file_revision(&pdf)), Condvar::new()));
    spawn_pdf_watcher(pdf.clone(), revision.clone());

    let browser_ip = if actual.ip().is_unspecified() {
        IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
    } else {
        actual.ip()
    };
    let url = http_url(browser_ip, actual.port());
    crate::output::ok(&format!(
        "LaTeX preview for '{}' is available at {url}",
        name
    ));
    if actual.ip().is_loopback() {
        crate::output::info(&format!(
            "Remote host: ssh -L {0}:{1}:{0} <user>@<host>",
            actual.port(),
            ssh_host(actual.ip())
        ));
    }

    for request in server.incoming_requests() {
        let pdf = pdf.clone();
        let revision = revision.clone();
        let title = document.name.clone();
        std::thread::spawn(move || {
            let path = request.url().split('?').next().unwrap_or("/");
            let result = match path {
                "/" => respond_html(request, &title),
                "/document.pdf" => respond_pdf(request, &pdf),
                "/events" => respond_events(request, revision),
                "/health" => respond_json(request, r#"{"status":"ok"}"#),
                _ => request.respond(Response::empty(StatusCode(404))),
            };
            if let Err(error) = result {
                tracing::debug!("LaTeX preview request ended: {error}");
            }
        });
    }
    Ok(())
}

fn load(config_path: &Option<String>) -> Result<(AiboxConfig, PathBuf)> {
    Ok((
        AiboxConfig::from_cli_option(config_path)?,
        config_root(config_path)?,
    ))
}

fn config_root(config_path: &Option<String>) -> Result<PathBuf> {
    let path = PathBuf::from(config_path.as_deref().unwrap_or("aibox.toml"));
    let absolute = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()?.join(path)
    };
    Ok(absolute.parent().unwrap_or(Path::new(".")).to_path_buf())
}

fn select_documents<'a>(
    config: &'a AiboxConfig,
    requested: &str,
) -> Result<Vec<&'a LatexDocument>> {
    if config.latex.documents.is_empty() {
        bail!("No LaTeX documents are configured in [[latex.documents]]");
    }
    if requested == "all" {
        return Ok(config.latex.documents.iter().collect());
    }
    Ok(vec![find_document(config, requested)?])
}

fn find_document<'a>(config: &'a AiboxConfig, name: &str) -> Result<&'a LatexDocument> {
    config
        .latex
        .documents
        .iter()
        .find(|document| document.name == name)
        .ok_or_else(|| anyhow::anyhow!("Unknown LaTeX document '{name}'"))
}

fn run_document(
    config: &AiboxConfig,
    root: &Path,
    document: &LatexDocument,
    watch: bool,
) -> Result<()> {
    let source = root.join(&document.source);
    if !source.is_file() {
        bail!("LaTeX source does not exist: {}", source.display());
    }
    fs::create_dir_all(root.join(&document.output_dir))?;
    fs::create_dir_all(root.join(&config.latex.cache_dir).join("texmf-var"))?;
    fs::create_dir_all(root.join(&config.latex.cache_dir).join("texmf-config"))?;

    let (program, args) = build_command(config, document, watch);
    if command_exists(&program) {
        let status = Command::new(&program)
            .args(&args)
            .current_dir(root)
            .env(
                "TEXMFVAR",
                root.join(&config.latex.cache_dir).join("texmf-var"),
            )
            .env(
                "TEXMFCONFIG",
                root.join(&config.latex.cache_dir).join("texmf-config"),
            )
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| format!("failed to run {program}"))?;
        if !status.success() {
            bail!(
                "LaTeX {} failed for '{}'",
                if watch { "watch" } else { "build" },
                document.name
            );
        }
        return Ok(());
    }

    let runtime = Runtime::detect()
        .context("LaTeX is unavailable on the host and no container runtime is available")?;
    if runtime.container_status(&config.container.name)? != ContainerState::Running {
        bail!(
            "LaTeX is unavailable on the host and container '{}' is not running",
            config.container.name
        );
    }
    let mut command = format!(
        "cd /workspace && TEXMFVAR={} TEXMFCONFIG={} {}",
        shell_quote(&format!("/workspace/{}/texmf-var", config.latex.cache_dir)),
        shell_quote(&format!(
            "/workspace/{}/texmf-config",
            config.latex.cache_dir
        )),
        shell_quote(&program),
    );
    for arg in args {
        command.push(' ');
        command.push_str(&shell_quote(&arg));
    }
    let ok = if watch {
        runtime.exec_interactive(
            &config.container.name,
            &config.container.user,
            &["sh", "-lc", &command],
        )?;
        true
    } else {
        runtime.exec_status(
            &config.container.name,
            &config.container.user,
            &["sh", "-lc", &command],
        )?
    };
    if !ok {
        bail!("LaTeX build failed for '{}'", document.name);
    }
    Ok(())
}

fn build_command(
    config: &AiboxConfig,
    document: &LatexDocument,
    watch: bool,
) -> (String, Vec<String>) {
    if config.latex.engine == LatexEngine::Tectonic {
        let mut args = vec!["--outdir".to_string(), document.output_dir.clone()];
        args.extend(config.latex.options.clone());
        args.push(document.source.clone());
        return ("tectonic".to_string(), args);
    }
    let engine = match config.latex.engine {
        LatexEngine::Lualatex => "-lualatex",
        LatexEngine::Pdflatex => "-pdf",
        LatexEngine::Xelatex => "-xelatex",
        LatexEngine::Tectonic => unreachable!(),
    };
    let mut args = vec![
        engine.to_string(),
        "-interaction=nonstopmode".to_string(),
        "-halt-on-error".to_string(),
        "-file-line-error".to_string(),
        format!("-outdir={}", document.output_dir),
    ];
    if watch {
        args.extend(["-pvc".to_string(), "-view=none".to_string()]);
    }
    args.extend(config.latex.options.clone());
    args.push(document.source.clone());
    ("latexmk".to_string(), args)
}

#[derive(Serialize)]
struct DocumentStatus {
    name: String,
    source: String,
    pdf: String,
    pdf_exists: bool,
    watching: bool,
    preview_running: bool,
    preview_url: String,
    recent_error: Option<String>,
}

fn document_status(config: &AiboxConfig, root: &Path, document: &LatexDocument) -> DocumentStatus {
    let pdf = pdf_path(root, document);
    let log = root.join(&document.output_dir).join(
        Path::new(&document.source)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
            + ".log",
    );
    DocumentStatus {
        name: document.name.clone(),
        source: document.source.clone(),
        pdf: relative_display(root, &pdf),
        pdf_exists: pdf.is_file(),
        watching: pid_is_running(&state_path(root, "watch", &document.name)),
        preview_running: pid_is_running(&state_path(root, "preview", &document.name)),
        preview_url: preview_url(config),
        recent_error: recent_error(&log),
    }
}

fn recent_error(path: &Path) -> Option<String> {
    let body = fs::read_to_string(path).ok()?;
    body.lines()
        .rev()
        .find(|line| {
            let lower = line.to_ascii_lowercase();
            line.starts_with('!') || lower.contains("error:")
        })
        .map(|line| line.trim().chars().take(160).collect())
}

fn pdf_path(root: &Path, document: &LatexDocument) -> PathBuf {
    let stem = Path::new(&document.source).file_stem().unwrap_or_default();
    root.join(&document.output_dir)
        .join(stem)
        .with_extension("pdf")
}

fn state_path(root: &Path, kind: &str, name: &str) -> PathBuf {
    root.join(".aibox")
        .join("latex")
        .join(format!("{kind}-{name}.pid"))
}

fn ensure_no_duplicate(root: &Path, kind: &str, name: &str) -> Result<()> {
    let path = state_path(root, kind, name);
    if pid_is_running(&path) {
        bail!("LaTeX {kind} for '{name}' is already running");
    }
    let _ = fs::remove_file(path);
    Ok(())
}

struct PidGuard(PathBuf);

impl PidGuard {
    fn create(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, std::process::id().to_string())?;
        Ok(Self(path.to_path_buf()))
    }
}

impl Drop for PidGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn pid_is_running(path: &Path) -> bool {
    read_pid(path).is_some_and(process_is_running)
}

fn read_pid(path: &Path) -> Option<u32> {
    fs::read_to_string(path)
        .ok()
        .and_then(|body| body.trim().parse().ok())
}

#[cfg(unix)]
fn process_is_running(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(unix)]
fn preview_process_matches(pid: u32, document: &str) -> bool {
    Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output()
        .is_ok_and(|output| {
            let command = String::from_utf8_lossy(&output.stdout);
            command.contains("preview latex") && command.contains(document)
        })
}

#[cfg(windows)]
fn preview_process_matches(pid: u32, _document: &str) -> bool {
    process_is_running(pid)
}

#[cfg(unix)]
fn terminate_process(pid: u32) -> bool {
    Command::new("kill")
        .arg(pid.to_string())
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(windows)]
fn terminate_process(pid: u32) -> bool {
    Command::new("taskkill")
        .args(["/PID", &pid.to_string()])
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(windows)]
fn process_is_running(pid: u32) -> bool {
    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH"])
        .output()
        .is_ok_and(|output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout).contains(&pid.to_string())
        })
}

fn command_exists(program: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|path| path.join(program).is_file()))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn http_url(ip: IpAddr, port: u16) -> String {
    format!("http://{}/", SocketAddr::new(ip, port))
}

fn preview_url(config: &AiboxConfig) -> String {
    http_url(
        config
            .latex
            .preview
            .bind
            .parse()
            .expect("validated LaTeX preview IP"),
        config.latex.preview.port,
    )
}

fn ssh_host(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(ip) => ip.to_string(),
        IpAddr::V6(ip) => format!("[{ip}]"),
    }
}

fn respond_html(request: tiny_http::Request, title: &str) -> io::Result<()> {
    let html = viewer_html(title);
    request.respond(
        Response::from_string(html).with_header(header("Content-Type", "text/html; charset=utf-8")),
    )
}

fn respond_pdf(request: tiny_http::Request, path: &Path) -> io::Result<()> {
    match fs::File::open(path) {
        Ok(file) => {
            let length = file.metadata().ok().map(|metadata| metadata.len() as usize);
            request.respond(Response::new(
                StatusCode(200),
                vec![
                    header("Content-Type", "application/pdf"),
                    header("Cache-Control", "no-store"),
                ],
                file,
                length,
                None,
            ))
        }
        Err(_) => request
            .respond(Response::from_string("PDF has not been built yet").with_status_code(404)),
    }
}

fn respond_json(request: tiny_http::Request, body: &str) -> io::Result<()> {
    request.respond(
        Response::from_string(body).with_header(header("Content-Type", "application/json")),
    )
}

fn respond_events(
    request: tiny_http::Request,
    revision: Arc<(Mutex<u64>, Condvar)>,
) -> io::Result<()> {
    let initial = *revision.0.lock().unwrap();
    request.respond(Response::new(
        StatusCode(200),
        vec![
            header("Content-Type", "text/event-stream"),
            header("Cache-Control", "no-cache"),
            header("X-Accel-Buffering", "no"),
        ],
        RevisionStream::new(revision, initial),
        None,
        None,
    ))
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name, value).expect("static HTTP header is valid")
}

fn viewer_html(title: &str) -> String {
    let safe_title = title
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    format!(
        r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>{safe_title} - LaTeX preview</title>
<style>html,body,#viewer{{height:100%;margin:0}}#waiting{{font:14px system-ui;padding:16px}}</style></head>
<body><div id="waiting">Waiting for the first completed PDF build...</div><div id="viewer"></div>
<script type="module">
import EmbedPDF from 'https://cdn.jsdelivr.net/npm/@embedpdf/snippet@{EMBEDPDF_VERSION}/dist/embedpdf.js';
let revision = Date.now();
async function openPdf() {{
  const response = await fetch(`/document.pdf?v=${{revision}}`, {{method:'HEAD', cache:'no-store'}});
  if (!response.ok) return;
  document.querySelector('#waiting').remove();
  const viewer = EmbedPDF.init({{
    type:'container', target:document.querySelector('#viewer'), theme:{{preference:'system'}},
    documentManager:{{initialDocuments:[{{url:`/document.pdf?v=${{revision}}`, documentId:'live-preview'}}]}}
  }});
  if (!viewer) return;
  const registry = await viewer.registry;
  const scroll = registry.getPlugin('scroll')?.provides();
  const zoom = registry.getPlugin('zoom')?.provides()?.forDocument('live-preview');
  const savedPage = Number(sessionStorage.getItem('aibox-latex-page') || '1');
  const savedZoom = JSON.parse(sessionStorage.getItem('aibox-latex-zoom') || 'null');
  scroll?.onLayoutReady(event => {{
    if (event.documentId !== 'live-preview') return;
    scroll.scrollToPage({{pageNumber:savedPage, behavior:'instant'}});
    if (savedZoom !== null) zoom?.requestZoom(savedZoom);
  }});
  scroll?.onPageChange(event => {{
    if (event.documentId === 'live-preview') sessionStorage.setItem('aibox-latex-page', String(event.pageNumber));
  }});
  zoom?.onStateChange(state => sessionStorage.setItem('aibox-latex-zoom', JSON.stringify(state.currentZoomLevel)));
}}
await openPdf();
const events = new EventSource('/events');
events.addEventListener('revision', event => {{
  const next = Number(event.data);
  if (next && next !== revision) {{ revision = next; location.reload(); }}
}});
</script></body></html>"#
    )
}

fn file_revision(path: &Path) -> u64 {
    let Ok(metadata) = fs::metadata(path) else {
        return 0;
    };
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0);
    modified ^ metadata.len().rotate_left(17)
}

fn spawn_pdf_watcher(path: PathBuf, revision: Arc<(Mutex<u64>, Condvar)>) {
    std::thread::spawn(move || {
        let mut candidate = file_revision(&path);
        let mut stable_polls = 0;
        loop {
            std::thread::sleep(Duration::from_millis(250));
            let observed = file_revision(&path);
            if observed == candidate {
                stable_polls += 1;
            } else {
                candidate = observed;
                stable_polls = 0;
            }
            if stable_polls >= 2 {
                let (lock, wake) = &*revision;
                let mut current = lock.lock().unwrap();
                if candidate != *current {
                    *current = candidate;
                    wake.notify_all();
                }
            }
        }
    });
}

struct RevisionStream {
    revision: Arc<(Mutex<u64>, Condvar)>,
    last: u64,
    pending: Vec<u8>,
    offset: usize,
}

impl RevisionStream {
    fn new(revision: Arc<(Mutex<u64>, Condvar)>, last: u64) -> Self {
        Self {
            revision,
            last,
            pending: Vec::new(),
            offset: 0,
        }
    }
}

impl Read for RevisionStream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.offset >= self.pending.len() {
            let (lock, wake) = &*self.revision;
            let current = lock.lock().unwrap();
            let (current, timeout) = wake
                .wait_timeout_while(current, Duration::from_secs(15), |value| {
                    *value == self.last
                })
                .unwrap();
            if *current != self.last {
                self.last = *current;
                self.pending = format!("event: revision\ndata: {}\n\n", self.last).into_bytes();
            } else if timeout.timed_out() {
                self.pending = b": keepalive\n\n".to_vec();
            }
            self.offset = 0;
        }
        let count = buffer
            .len()
            .min(self.pending.len().saturating_sub(self.offset));
        buffer[..count].copy_from_slice(&self.pending[self.offset..self.offset + count]);
        self.offset += count;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latexmk_command_has_stable_watch_flags() {
        let mut config = crate::config::test_config();
        config.latex.engine = LatexEngine::Lualatex;
        let document = LatexDocument {
            name: "overview".into(),
            source: "doc/overview.tex".into(),
            output_dir: "out".into(),
        };
        let (_, args) = build_command(&config, &document, true);
        assert!(args.iter().any(|arg| arg == "-pvc"));
        assert!(args.iter().any(|arg| arg == "-view=none"));
        assert!(args.iter().any(|arg| arg == "-halt-on-error"));
    }

    #[test]
    fn shell_quote_handles_spaces_and_apostrophes() {
        assert_eq!(shell_quote("a b's"), "'a b'\\''s'");
    }

    #[test]
    fn preview_urls_are_valid_for_ipv4_and_ipv6() {
        assert_eq!(
            http_url("127.0.0.1".parse().unwrap(), 8765),
            "http://127.0.0.1:8765/"
        );
        assert_eq!(http_url("::1".parse().unwrap(), 8765), "http://[::1]:8765/");
    }

    #[test]
    fn viewer_pins_embedpdf_and_uses_sse() {
        let html = viewer_html("overview");
        assert!(html.contains("@2.14.3/dist/embedpdf.js"));
        assert!(html.contains("new EventSource('/events')"));
        assert!(html.contains("aibox-latex-page"));
        assert!(html.contains("aibox-latex-zoom"));
    }

    #[test]
    fn guidance_is_managed_without_overwriting_user_content() {
        let root = tempfile::tempdir().unwrap();
        let mut config = crate::config::test_config();
        config.latex.documents.push(LatexDocument {
            name: "overview".into(),
            source: "docs/overview.tex".into(),
            output_dir: ".latex-cache/overview".into(),
        });
        sync_agent_guidance(&config, root.path()).unwrap();
        let path = root.path().join(GUIDANCE_FILE);
        assert!(
            fs::read_to_string(&path)
                .unwrap()
                .starts_with(GUIDANCE_HEADER)
        );

        fs::write(&path, "user content\n").unwrap();
        assert!(sync_agent_guidance(&config, root.path()).is_err());
    }
}
