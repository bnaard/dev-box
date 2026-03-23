// Composable process packages and convenience presets.
//
// Each package bundles a set of skills and optional context artifacts.
// Users compose packages freely via `[process] packages` in dev-box.toml.
// Presets expand to a list of packages for common workflows.

use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A composable process package.
#[derive(Debug)]
pub struct ProcessPackage {
    pub name: &'static str,
    pub description: &'static str,
    pub skills: &'static [&'static str],
    pub context_files: &'static [ContextFileDef],
    pub directories: &'static [&'static str],
}

/// A context file to scaffold when a package is selected.
#[derive(Debug)]
pub struct ContextFileDef {
    pub path: &'static str,
    pub template_key: &'static str,
}

/// A convenience preset that expands to a list of package names.
#[derive(Debug)]
pub struct ProcessPreset {
    pub name: &'static str,
    pub description: &'static str,
    pub packages: &'static [&'static str],
}

// ---------------------------------------------------------------------------
// Core skills — these can never be excluded
// ---------------------------------------------------------------------------

pub const CORE_SKILLS: &[&str] = &["agent-management", "owner-profile"];

// ---------------------------------------------------------------------------
// Package registry (13 packages)
// ---------------------------------------------------------------------------

static PACKAGES: &[ProcessPackage] = &[
    // 1. core — always included
    ProcessPackage {
        name: "core",
        description: "Essential agent management and owner profile (always present)",
        skills: &["agent-management", "owner-profile"],
        context_files: &[
            ContextFileDef {
                path: "context/DEVBOX.md",
                template_key: "devbox_md",
            },
            ContextFileDef {
                path: "context/OWNER.md",
                template_key: "owner_md",
            },
        ],
        directories: &["context/"],
    },
    // 2. tracking
    ProcessPackage {
        name: "tracking",
        description: "Backlog, decisions, and event tracking",
        skills: &["backlog-context", "decisions-adr", "event-log"],
        context_files: &[
            ContextFileDef {
                path: "context/BACKLOG.md",
                template_key: "backlog_md",
            },
            ContextFileDef {
                path: "context/DECISIONS.md",
                template_key: "decisions_md",
            },
            ContextFileDef {
                path: "context/EVENTLOG.md",
                template_key: "eventlog_md",
            },
        ],
        directories: &["context/"],
    },
    // 3. standups
    ProcessPackage {
        name: "standups",
        description: "Daily standup context and notes",
        skills: &["standup-context"],
        context_files: &[ContextFileDef {
            path: "context/STANDUPS.md",
            template_key: "standups_md",
        }],
        directories: &["context/"],
    },
    // 4. handover
    ProcessPackage {
        name: "handover",
        description: "Session and inter-agent handover protocols",
        skills: &["session-handover", "inter-agent-handover"],
        context_files: &[ContextFileDef {
            path: "context/project-notes/session-template.md",
            template_key: "session_template_md",
        }],
        directories: &["context/project-notes/"],
    },
    // 5. product
    ProcessPackage {
        name: "product",
        description: "Product planning, estimation, and retrospectives",
        skills: &["estimation-planning", "retrospective"],
        context_files: &[
            ContextFileDef {
                path: "context/PRD.md",
                template_key: "prd_md",
            },
            ContextFileDef {
                path: "context/PROJECTS.md",
                template_key: "projects_md",
            },
        ],
        directories: &["context/"],
    },
    // 6. code
    ProcessPackage {
        name: "code",
        description: "Software development practices and workflows",
        skills: &[
            "code-review",
            "testing-strategy",
            "debugging",
            "refactoring",
            "tdd-workflow",
            "error-handling",
            "git-workflow",
            "integration-testing",
        ],
        context_files: &[ContextFileDef {
            path: "context/work-instructions/DEVELOPMENT.md",
            template_key: "development_md",
        }],
        directories: &["context/work-instructions/"],
    },
    // 7. research
    ProcessPackage {
        name: "research",
        description: "Data science, visualization, and feature engineering",
        skills: &[
            "data-science",
            "data-visualization",
            "feature-engineering",
        ],
        context_files: &[ContextFileDef {
            path: "context/PROGRESS.md",
            template_key: "progress_md",
        }],
        directories: &[
            "context/research/",
            "context/analysis/",
            "experiments/",
        ],
    },
    // 8. documentation
    ProcessPackage {
        name: "documentation",
        description: "Technical writing and LaTeX authoring",
        skills: &["documentation", "latex-authoring"],
        context_files: &[],
        directories: &[],
    },
    // 9. design
    ProcessPackage {
        name: "design",
        description: "Visual design, diagrams, and frontend/mobile design",
        skills: &[
            "excalidraw",
            "infographics",
            "logo-design",
            "frontend-design",
            "mobile-app-design",
        ],
        context_files: &[],
        directories: &[],
    },
    // 10. architecture
    ProcessPackage {
        name: "architecture",
        description: "Software and system architecture patterns",
        skills: &[
            "software-architecture",
            "system-design",
            "domain-driven-design",
            "event-driven-architecture",
        ],
        context_files: &[],
        directories: &[],
    },
    // 11. security
    ProcessPackage {
        name: "security",
        description: "Secure coding, threat modeling, and dependency management",
        skills: &[
            "secure-coding",
            "threat-modeling",
            "dependency-audit",
            "auth-patterns",
            "secret-management",
            "dependency-management",
        ],
        context_files: &[],
        directories: &[],
    },
    // 12. data
    ProcessPackage {
        name: "data",
        description: "Data pipelines, quality, and vector databases",
        skills: &[
            "data-pipeline",
            "data-quality",
            "pandas-polars",
            "embedding-vectordb",
        ],
        context_files: &[],
        directories: &[],
    },
    // 13. operations
    ProcessPackage {
        name: "operations",
        description: "CI/CD, container orchestration, monitoring, and incident response",
        skills: &[
            "ci-cd-setup",
            "dockerfile-review",
            "container-orchestration",
            "logging-strategy",
            "metrics-monitoring",
            "incident-response",
            "alerting-oncall",
            "performance-profiling",
        ],
        context_files: &[ContextFileDef {
            path: "context/work-instructions/TEAM.md",
            template_key: "team_md",
        }],
        directories: &["context/work-instructions/"],
    },
];

