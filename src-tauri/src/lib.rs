use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

fn base_output_dir() -> PathBuf {
    let base = dirs_next().unwrap_or_else(|| PathBuf::from("/tmp"));
    base.join(".bio-om-expert").join("output")
}

// ── Skill Manifest (loaded at runtime, built-in fallback) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkillManifest {
    name: String,
    display: String,
    description: String,
    trigger_patterns: Vec<String>,
    cli_invoke: String,
    estimated_time: String,
    #[serde(default)]
    depends_on: Vec<String>,
    #[serde(default)]
    required_args: Vec<String>,
    #[serde(default)]
    produces: String,
    #[serde(default)]
    output_pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExecutionStep {
    skill: String,
    display: String,
    description: String,
    produces: String,
    prompt: String,
}

fn skill_dir(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap_or_else(|_| dirs_next().unwrap_or_default())
        .join("skills")
}

fn seed_default_skills(app: &AppHandle) {
    let dir = skill_dir(app);
    if !dir.exists() {
        fs::create_dir_all(&dir).ok();

        let builtins: &[(&str, &str)] = &[
            ("web-research.json", include_str!("../../skills-manifest/web-research.json")),
            ("url-research.json", include_str!("../../skills-manifest/url-research.json")),
            ("local-research.json", include_str!("../../skills-manifest/local-research.json")),
            ("report-generator.json", include_str!("../../skills-manifest/report-generator.json")),
            ("content-writing.json", include_str!("../../skills-manifest/content-writing.json")),
        ];
        for (name, content) in builtins {
            let path = dir.join(name);
            if !path.exists() {
                fs::write(&path, content).ok();
            }
        }
    }
    // Always seed SKILL.md and CLAUDE.md to keep them up-to-date
    seed_claude_skills();
}

fn seed_claude_skills() {
    let home = dirs_next().unwrap_or_else(|| PathBuf::from("/tmp"));
    let claude_skills_dir = home.join(".claude").join("skills");

    let skill_mds: &[(&str, &str)] = &[
        ("web-research/SKILL.md", include_str!("../../skills/web-research.md")),
        ("url-research/SKILL.md", include_str!("../../skills/url-research.md")),
        ("local-research/SKILL.md", include_str!("../../skills/local-research.md")),
        ("report-generator/SKILL.md", include_str!("../../skills/report-generator.md")),
        ("content-writing/SKILL.md", include_str!("../../skills/content-writing.md")),
    ];

    for (rel_path, content) in skill_mds {
        let path = claude_skills_dir.join(rel_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&path, content).ok();
    }

    // Seed CLAUDE.md
    let claude_md = include_str!("../../CLAUDE.md");
    fs::create_dir_all(home.join(".claude")).ok();
    fs::write(home.join(".claude").join("CLAUDE.md"), claude_md).ok();
}

fn load_builtin_manifests() -> Vec<SkillManifest> {
    let json_files = [
        include_str!("../../skills-manifest/web-research.json"),
        include_str!("../../skills-manifest/url-research.json"),
        include_str!("../../skills-manifest/local-research.json"),
        include_str!("../../skills-manifest/report-generator.json"),
        include_str!("../../skills-manifest/content-writing.json"),
    ];
    json_files.iter()
        .filter_map(|s| serde_json::from_str::<SkillManifest>(s).ok())
        .collect()
}

fn load_manifests(app: &AppHandle) -> (Vec<SkillManifest>, Vec<String>) {
    let dir = skill_dir(app);
    let mut manifests: Vec<SkillManifest> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    if dir.exists() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json") {
                    match fs::read_to_string(&path) {
                        Ok(content) => {
                            match serde_json::from_str::<SkillManifest>(&content) {
                                Ok(m) => manifests.push(m),
                                Err(e) => {
                                    let name = path.file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_else(|| "unknown".to_string());
                                    errors.push(format!(
                                        "{}: invalid JSON — {}",
                                        name, e
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            errors.push(format!(
                                "{}: read failed — {}",
                                path.display(), e
                            ));
                        }
                    }
                }
            }
        }
    }

    // Fall back to built-in manifests if no external ones found
    if manifests.is_empty() {
        manifests = load_builtin_manifests();
    }

    (manifests, errors)
}

fn match_skills(input: &str, manifests: &[SkillManifest]) -> Vec<(SkillManifest, u32)> {
    let input_lower = input.to_lowercase();
    let mut scored: Vec<(SkillManifest, u32)> = manifests.iter().map(|m| {
        let score = m.trigger_patterns.iter()
            .filter(|p| input_lower.contains(&p.to_lowercase()))
            .count() as u32;
        (m.clone(), score)
    }).collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored
}

