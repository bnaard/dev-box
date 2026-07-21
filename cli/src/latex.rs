use crate::config::{AiboxConfig, LatexDocument, LatexEngine};
use crate::runtime::Runtime;
use anyhow::{Result, bail};
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const LEGACY_GUIDANCE_FILE: &str = "AIBOX-LATEX.md";
const LEGACY_GUIDANCE_HEADER: &str = "<!-- aibox-managed:latex-guidance -->";
const AGENTS_GUIDANCE_BEGIN: &str = "<!-- aibox-managed:latex-runtime BEGIN -->";
const AGENTS_GUIDANCE_END: &str = "<!-- aibox-managed:latex-runtime END -->";
pub const BUILD_SCRIPT_PATH: &str = ".local/bin/aibox-latex-build";
pub const WATCH_SCRIPT_PATH: &str = ".local/bin/aibox-latex-watch";
const SCRIPT_HEADER: &str = "# aibox-managed:latex-container-script";

pub fn sync_agent_guidance(config: &AiboxConfig, root: &Path) -> Result<()> {
    remove_legacy_guidance(root)?;
    crate::sync_perimeter::check_perimeter(Path::new(crate::processkit_vocab::AGENTS_FILENAME))?;
    let path = root.join(crate::processkit_vocab::AGENTS_FILENAME);
    let Ok(existing) = fs::read_to_string(&path) else {
        return Ok(());
    };

    let retained = remove_managed_block(&existing)?;
    let body = if config.latex.documents.is_empty() {
        retained
    } else {
        format!(
            "{}\n\n{}",
            retained.trim_end(),
            render_agent_guidance(config)
        )
    };
    if body != existing {
        fs::write(path, body)?;
    }
    Ok(())
}

fn remove_legacy_guidance(root: &Path) -> Result<()> {
    let path = root.join(LEGACY_GUIDANCE_FILE);
    if fs::read_to_string(&path).is_ok_and(|body| body.starts_with(LEGACY_GUIDANCE_HEADER)) {
        crate::sync_perimeter::check_perimeter(Path::new(LEGACY_GUIDANCE_FILE))?;
        fs::remove_file(path)?;
    }
    Ok(())
}

fn remove_managed_block(body: &str) -> Result<String> {
    let begin = body.find(AGENTS_GUIDANCE_BEGIN);
    let end = body.find(AGENTS_GUIDANCE_END);
    match (begin, end) {
        (None, None) => Ok(body.to_string()),
        (Some(begin), Some(end)) if end >= begin => {
            let after = end + AGENTS_GUIDANCE_END.len();
            Ok(format!("{}{}", &body[..begin], &body[after..])
                .trim_end()
                .to_string())
        }
        _ => bail!("Malformed aibox-managed LaTeX guidance block in AGENTS.md"),
    }
}

fn render_agent_guidance(config: &AiboxConfig) -> String {
    let service = preview_service_name(config);
    let mut body = format!(
        "{AGENTS_GUIDANCE_BEGIN}\n## LaTeX preview companion\n\nBuild and watch LaTeX only inside the development container. The read-only preview sidecar serves completed PDFs and never compiles source files. Check it from the development container with `curl -fsS http://{service}:8765/health`. Tell the user to open `{}`; the browser refreshes after each completed PDF rebuild.\n\n",
        preview_url(config)
    );
    body.push_str("| Document | Source | Build | Watch | Preview |\n|---|---|---|---|---|\n");
    for document in &config.latex.documents {
        let watch = if config.latex.engine == LatexEngine::Tectonic {
            "not supported by tectonic".to_string()
        } else {
            format!("`aibox-latex-watch {}`", document.name)
        };
        body.push_str(&format!(
            "| `{}` | `{}` | `aibox-latex-build {}` | {} | `{}` |\n",
            document.name,
            document.source,
            document.name,
            watch,
            preview_document_url(config, &document.name)
        ));
    }
    body.push_str("\nRun `aibox-latex-build` with no argument to build every configured document. Run one `aibox-latex-watch <name>` process per document that should rebuild continuously; stop it with Ctrl-C. Host-side `aibox up` starts the shared Compose preview sidecar when `latex.preview.enabled = true`. Loopback previews on a remote host require SSH port forwarding.\n");
    body.push_str(AGENTS_GUIDANCE_END);
    body.push('\n');
    body
}