// ---------------------------------------------------------------------------
// Preset registry (4 presets)
// ---------------------------------------------------------------------------

static PRESETS: &[ProcessPreset] = &[
    ProcessPreset {
        name: "managed",
        description: "Lightweight managed workflow (core + tracking + standups + handover)",
        packages: &["core", "tracking", "standups", "handover"],
    },
    ProcessPreset {
        name: "software",
        description: "Software development (managed + code + architecture)",
        packages: &[
            "core",
            "tracking",
            "standups",
            "handover",
            "code",
            "architecture",
        ],
    },
    ProcessPreset {
        name: "research-project",
        description: "Research workflow (managed + research + documentation)",
        packages: &[
            "core",
            "tracking",
            "standups",
            "handover",
            "research",
            "documentation",
        ],
    },
    ProcessPreset {
        name: "full-product",
        description: "Full product development (managed + code + architecture + design + product + security + operations)",
        packages: &[
            "core",
            "tracking",
            "standups",
            "handover",
            "code",
            "architecture",
            "design",
            "product",
            "security",
            "operations",
        ],
    },
];

// ---------------------------------------------------------------------------
// Lookup functions
// ---------------------------------------------------------------------------

/// Returns all 13 process packages.
pub fn all_packages() -> &'static [ProcessPackage] {
    PACKAGES
}

/// Look up a package by name.
pub fn get_package(name: &str) -> Option<&'static ProcessPackage> {
    PACKAGES.iter().find(|p| p.name == name)
}

/// Returns all 4 convenience presets.
pub fn all_presets() -> &'static [ProcessPreset] {
    PRESETS
}

/// Look up a preset by name.
pub fn get_preset(name: &str) -> Option<&'static ProcessPreset> {
    PRESETS.iter().find(|p| p.name == name)
}

// ---------------------------------------------------------------------------
// Resolution logic
// ---------------------------------------------------------------------------