fn resolve_dependencies<'a>(
    skill: &'a SkillManifest,
    manifests: &'a [SkillManifest],
    seen: &mut Vec<String>,
) -> Vec<&'a SkillManifest> {
    let mut chain: Vec<&SkillManifest> = Vec::new();
    for dep_name in &skill.depends_on {
        // Parse pipe-separated alternatives: "report-generator | web-research" means either
        let alternatives: Vec<&str> = dep_name.split('|').map(|s| s.trim()).collect();
        let resolved = alternatives.iter().find_map(|alt| {
            manifests.iter().find(|m| m.name == *alt)
        });
        if let Some(dep) = resolved {
            if !seen.contains(&dep.name) {
                seen.push(dep.name.clone());
                // Recurse: dependencies of dependencies
                let sub_chain = resolve_dependencies(dep, manifests, seen);
                chain.extend(sub_chain);
                chain.push(dep);
            }
        }
    }
    chain
}

fn orchestrate_via_claude(input: &str, manifests: &[SkillManifest]) -> Option<Vec<ExecutionStep>> {
    let manifest_text: String = manifests.iter().map(|m| {
        format!("- **{}** ({})\n  描述: {}\n  产出: {}\n  依赖: {}",
            m.name, m.display, m.description, m.produces,
            if m.depends_on.is_empty() { "无".to_string() } else { m.depends_on.join(", ") })
    }).collect::<Vec<_>>().join("\n\n");

    let prompt = format!(
        "你是一个内容运营工作流的编排器。用户输入了一段自然语言需求，你需要判断应该执行哪些 skill。\n\n\
         ## 可用 skills\n\n{}\n\n\
         ## 依赖规则\n\
         - report-generator 依赖 web-research / url-research / local-research（至少选一个）\n\
         - content-writing 依赖 report-generator 或 web-research（至少选一个）\n\
         - 研究类 skill（web/url/local-research）可以独立执行\n\n\
         ## 编排要求\n\
         - 如果用户需要内容产出（推文/脚本/文案），确保包含 content-writing\n\
         - 如果用户需要研究数据但没有指定来源，默认使用 web-research\n\
         - 依赖 skill 必须排在被依赖 skill 之前\n\
         - 每个 skill 的 prompt 要具体，包含用户需求的关键信息\n\n\
         用户输入: \"{}\"\n\n\
         请以 JSON 数组格式返回执行计划，每个元素包含 skill(技能名)、prompt(给该 skill 的具体指令)。\n\
         只返回 JSON，不要其他文字。例如:\n\
         [{{\"skill\":\"content-writing\",\"prompt\":\"topic=SOD抗氧化机制科普\"}}]",
        manifest_text, input
    );

    let output = Command::new("claude")
        .arg("-p").arg(&prompt).arg("--output-format").arg("text")
        .stdout(Stdio::piped()).stderr(Stdio::piped())
        .output().ok()?;

    if !output.status.success() { return None; }

    let text = String::from_utf8_lossy(&output.stdout);
    // Extract JSON array from response
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            let json_str = &text[start..=end];
            if let Ok(steps) = serde_json::from_str::<Vec<ExecutionStepRaw>>(json_str) {
                let manifest_map: HashMap<String, SkillManifest> = manifests.iter()
                    .map(|m| (m.name.clone(), m.clone()))
                    .collect();
                return Some(steps.into_iter().filter_map(|s| {
                    manifest_map.get(&s.skill).map(|m| ExecutionStep {
                        skill: s.skill,
                        display: m.display.clone(),
                        description: m.description.clone(),
                        produces: m.produces.clone(),
                        prompt: s.prompt,
                    })
                }).collect());
            }
        }
    }
    None
}

#[derive(Debug, Deserialize)]
struct ExecutionStepRaw {
    skill: String,
    prompt: String,
}

// ── Utilities ──

fn make_run_dir(topic: &str) -> String {
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let safe_topic: String = topic.chars().take(20)
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
    let base = base_output_dir().to_string_lossy().to_string();
    let dir = format!("{}/{}_{}", base, safe_topic, ts);
    fs::create_dir_all(&dir).ok();
    dir
}

fn history_dir() -> PathBuf {
    dirs_next().unwrap_or_else(|| PathBuf::from("/tmp")).join(".bio-om-expert")
}

fn history_file() -> PathBuf { history_dir().join("history.json") }

fn ensure_python_script() -> PathBuf {
    let cache_dir = history_dir().join("scripts");
    fs::create_dir_all(&cache_dir).ok();
    let script_path = cache_dir.join("generate_docx.py");
    if !script_path.exists() {
        let content = include_str!("../../scripts/generate_docx.py");
        fs::write(&script_path, content).ok();
    }
    script_path
}