pub fn build_script(config: &AiboxConfig) -> String {
    let mut script = script_prelude(
        "Build one configured LaTeX document, or all documents when omitted.",
        config,
    );
    for (index, document) in config.latex.documents.iter().enumerate() {
        script.push_str(&format!(
            "build_{index}() {{\n  prepare {} {}\n  printf '%s\\n' {}\n  {}\n}}\n\n",
            shell_quote(&document.source),
            shell_quote(&document.output_dir),
            shell_quote(&format!("Building LaTeX document '{}'", document.name)),
            shell_command(config, document, false)
        ));
    }
    script.push_str("case \"${1:-all}\" in\n");
    script.push_str("  all)\n");
    for index in 0..config.latex.documents.len() {
        script.push_str(&format!("    build_{index}\n"));
    }
    script.push_str("    ;;\n");
    for (index, document) in config.latex.documents.iter().enumerate() {
        script.push_str(&format!(
            "  {}) build_{index} ;;\n",
            shell_case_pattern(&document.name)
        ));
    }
    script.push_str(&unknown_document_branch(config));
    script.push_str("esac\n");
    script
}

pub fn watch_script(config: &AiboxConfig) -> String {
    let mut script = script_prelude(
        "Continuously rebuild one configured LaTeX document in this container.",
        config,
    );
    let default = if config.latex.documents.len() == 1 {
        config.latex.documents[0].name.as_str()
    } else {
        ""
    };
    script.push_str(&format!("document=${{1:-{}}}\n", shell_word(default)));
    script.push_str("case \"$document\" in\n");
    for document in &config.latex.documents {
        script.push_str(&format!("  {})\n", shell_case_pattern(&document.name)));
        if config.latex.engine == LatexEngine::Tectonic {
            script.push_str("    printf '%s\\n' 'Watch mode requires a latexmk engine; tectonic supports build only.' >&2\n    exit 2\n");
        } else {
            script.push_str(&format!(
                "    prepare {} {}\n    printf '%s\\n' {}\n    exec {}\n",
                shell_quote(&document.source),
                shell_quote(&document.output_dir),
                shell_quote(&format!(
                    "Watching LaTeX document '{}' (Ctrl-C to stop)",
                    document.name
                )),
                shell_command(config, document, true)
            ));
        }
        script.push_str("    ;;\n");
    }
    script.push_str(&unknown_document_branch(config));
    script.push_str("esac\n");
    script
}

fn script_prelude(description: &str, config: &AiboxConfig) -> String {
    format!(
        "#!/bin/sh\n{SCRIPT_HEADER}\n# {description}\nset -eu\n\nworkspace=${{AIBOX_WORKSPACE:-/workspace}}\ncd \"$workspace\"\nexport TEXMFVAR=\"$workspace/{}\"\nexport TEXMFCONFIG=\"$workspace/{}\"\n\nprepare() {{\n  source_path=$1\n  output_dir=$2\n  if [ ! -f \"$workspace/$source_path\" ]; then\n    printf 'LaTeX source does not exist: %s\\n' \"$workspace/$source_path\" >&2\n    exit 1\n  fi\n  mkdir -p \"$workspace/$output_dir\" \"$TEXMFVAR\" \"$TEXMFCONFIG\"\n}}\n\n",
        join_relative(&config.latex.cache_dir, "texmf-var"),
        join_relative(&config.latex.cache_dir, "texmf-config")
    )
}