/// Takes user input (a mix of package names and preset names), expands
/// presets, deduplicates, and returns an ordered list of packages.
/// The `core` package is always included.
pub fn resolve_packages(input: &[String]) -> Result<Vec<&'static ProcessPackage>, String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    // Always start with core
    seen.insert("core");

    // Collect all package names from input (expanding presets)
    let mut package_names: Vec<&str> = vec!["core"];
    for token in input {
        if let Some(preset) = get_preset(token) {
            for &pkg_name in preset.packages {
                package_names.push(pkg_name);
            }
        } else if get_package(token).is_some() {
            package_names.push(token);
        } else {
            return Err(format!(
                "Unknown package or preset: '{}'. Use a package name ({}) or preset name ({}).",
                token,
                all_packages()
                    .iter()
                    .map(|p| p.name)
                    .collect::<Vec<_>>()
                    .join(", "),
                all_presets()
                    .iter()
                    .map(|p| p.name)
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }
    }

    // Deduplicate while preserving order
    for name in &package_names {
        let dominated = seen.insert(*name) || (*name == "core" && result.is_empty());
        if let Some(pkg) = dominated.then(|| get_package(name)).flatten() {
            result.push(pkg);
        }
    }

    // Ensure core is first if not already
    let needs_core = result.is_empty() || result[0].name != "core";
    if let Some(core_pkg) = needs_core.then(|| get_package("core")).flatten() {
        result.insert(0, core_pkg);
    }

    Ok(result)
}

/// Computes the effective skill set from selected packages, include, and exclude lists.
///
/// Resolution order:
///   1. Union all skills from all packages
///   2. Add `include` skills
///   3. Remove `exclude` skills
///
/// Returns an error if the user tries to exclude core skills.
pub fn resolve_skills(
    packages: &[&ProcessPackage],
    include: &[String],
    exclude: &[String],
) -> Result<Vec<String>, String> {
    // Check for attempts to exclude core skills
    for skill in exclude {
        if CORE_SKILLS.contains(&skill.as_str()) {
            return Err(format!(
                "Cannot exclude core skill '{}'. Core skills ({}) are always required.",
                skill,
                CORE_SKILLS.join(", "),
            ));
        }
    }

    let exclude_set: HashSet<&str> = exclude.iter().map(|s| s.as_str()).collect();

    // Union all package skills
    let mut skill_set: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for pkg in packages {
        for &skill in pkg.skills {
            if seen.insert(skill.to_string()) {
                skill_set.push(skill.to_string());
            }
        }
    }

    // Add include skills
    for skill in include {
        if seen.insert(skill.clone()) {
            skill_set.push(skill.clone());
        }
    }

    // Remove exclude skills
    skill_set.retain(|s| !exclude_set.contains(s.as_str()));

    Ok(skill_set)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_packages_count() {
        assert_eq!(all_packages().len(), 13);
    }

    #[test]
    fn test_all_presets_count() {
        assert_eq!(all_presets().len(), 4);
    }

    #[test]
    fn test_get_package_found() {
        assert!(get_package("core").is_some());
        assert!(get_package("operations").is_some());
        assert!(get_package("research").is_some());
    }

    #[test]
    fn test_get_package_not_found() {
        assert!(get_package("nonexistent").is_none());
    }

    #[test]
    fn test_get_preset_found() {
        assert!(get_preset("managed").is_some());
        assert!(get_preset("full-product").is_some());
    }

    #[test]
    fn test_get_preset_not_found() {
        assert!(get_preset("nonexistent").is_none());
    }

    #[test]
    fn test_package_names_unique() {
        let names: Vec<&str> = all_packages().iter().map(|p| p.name).collect();
        let unique: HashSet<&str> = names.iter().copied().collect();
        assert_eq!(names.len(), unique.len(), "Package names must be unique");
    }

    #[test]
    fn test_preset_names_unique() {
        let names: Vec<&str> = all_presets().iter().map(|p| p.name).collect();
        let unique: HashSet<&str> = names.iter().copied().collect();
        assert_eq!(names.len(), unique.len(), "Preset names must be unique");
    }

    #[test]
    fn test_preset_packages_all_exist() {
        for preset in all_presets() {
            for &pkg_name in preset.packages {
                assert!(
                    get_package(pkg_name).is_some(),
                    "Preset '{}' references unknown package '{}'",
                    preset.name,
                    pkg_name,
                );
            }
        }
    }

    // -- resolve_packages tests --

    #[test]
    fn test_resolve_empty_input_gives_core() {
        let result = resolve_packages(&[]).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "core");
    }

    #[test]
    fn test_resolve_core_always_first() {
        let input = vec!["code".to_string(), "tracking".to_string()];
        let result = resolve_packages(&input).unwrap();
        assert_eq!(result[0].name, "core");
    }

    #[test]
    fn test_resolve_preset_expands() {
        let input = vec!["managed".to_string()];
        let result = resolve_packages(&input).unwrap();
        let names: Vec<&str> = result.iter().map(|p| p.name).collect();
        assert!(names.contains(&"core"));
        assert!(names.contains(&"tracking"));
        assert!(names.contains(&"standups"));
        assert!(names.contains(&"handover"));
    }

    #[test]
    fn test_resolve_deduplicates() {
        let input = vec![
            "managed".to_string(),
            "core".to_string(),
            "tracking".to_string(),
        ];
        let result = resolve_packages(&input).unwrap();
        let names: Vec<&str> = result.iter().map(|p| p.name).collect();
        // core should appear exactly once
        assert_eq!(names.iter().filter(|&&n| n == "core").count(), 1);
        assert_eq!(names.iter().filter(|&&n| n == "tracking").count(), 1);
    }

    #[test]
    fn test_resolve_unknown_input_errors() {
        let input = vec!["bogus".to_string()];
        let result = resolve_packages(&input);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown package or preset"));
    }

    #[test]
    fn test_resolve_mixed_preset_and_package() {
        let input = vec!["managed".to_string(), "security".to_string()];
        let result = resolve_packages(&input).unwrap();
        let names: Vec<&str> = result.iter().map(|p| p.name).collect();
        assert!(names.contains(&"core"));
        assert!(names.contains(&"tracking"));
        assert!(names.contains(&"security"));
    }

    #[test]
    fn test_resolve_software_preset() {
        let input = vec!["software".to_string()];
        let result = resolve_packages(&input).unwrap();
        let names: Vec<&str> = result.iter().map(|p| p.name).collect();
        assert!(names.contains(&"core"));
        assert!(names.contains(&"code"));
        assert!(names.contains(&"architecture"));
    }

    #[test]
    fn test_resolve_full_product_preset() {
        let input = vec!["full-product".to_string()];
        let result = resolve_packages(&input).unwrap();
        let names: Vec<&str> = result.iter().map(|p| p.name).collect();
        assert!(names.contains(&"core"));
        assert!(names.contains(&"code"));
        assert!(names.contains(&"design"));
        assert!(names.contains(&"product"));
        assert!(names.contains(&"security"));
        assert!(names.contains(&"operations"));
    }

    // -- resolve_skills tests --

    #[test]
    fn test_resolve_skills_basic() {
        let core = get_package("core").unwrap();
        let tracking = get_package("tracking").unwrap();
        let packages = vec![core, tracking];
        let skills = resolve_skills(&packages, &[], &[]).unwrap();
        assert!(skills.contains(&"agent-management".to_string()));
        assert!(skills.contains(&"owner-profile".to_string()));
        assert!(skills.contains(&"backlog-context".to_string()));
        assert!(skills.contains(&"decisions-adr".to_string()));
        assert!(skills.contains(&"event-log".to_string()));
    }

    #[test]
    fn test_resolve_skills_include() {
        let core = get_package("core").unwrap();
        let packages = vec![core];
        let include = vec!["flutter-development".to_string()];
        let skills = resolve_skills(&packages, &include, &[]).unwrap();
        assert!(skills.contains(&"flutter-development".to_string()));
    }

    #[test]
    fn test_resolve_skills_exclude() {
        let core = get_package("core").unwrap();
        let tracking = get_package("tracking").unwrap();
        let packages = vec![core, tracking];
        let exclude = vec!["event-log".to_string()];
        let skills = resolve_skills(&packages, &[], &exclude).unwrap();
        assert!(!skills.contains(&"event-log".to_string()));
        // Core skills still present
        assert!(skills.contains(&"agent-management".to_string()));
    }

    #[test]
    fn test_resolve_skills_cannot_exclude_core() {
        let core = get_package("core").unwrap();
        let packages = vec![core];
        let exclude = vec!["agent-management".to_string()];
        let result = resolve_skills(&packages, &[], &exclude);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot exclude core skill"));
    }

    #[test]
    fn test_resolve_skills_cannot_exclude_owner_profile() {
        let core = get_package("core").unwrap();
        let packages = vec![core];
        let exclude = vec!["owner-profile".to_string()];
        let result = resolve_skills(&packages, &[], &exclude);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_skills_deduplicates() {
        let code = get_package("code").unwrap();
        let packages = vec![code, code]; // same package twice
        let skills = resolve_skills(&packages, &[], &[]).unwrap();
        let unique: HashSet<&String> = skills.iter().collect();
        assert_eq!(skills.len(), unique.len(), "Skills must be deduplicated");
    }

    #[test]
    fn test_resolve_skills_include_already_present() {
        let core = get_package("core").unwrap();
        let packages = vec![core];
        let include = vec!["agent-management".to_string()]; // already in core
        let skills = resolve_skills(&packages, &include, &[]).unwrap();
        let count = skills
            .iter()
            .filter(|s| *s == "agent-management")
            .count();
        assert_eq!(count, 1, "Duplicate include should be deduplicated");
    }

    #[test]
    fn test_code_package_skills() {
        let code = get_package("code").unwrap();
        assert_eq!(code.skills.len(), 8);
        assert!(code.skills.contains(&"tdd-workflow"));
        assert!(code.skills.contains(&"integration-testing"));
    }

    #[test]
    fn test_operations_package_skills() {
        let ops = get_package("operations").unwrap();
        assert_eq!(ops.skills.len(), 8);
        assert!(ops.skills.contains(&"alerting-oncall"));
        assert!(ops.skills.contains(&"performance-profiling"));
    }

    #[test]
    fn test_core_context_files() {
        let core = get_package("core").unwrap();
        assert_eq!(core.context_files.len(), 2);
        assert_eq!(core.context_files[0].path, "context/DEVBOX.md");
        assert_eq!(core.context_files[1].path, "context/OWNER.md");
    }

    #[test]
    fn test_research_directories() {
        let research = get_package("research").unwrap();
        assert!(research.directories.contains(&"context/research/"));
        assert!(research.directories.contains(&"experiments/"));
    }
}