const MIN_CLAUDE_VERSION: &str = "1.0.0";

fn check_claude_installed() -> bool {
    Command::new("claude").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

fn get_claude_version() -> Option<String> {
    Command::new("claude")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                let combined = format!("{}{}", stdout, stderr);
                // Extract semver-like version from output
                let v = combined.split_whitespace()
                    .find(|w| w.chars().filter(|c| *c == '.').count() >= 1
                        && w.chars().all(|c| c.is_ascii_digit() || c == '.'))
                    .map(|s| s.to_string());
                // Also try "claude" prefix: "claude/1.2.3"
                v.or_else(|| {
                    combined.lines()
                        .find(|l| l.contains("claude"))
                        .and_then(|l| l.split('/').nth(1))
                        .map(|s| s.trim().to_string())
                })
            } else {
                None
            }
        })
}

fn version_at_least(version: &str, min: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };
    let v = parse(version);
    let m = parse(min);
    for i in 0..m.len().min(v.len()) {
        if v[i] > m[i] { return true; }
        if v[i] < m[i] { return false; }
    }
    v.len() >= m.len()
}

fn sanitize_input(input: &str) -> String {
    // Strip control characters (except newlines and tabs) and limit length
    input.chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .take(2000)
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryEntry {
    id: String, topic: String, output_dir: String,
    status: String, files: Vec<String>, created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryStore { entries: Vec<HistoryEntry> }

struct AppState {
    running: Mutex<bool>,
    active_pid: Mutex<Option<u32>>,
    run_id: Mutex<u64>,
}

fn load_history() -> HistoryStore {
    let path = history_file();
    if path.exists() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(store) = serde_json::from_str::<HistoryStore>(&data) {
                return store;
            }
        }
    }
    HistoryStore { entries: vec![] }
}

fn save_history_entry(entry: HistoryEntry) {
    let mut store = load_history();
    store.entries.insert(0, entry);
    if store.entries.len() > 50 { store.entries.truncate(50); }
    fs::create_dir_all(&history_dir()).ok();
    if let Ok(json) = serde_json::to_string_pretty(&store) {
        fs::write(history_file(), json).ok();
    }
}

fn rename_to_chinese(dir: &str) {
    let mapping = [
        ("article_draft.md", "推文草稿.md"),
        ("article_interaction.md", "图文互动设计.md"),
        ("video_script.md", "视频脚本.md"),
        ("voiceover.md", "口播稿.md"),
        ("article_engage.md", "视频互动设计.md"),
        ("image_suggestions.md", "配图建议.md"),
        ("article_headlines.md", "标题备选.md"),
        ("script_scenes.md", "分镜提示词.md"),
        ("research_report.md", "研究报告.md"),
    ];
    for (en, cn) in &mapping {
        let en_path = format!("{}/{}", dir, en);
        let cn_path = format!("{}/{}", dir, cn);
        if PathBuf::from(&en_path).exists() {
            fs::rename(&en_path, &cn_path).ok();
        }
    }
}

fn scan_output_files(dir: &str) -> Vec<String> {
    let mut files = vec![];
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with('.') {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                files.push(format!("{} ({}KB)", name, size / 1024));
            }
        }
    }
    files.sort();
    files
}

fn generate_combined_docx(output_dir: &str) -> Option<String> {
    let script = ensure_python_script();
    let docx_path = format!("{}/_合集输出.docx", output_dir);
    let output = Command::new("python3")
        .arg(script.to_string_lossy().to_string())
        .arg(output_dir).arg(&docx_path)
        .stdout(Stdio::piped()).stderr(Stdio::piped()).output().ok()?;
    if output.status.success() { Some(docx_path) } else { None }
}

// ── Tauri Commands ──