fn shell_command(config: &AiboxConfig, document: &LatexDocument, watch: bool) -> String {
    let (program, args) = build_command(config, document, watch);
    std::iter::once(program.as_str())
        .chain(args.iter().map(String::as_str))
        .map(shell_quote)
        .collect::<Vec<_>>()
        .join(" ")
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

fn unknown_document_branch(config: &AiboxConfig) -> String {
    let names = config
        .latex
        .documents
        .iter()
        .map(|document| document.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "  *) printf 'Unknown LaTeX document: %s\\nConfigured documents: %s\\n' \"${{1:-}}\" {} >&2; exit 2 ;;\n",
        shell_quote(&names)
    )
}

fn join_relative(base: &str, child: &str) -> String {
    format!("{}/{}", base.trim_end_matches('/'), child)
}

fn shell_case_pattern(value: &str) -> String {
    shell_quote(value)
}

fn shell_word(value: &str) -> String {
    if value.is_empty() {
        "''".to_string()
    } else {
        shell_quote(value)
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub fn start_enabled_preview(
    config_path: &Option<String>,
    runtime: &Runtime,
    config: &AiboxConfig,
) -> Result<()> {
    if !config.latex.preview.enabled || config.latex.documents.is_empty() {
        return Ok(());
    }
    cleanup_legacy_host_previews(config_path, config)?;
    let service = preview_service_name(config);
    runtime.compose_up_no_deps(crate::config::COMPOSE_FILE, &service)?;
    runtime.wait_for_running(&service, 7500)?;
    crate::output::ok(&format!(
        "Started LaTeX preview sidecar for {} document(s) at {}",
        config.latex.documents.len(),
        preview_url(config)
    ));
    Ok(())
}

pub fn cleanup_legacy_host_previews(
    config_path: &Option<String>,
    config: &AiboxConfig,
) -> Result<()> {
    let root = config_root(config_path)?;
    for path in std::iter::once(state_path(&root, "preview", "server")).chain(
        config
            .latex
            .documents
            .iter()
            .map(|document| state_path(&root, "preview", &document.name)),
    ) {
        let Some(pid) = read_pid(&path) else {
            continue;
        };
        if preview_process_matches(pid) && terminate_process(pid) {
            crate::output::ok("Stopped legacy host LaTeX preview process");
        }
        let _ = fs::remove_file(path);
    }
    Ok(())
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

fn state_path(root: &Path, kind: &str, name: &str) -> PathBuf {
    root.join(".aibox")
        .join("latex")
        .join(format!("{kind}-{name}.pid"))
}

fn read_pid(path: &Path) -> Option<u32> {
    fs::read_to_string(path)
        .ok()
        .and_then(|body| body.trim().parse().ok())
}

#[cfg(unix)]
fn preview_process_matches(pid: u32) -> bool {
    Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output()
        .is_ok_and(|output| String::from_utf8_lossy(&output.stdout).contains("preview latex"))
}

#[cfg(windows)]
fn preview_process_matches(pid: u32) -> bool {
    Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/NH"])
        .output()
        .is_ok_and(|output| {
            output.status.success()
                && String::from_utf8_lossy(&output.stdout).contains(&pid.to_string())
        })
}

#[cfg(unix)]
fn terminate_process(pid: u32) -> bool {
    Command::new("kill")
        .arg(pid.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
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

pub fn preview_service_name(config: &AiboxConfig) -> String {
    format!("{}-latex-preview", config.container.name)
}

fn http_url(ip: IpAddr, port: u16) -> String {
    format!("http://{}/", SocketAddr::new(ip, port))
}

fn preview_url(config: &AiboxConfig) -> String {
    let bind: IpAddr = config
        .latex
        .preview
        .bind
        .parse()
        .expect("validated LaTeX preview IP");
    http_url(browser_ip(bind), config.latex.preview.port)
}

fn browser_ip(bind: IpAddr) -> IpAddr {
    match bind {
        IpAddr::V4(ip) if ip.is_unspecified() => IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        IpAddr::V6(ip) if ip.is_unspecified() => IpAddr::V6(std::net::Ipv6Addr::LOCALHOST),
        ip => ip,
    }
}

fn preview_document_url(config: &AiboxConfig, name: &str) -> String {
    format!("{}documents/{name}/", preview_url(config))
}

pub fn is_managed_script(path: &Path) -> bool {
    fs::read_to_string(path).is_ok_and(|body| body.lines().nth(1) == Some(SCRIPT_HEADER))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_documents() -> AiboxConfig {
        let mut config = crate::config::test_config();
        config.latex.documents = vec![
            LatexDocument {
                name: "overview".into(),
                source: "docs/overview.tex".into(),
                output_dir: ".latex-cache/overview".into(),
            },
            LatexDocument {
                name: "appendix".into(),
                source: "docs/appendix file.tex".into(),
                output_dir: ".latex-cache/appendix output".into(),
            },
        ];
        config
    }

    #[test]
    fn generated_build_script_dispatches_all_documents_and_quotes_paths() {
        let script = build_script(&config_with_documents());
        assert!(script.contains("build_0\n    build_1"));
        assert!(script.contains("'docs/appendix file.tex'"));
        assert!(script.contains("'-outdir=.latex-cache/appendix output'"));
        assert!(script.contains("AIBOX_WORKSPACE:-/workspace"));
    }

    #[test]
    fn generated_watch_script_uses_foreground_latexmk_without_viewer() {
        let script = watch_script(&config_with_documents());
        assert!(script.contains("exec 'latexmk'"));
        assert!(script.contains("'-pvc' '-view=none'"));
        assert!(!script.contains("docker"));
    }

    #[test]
    fn generated_container_scripts_have_valid_posix_shell_syntax() {
        let root = tempfile::tempdir().unwrap();
        for (name, body) in [
            ("build", build_script(&config_with_documents())),
            ("watch", watch_script(&config_with_documents())),
        ] {
            let path = root.path().join(name);
            fs::write(&path, body).unwrap();
            let status = Command::new("sh").args(["-n"]).arg(&path).status().unwrap();
            assert!(status.success(), "generated {name} script is invalid");
        }
    }

    #[test]
    fn runtime_projection_contains_scripts_only_for_configured_documents() {
        let configured = config_with_documents();
        let configured_paths = crate::seed::managed_runtime_files(&configured)
            .into_iter()
            .map(|(path, _)| path)
            .collect::<Vec<_>>();
        assert!(configured_paths.contains(&PathBuf::from(BUILD_SCRIPT_PATH)));
        assert!(configured_paths.contains(&PathBuf::from(WATCH_SCRIPT_PATH)));

        let unconfigured = crate::config::test_config();
        let unconfigured_paths = crate::seed::managed_runtime_files(&unconfigured)
            .into_iter()
            .map(|(path, _)| path)
            .collect::<Vec<_>>();
        assert!(!unconfigured_paths.contains(&PathBuf::from(BUILD_SCRIPT_PATH)));
        assert!(!unconfigured_paths.contains(&PathBuf::from(WATCH_SCRIPT_PATH)));
    }

    #[test]
    fn tectonic_watch_script_explains_that_watch_is_unsupported() {
        let mut config = config_with_documents();
        config.latex.engine = LatexEngine::Tectonic;
        let script = watch_script(&config);
        assert!(script.contains("tectonic supports build only"));
        assert!(!script.contains("exec 'tectonic'"));
    }

    #[test]
    fn preview_urls_are_valid_for_ipv4_and_ipv6() {
        assert_eq!(
            http_url("127.0.0.1".parse().unwrap(), 8765),
            "http://127.0.0.1:8765/"
        );
        assert_eq!(http_url("::1".parse().unwrap(), 8765), "http://[::1]:8765/");
        assert_eq!(
            browser_ip("0.0.0.0".parse().unwrap()).to_string(),
            "127.0.0.1"
        );
    }

    #[test]
    fn runtime_helper_pins_embedpdf_and_uses_sse() {
        let helper = include_str!("../../images/base-debian/config/bin/aibox-latex-preview.py");
        assert!(helper.contains("EMBEDPDF_VERSION = \"2.14.3\""));
        assert!(helper.contains("new EventSource('{events_url}')"));
        assert!(helper.contains("ThreadingHTTPServer"));
        assert!(helper.contains("scroll?.forDocument('live-preview')"));
        assert!(helper.contains("!event.isInitial"));
        assert!(helper.contains("let restoringView = true"));
    }

    #[test]
    fn guidance_is_a_conditional_agents_block_with_container_commands() {
        let root = tempfile::tempdir().unwrap();
        let config = config_with_documents();
        fs::write(
            root.path().join(crate::processkit_vocab::AGENTS_FILENAME),
            "# Project\n",
        )
        .unwrap();
        fs::write(
            root.path().join(LEGACY_GUIDANCE_FILE),
            format!("{LEGACY_GUIDANCE_HEADER}\nlegacy guidance\n"),
        )
        .unwrap();
        sync_agent_guidance(&config, root.path()).unwrap();
        let body =
            fs::read_to_string(root.path().join(crate::processkit_vocab::AGENTS_FILENAME)).unwrap();
        assert!(body.contains("aibox-latex-build overview"));
        assert!(body.contains("aibox-latex-watch overview"));
        assert!(body.contains("http://test-proj-latex-preview:8765/health"));
        assert!(body.contains(AGENTS_GUIDANCE_BEGIN));
        assert!(body.contains(AGENTS_GUIDANCE_END));
        assert!(!body.contains("aibox latex"));
        assert!(!body.contains("aibox preview"));
        assert!(!root.path().join(LEGACY_GUIDANCE_FILE).exists());

        sync_agent_guidance(&config, root.path()).unwrap();
        let repeated =
            fs::read_to_string(root.path().join(crate::processkit_vocab::AGENTS_FILENAME)).unwrap();
        assert_eq!(repeated.matches(AGENTS_GUIDANCE_BEGIN).count(), 1);
    }

    #[test]
    fn disabled_latex_removes_only_its_managed_agents_block() {
        let root = tempfile::tempdir().unwrap();
        let config = config_with_documents();
        let agents = root.path().join(crate::processkit_vocab::AGENTS_FILENAME);
        fs::write(&agents, "# Project\n").unwrap();
        sync_agent_guidance(&config, root.path()).unwrap();

        let disabled = crate::config::test_config();
        sync_agent_guidance(&disabled, root.path()).unwrap();
        assert_eq!(fs::read_to_string(agents).unwrap(), "# Project");
    }
}