#[tauri::command]
async fn orchestrate(app: AppHandle, input: String) -> Result<Vec<ExecutionStep>, String> {
    let (manifests, load_errors) = load_manifests(&app);

    // Report any manifest loading errors to the frontend
    for error in &load_errors {
        let _ = app.emit("skill-error", format!("[manifest] {}", error));
    }
    let scored = match_skills(&input, &manifests);

    // If we have a clear match (score > 0), use keyword matching
    if let Some((top, score)) = scored.first() {
        if *score > 0 {
            let second_score = scored.get(1).map(|(_, s)| *s).unwrap_or(0);
            if *score > second_score {
                // Build execution plan with dependencies resolved
                let mut seen: Vec<String> = Vec::new();
                seen.push(top.name.clone());
                let dep_chain = resolve_dependencies(top, &manifests, &mut seen);

                let mut steps: Vec<ExecutionStep> = Vec::new();
                for dep in dep_chain {
                    steps.push(ExecutionStep {
                        skill: dep.name.clone(),
                        display: dep.display.clone(),
                        description: dep.description.clone(),
                        produces: dep.produces.clone(),
                        prompt: format!(
                            "你正在执行 Bio-OM Expert 的「{}」技能。\n技能描述：{}\n预期产出：{}\n\n任务：基于用户需求「{}」执行该技能，将输出文件保存到指定目录。",
                            dep.display, dep.description, dep.produces, input
                        ),
                    });
                }

                // Add the main skill last (dependencies come first)
                let prompt = format!(
                    "你正在执行 Bio-OM Expert 的「{}」技能。\n技能描述：{}\n预期产出：{}\n\n任务：基于用户需求「{}」执行该技能，将输出文件保存到指定目录。",
                    top.display, top.description, top.produces, input
                );
                steps.push(ExecutionStep {
                    skill: top.name.clone(),
                    display: top.display.clone(),
                    description: top.description.clone(),
                    produces: top.produces.clone(),
                    prompt,
                });

                return Ok(steps);
            }
        }
    }

    // Fallback: ask Claude to decide
    if let Some(steps) = orchestrate_via_claude(&input, &manifests) {
        return Ok(steps);
    }

    // Ultimate fallback: default to content-writing
    let fallback = manifests.iter().find(|m| m.name == "content-writing")
        .cloned()
        .unwrap_or_else(|| SkillManifest {
            name: "content-writing".to_string(),
            display: "文案撰写与视频脚本".to_string(),
            description: "基于研究报告撰写推文、视频脚本、口播稿".to_string(),
            produces: "推文草稿 + 视频脚本 + 口播稿 + 互动设计".to_string(),
            trigger_patterns: vec![],
            cli_invoke: String::new(),
            estimated_time: String::new(),
            depends_on: vec![],
            required_args: vec![],
            output_pattern: String::new(),
        });
    Ok(vec![ExecutionStep {
        skill: fallback.name.clone(),
        display: fallback.display.clone(),
        description: fallback.description.clone(),
        produces: fallback.produces.clone(),
        prompt: format!(
            "你正在执行 Bio-OM Expert 的「{}」技能。\n技能描述：{}\n预期产出：{}\n\n任务：基于用户需求「{}」执行该技能，将输出文件保存到指定目录。",
            fallback.display, fallback.description, fallback.produces, input
        ),
    }])
}

#[tauri::command]
async fn run_pipeline(
    app: AppHandle,
    state: State<'_, AppState>,
    input: String,
    steps: Vec<ExecutionStep>,
) -> Result<String, String> {
    {
        let mut running = state.running.lock().map_err(|e| e.to_string())?;
        if *running { return Err("已有任务正在运行".to_string()); }
        *running = true;
    }

    let sanitized = sanitize_input(&input);
    let app_handle = app.clone();
    let output_dir = make_run_dir(&sanitized);
    let output_dir_ret = output_dir.clone();
    let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let history_run_id = format!("run-{}", chrono::Local::now().format("%Y%m%d%H%M%S"));
    let steps_count = steps.len();
    let total_steps = steps_count as u32 * 100;
    let steps_for_thread = steps.clone();
    // Increment run_id for cancellation tracking
    let my_run_id = {
        let state = app_handle.state::<AppState>();
        let mut id = state.run_id.lock().unwrap();
        *id = id.wrapping_add(1);
        *id
    };

    let _ = app_handle.emit("skill-progress", serde_json::json!({
        "step": 0, "total": total_steps, "name": "编排完成，开始执行..."
    }).to_string());

    std::thread::spawn(move || {
        let state = app_handle.state::<AppState>();
        let mut global_step;
        let mut all_success = true;
        let mut seen_lines: Vec<String> = Vec::new();

        for (i, step) in steps_for_thread.iter().enumerate() {
            let skill_base = i as u32 * 100;
            let _ = app_handle.emit("skill-output", &format!(
                "\n━━━ 第 {}/{} 步: {} ━━━", i + 1, steps_count, step.display
            ));
            let _ = app_handle.emit("skill-progress", serde_json::json!({
                "step": skill_base, "total": total_steps,
                "name": format!("正在执行: {}", step.display),
            }).to_string());

            let prompt = format!(
                "你正在执行 Bio-OM Expert 工作流的第 {step_num}/{total} 步：**{display}**。\n\n\
                 ## 技能说明\n{description}\n\n\
                 ## 预期产出\n{produces}\n\n\
                 ## 具体任务\n{task}\n\n\
                 ## 输出目录\n{output_dir}/\n\n\
                 ## 重要规则\n\
                 - 所有输出文件使用英文文件名（如 research_report.md、article_draft.md）\n\
                 - 唯一的例外是「配图建议.json」（使用中文文件名）\n\
                 - 不要创建中文文件名的 Markdown 副本\n\
                 - 将文件保存到指定的输出目录",
                step_num = i + 1, total = steps_count,
                display = step.display, description = step.description,
                produces = step.produces, task = step.prompt, output_dir = output_dir,
            );

            let mut child = match Command::new("claude")
                .arg("-p").arg(&prompt).arg("--output-format").arg("text")
                .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    let _ = app_handle.emit("skill-error",
                        format!("启动 {} 失败: {}", step.display, e));
                    all_success = false;
                    break;
                }
            };
            // Store PID so cancel_skill can kill this process
            *state.active_pid.lock().unwrap() = Some(child.id());

            if let Some(stdout) = child.stdout.take() {
                let mut reader = BufReader::new(stdout);
                let mut line_count = 0u32;
                let mut buf = String::new();
                loop {
                    // Check cancellation between reads
                    if !*state.running.lock().unwrap() { all_success = false; break; }
                    buf.clear();
                    match reader.read_line(&mut buf) {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            let line = buf.trim_end_matches(|c| c == '\n' || c == '\r').to_string();
                            if !line.is_empty() {
                                seen_lines.push(line.clone());
                                if seen_lines.len() > 200 { seen_lines.remove(0); }
                                line_count += 1;
                                if line_count % 20 == 0 {
                                    let sub_progress = ((line_count / 20) as u32).min(90);
                                    global_step = skill_base + sub_progress;
                                    let _ = app_handle.emit("skill-progress", serde_json::json!({
                                        "step": global_step, "total": total_steps,
                                        "name": format!("{} ({}行输出)", step.display, line_count),
                                    }).to_string());
                                }
                                let _ = app_handle.emit("skill-output", &line);
                            }
                        }
                        Err(_) => break,
                    }
                }
            }

            if let Some(stderr) = child.stderr.take() {
                for line in BufReader::new(stderr).lines().flatten() {
                    if !line.is_empty() && !seen_lines.contains(&line) {
                        let _ = app_handle.emit("skill-output",
                            &format!("[stderr] {}", line));
                    }
                }
            }

            match child.wait() {
                Ok(status) if !status.success() => {
                    // Check if user cancelled
                    let was_cancelled = !*state.running.lock().unwrap();
                    if !was_cancelled {
                        let _ = app_handle.emit("skill-error",
                            format!("{} 执行失败，退出码: {}",
                                step.display, status.code().unwrap_or(-1)));
                    }
                    all_success = false;
                    break;
                }
                Err(e) => {
                    let _ = app_handle.emit("skill-error",
                        format!("{} 异常: {}", step.display, e));
                    all_success = false;
                    break;
                }
                _ => {}
            }
            // Clear PID AFTER child exits (so cancel_skill can always find it)
            *state.active_pid.lock().unwrap() = None;

            // Check if cancelled before proceeding to next skill
            if !*state.running.lock().unwrap() { all_success = false; break; }

            // Skill complete — advance to end of this skill's range
            global_step = skill_base + 99;
            let _ = app_handle.emit("skill-progress", serde_json::json!({
                "step": global_step, "total": total_steps,
                "name": format!("✓ {} 完成", step.display),
            }).to_string());
        }

        let status_str = if all_success { "done" } else { "error" };
        if all_success { rename_to_chinese(&output_dir); }
        let docx_path = if all_success {
            generate_combined_docx(&output_dir)
        } else { None };
        let final_files = scan_output_files(&output_dir);

        save_history_entry(HistoryEntry {
            id: history_run_id, topic: input,
            output_dir: output_dir.clone(),
            status: status_str.to_string(),
            files: final_files, created_at: ts,
        });

        if all_success {
            let _ = app_handle.emit("skill-progress", serde_json::json!({
                "step": total_steps, "total": total_steps, "name": "完成",
            }).to_string());
            let _ = app_handle.emit("skill-done", serde_json::json!({
                "message": "全部任务完成",
                "output_dir": output_dir,
                "docx_path": docx_path,
            }).to_string());
        }

        // Only reset state if we're still the active run (not superseded by a new task)
        if *state.run_id.lock().unwrap() == my_run_id {
            *state.running.lock().unwrap() = false;
            *state.active_pid.lock().unwrap() = None;
        }
    });

    Ok(format!("编排完成，共 {} 步 → {}", steps.len(), output_dir_ret))
}

#[tauri::command]
async fn revise_output(
    app: AppHandle,
    output_dir: String,
    instruction: String,
) -> Result<String, String> {
    {
        let state = app.state::<AppState>();
        let mut running = state.running.lock().map_err(|e| e.to_string())?;
        if *running { return Err("已有任务正在运行".to_string()); }
        *running = true;
    }

    let app_handle = app.clone();
    let dir_for_thread = output_dir.clone();

    // List existing files for reference (filenames only, not content)
    let mut file_list = String::new();
    if let Ok(entries) = fs::read_dir(&output_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") && name != ".gitkeep" {
                file_list.push_str(&format!("- {}\n", name));
            }
        }
    }

    let _ = app_handle.emit("skill-progress", serde_json::json!({
        "step": 0, "total": 11, "name": "正在修改..."
    }).to_string());

    let prompt = format!(
        "之前已生成以下文件（在 {dir} 目录中）:\n{files}\n\n用户要求修改: {instruction}\n\n请读取 {dir}/ 目录中的已有文件，根据用户要求修改内容并重新生成所有文件。保持相同的文件名和格式。",
        dir = output_dir, files = file_list, instruction = instruction
    );

    // Increment run_id for cancellation tracking
    let my_run_id = {
        let state = app_handle.state::<AppState>();
        let mut id = state.run_id.lock().unwrap();
        *id = id.wrapping_add(1);
        *id
    };

    std::thread::spawn(move || {
        let state = app_handle.state::<AppState>();
        let mut seen_lines: Vec<String> = Vec::new();
        let mut child = match Command::new("claude")
            .arg("-p").arg(&prompt).arg("--output-format").arg("text")
            .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()
        {
            Ok(c) => c,
            Err(e) => {
                let _ = app_handle.emit("skill-error", format!("启动修改失败: {}", e));
                *state.running.lock().unwrap() = false;
                return;
            }
        };
        *state.active_pid.lock().unwrap() = Some(child.id());

        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout);
            let mut line_count = 0u32;
            let mut buf = String::new();
            loop {
                if !*state.running.lock().unwrap() { break; }
                buf.clear();
                match reader.read_line(&mut buf) {
                    Ok(0) => break,
                    Ok(_) => {
                        let line = buf.trim_end_matches(|c| c == '\n' || c == '\r').to_string();
                        if !line.is_empty() {
                            seen_lines.push(line.clone());
                            if seen_lines.len() > 200 { seen_lines.remove(0); }
                            line_count += 1;
                            if line_count % 20 == 0 {
                                let prog = (50u32 + (line_count / 20).min(45)).min(95);
                                let _ = app_handle.emit("skill-progress", serde_json::json!({
                                    "step": prog, "total": 100, "name": format!("修改中... ({}行)", line_count),
                                }).to_string());
                            }
                            let _ = app_handle.emit("skill-output", &line);
                        }
                    }
                    Err(_) => break,
                }
            }
        }

        if let Some(stderr) = child.stderr.take() {
            for line in BufReader::new(stderr).lines().flatten() {
                if !line.is_empty() && !seen_lines.contains(&line) {
                    let _ = app_handle.emit("skill-output", &format!("[stderr] {}", line));
                }
            }
        }

        let success = child.wait().map(|s| s.success()).unwrap_or(false);
        *state.active_pid.lock().unwrap() = None;

        // Check if cancelled
        if !*state.running.lock().unwrap() { return; }

        if success {
            rename_to_chinese(&dir_for_thread);
            generate_combined_docx(&dir_for_thread);
            let _ = app_handle.emit("skill-progress", serde_json::json!({
                "step": 100, "total": 100, "name": "修改完成",
            }).to_string());
            let _ = app_handle.emit("skill-done", serde_json::json!({
                "message": "修改完成",
                "output_dir": dir_for_thread,
                "docx_path": format!("{}/_合集输出.docx", dir_for_thread),
            }).to_string());
        } else {
            let _ = app_handle.emit("skill-error", "修改执行失败".to_string());
        }

        if *state.run_id.lock().unwrap() == my_run_id {
            *state.running.lock().unwrap() = false;
        }
    });

    Ok(format!("正在修改: {}", instruction))
}

#[tauri::command]
fn read_output_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("读取失败: {}", e))
}

#[tauri::command]
fn list_output_files(dir: String) -> Result<Vec<String>, String> {
    let mut files = vec![];
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".md") && name != ".gitkeep" {
                files.push(name);
            }
        }
    }
    files.sort();
    Ok(files)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DashboardAsset {
    path: String,
    name: String,
    size_kb: f64,
    category: String,
    date: String,
    title: String,
    summary: String,
}

#[tauri::command]
fn scan_dashboard() -> Result<Vec<DashboardAsset>, String> {
    let output_dir = base_output_dir();
    let mut assets: Vec<DashboardAsset> = Vec::new();

    if !output_dir.exists() {
        return Ok(assets);
    }

    fn walk_dir(dir: &PathBuf, base: &PathBuf, assets: &mut Vec<DashboardAsset>) {
        if assets.len() >= 200 { return; } // Safety limit
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if assets.len() >= 200 { break; }
                let path = entry.path();
                if path.is_dir() {
                    walk_dir(&path, base, assets);
                } else if path.extension().map_or(false, |ext| ext == "md" || ext == "json") {
                    let rel = path.strip_prefix(base).unwrap_or(&path);
                    let name = path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let meta = entry.metadata().ok();
                    let size = meta.as_ref().map(|m| m.len() as f64 / 1024.0).unwrap_or(0.0);
                    let date = meta.as_ref()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| {
                            let d: chrono::DateTime<chrono::Utc> = t.into();
                            Some(d.format("%Y-%m-%d").to_string())
                        }).unwrap_or_default();

                    // Category detection by filename keywords
                    let rel_str = rel.to_string_lossy().to_string();
                    let name_lower = name.to_lowercase();
                    let rel_lower = rel_str.to_lowercase();
                    let category = if rel_lower.contains("report") || name_lower.contains("research")
                        || name_lower.contains("报告") || name_lower.contains("研究") || name_lower.contains("调研")
                        || name_lower.contains("_report") { "report" }
                        else if rel_lower.contains("article") || name_lower.contains("推文") || name_lower.contains("科普")
                        || name_lower.contains("draft") || name_lower.contains("headline") || name_lower.contains("标题")
                        || name_lower.contains("outline") || name_lower.contains("大纲")
                        || name_lower.contains("草稿") || name_lower.contains("文案") || name_lower.contains("正文")
                        || name_lower.contains("图文") || name_lower.contains("engage")
                        { "article" }
                        else if rel_lower.contains("script") || name_lower.contains("脚本") || name_lower.contains("视频")
                        || name_lower.contains("voiceover") || name_lower.contains("口播") || name_lower.contains("scene")
                        || name_lower.contains("分镜") || name_lower.contains("旁白")
                        { "script" }
                        else if rel_lower.contains("image") || name_lower.contains("配图") || name_lower.contains("suggestion")
                        || name_lower.contains("图片") || name_lower.contains("视觉") || name_lower.contains("素材")
                        { "image" }
                        else { "other" };

                    // Try to extract title from file content
                    let (title, summary) = if path.extension().map_or(false, |ext| ext == "json") {
                        // JSON files: use filename as title, size as summary
                        let title = name.trim_end_matches(".json").to_string();
                        (title, format!("JSON 数据文件 ({:.1}KB)", size))
                    } else {
                        match fs::read_to_string(&path) {
                        Ok(content) => {
                            let first_line = content.lines().next().unwrap_or("").to_string();
                            let title = first_line.trim_start_matches("# ").trim().to_string();
                            let title = if title.is_empty() { name.trim_end_matches(".md").to_string() } else { title };
                            let body: String = content.lines()
                                .skip(1).take(8)
                                .filter(|l| !l.trim().is_empty())
                                .collect::<Vec<_>>().join(" ");
                            // Safe char-based truncation for UTF-8 Chinese text
                            let summary = if body.chars().count() > 120 {
                                format!("{}...", body.chars().take(120).collect::<String>())
                            } else { body };
                            (title, summary)
                        }
                        Err(_) => {
                            let ext_stripped = if name.ends_with(".json") { ".json" } else { ".md" };
                            (name.trim_end_matches(ext_stripped).to_string(), String::new())
                        }
                    } // end match
                    }; // end else (JSON vs MD branch)

                    assets.push(DashboardAsset {
                        path: rel_str,
                        name,
                        size_kb: format!("{:.1}", size).parse().unwrap_or(0.0),
                        category: category.to_string(),
                        date,
                        title,
                        summary,
                    });
                }
            }
        }
    }

    walk_dir(&output_dir, &output_dir, &mut assets);
    assets.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.title.cmp(&b.title)));
    Ok(assets)
}

#[tauri::command]
fn check_prerequisites(app: AppHandle) -> serde_json::Value {
    let claude_ok = check_claude_installed();
    let claude_version = get_claude_version();
    let claude_version_ok = claude_version.as_ref()
        .map(|v| version_at_least(v, MIN_CLAUDE_VERSION))
        .unwrap_or(false);
    let node_ok = Command::new("node").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
    let python_ok = Command::new("python3").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
    let skills_dir = skill_dir(&app);
    let skills_ok = skills_dir.exists() && skills_dir.read_dir().map(|mut d| d.next().is_some()).unwrap_or(false);

    serde_json::json!({
        "claude": claude_ok,
        "claude_version": claude_version,
        "claude_version_ok": claude_version_ok,
        "claude_min_version": MIN_CLAUDE_VERSION,
        "node": node_ok,
        "python3": python_ok,
        "skills": skills_ok,
        "skills_dir": skills_dir.to_string_lossy(),
        "ready": claude_ok && claude_version_ok && node_ok && python_ok && skills_ok,
    })
}

#[tauri::command]
fn get_history() -> Result<Vec<HistoryEntry>, String> {
    Ok(load_history().entries)
}

#[tauri::command]
fn get_history_detail(id: String) -> Result<HistoryEntry, String> {
    load_history().entries.iter().find(|e| e.id == id)
        .cloned().ok_or_else(|| "未找到该记录".to_string())
}

#[tauri::command]
fn delete_history(id: String) -> Result<(), String> {
    let mut store = load_history();

    // Find the entry to get its output_dir before removing
    let output_dir = store.entries.iter()
        .find(|e| e.id == id)
        .map(|e| e.output_dir.clone());

    store.entries.retain(|e| e.id != id);
    if let Ok(json) = serde_json::to_string_pretty(&store) {
        fs::write(history_file(), json).map_err(|e| e.to_string())?;
    }

    // Delete the associated output files
    if let Some(dir) = output_dir {
        if PathBuf::from(&dir).exists() {
            fs::remove_dir_all(&dir).ok();
        }
    }

    Ok(())
}

#[tauri::command]
fn check_path_exists(path: String) -> bool {
    PathBuf::from(&path).exists()
}

#[tauri::command]
fn get_base_output_dir() -> String {
    base_output_dir().to_string_lossy().to_string()
}

#[tauri::command]
fn open_output_folder(path: String) -> Result<(), String> {
    if !PathBuf::from(&path).exists() {
        return Err(format!("目录不存在: {}", path));
    }
    #[cfg(target_os = "macos")]
    { Command::new("open").arg(&path).spawn().map_err(|e| e.to_string())?; }
    #[cfg(target_os = "windows")]
    { Command::new("explorer").arg(&path).spawn().map_err(|e| e.to_string())?; }
    #[cfg(target_os = "linux")]
    { Command::new("xdg-open").arg(&path).spawn().map_err(|e| e.to_string())?; }
    Ok(())
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    { Command::new("open").arg(&url).spawn().map_err(|e| e.to_string())?; }
    #[cfg(target_os = "windows")]
    { Command::new("cmd").args(["/c", "start", &url]).spawn().map_err(|e| e.to_string())?; }
    #[cfg(target_os = "linux")]
    { Command::new("xdg-open").arg(&url).spawn().map_err(|e| e.to_string())?; }
    Ok(())
}

#[tauri::command]
async fn cancel_skill(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    // Kill the active Claude CLI subprocess
    if let Some(pid) = *state.active_pid.lock().unwrap() {
        let pid_str = pid.to_string();
        #[cfg(target_os = "windows")]
        { Command::new("taskkill").args(["/PID", &pid_str, "/F"]).spawn().ok(); }
        #[cfg(target_os = "macos")]
        { Command::new("kill").arg("-9").arg(&pid_str).spawn().ok(); }
        #[cfg(all(unix, not(target_os = "macos")))]
        { Command::new("kill").arg("-9").arg(&pid_str).spawn().ok(); }
    }
    *state.active_pid.lock().unwrap() = None;
    let mut running = state.running.lock().map_err(|e| e.to_string())?;
    *running = false;
    // Bump run_id so old thread can't re-lock running
    { let mut id = state.run_id.lock().unwrap(); *id = id.wrapping_add(1); }
    let _ = app.emit("skill-error", "⏹ 用户中止了任务");
    Ok("已取消".to_string())
}

fn dirs_next() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    { std::env::var("HOME").ok().map(PathBuf::from) }
    #[cfg(target_os = "windows")]
    { std::env::var("USERPROFILE").ok().map(PathBuf::from) }
    #[cfg(target_os = "linux")]
    { std::env::var("HOME").ok().map(PathBuf::from) }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    fs::create_dir_all(history_dir()).ok();
    tauri::Builder::default()
        .manage(AppState { running: Mutex::new(false), active_pid: Mutex::new(None), run_id: Mutex::new(0) })
        .setup(|app| {
            seed_default_skills(app.handle());
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info).build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_prerequisites,
            orchestrate, run_pipeline, revise_output, read_output_file, list_output_files,
            open_output_folder, open_url, cancel_skill, get_history, get_history_detail, delete_history,
            check_path_exists, scan_dashboard, get_base_output_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
