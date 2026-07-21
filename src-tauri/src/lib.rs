use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

fn base_output_dir() -> PathBuf {
    std::env::temp_dir().join("bio-om-output")
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExecutionStep {
    skill: String,
    display: String,
    prompt: String,
}

fn skill_dir(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap_or_else(|_| dirs_next().unwrap_or_default())
        .join("skills")
}

fn seed_default_skills(app: &AppHandle) {
    let dir = skill_dir(app);
    if dir.exists() { return; }
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
        format!("- {}: {} (触发词: {})", m.name, m.description, m.trigger_patterns.join(", "))
    }).collect::<Vec<_>>().join("\n");

    let prompt = format!(
        "你是一个内容运营工作流的编排器。用户输入了一段自然语言需求，你需要判断应该执行哪些 skill。\n\n\
        可用 skills:\n{}\n\n\
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

struct AppState { running: Mutex<bool> }

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
        ("video_interaction.md", "视频互动设计.md"),
        ("image_suggestions.json", "配图建议.json"),
        ("headline_options.md", "标题备选.md"),
        ("scene_prompts.md", "分镜提示词.md"),
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
                        prompt: format!("运行 {} skill，topic={}", dep.name, input),
                    });
                }

                // Add the main skill last (dependencies come first)
                let prompt = if top.name == "content-writing" {
                    format!("运行 content-writing skill，topic={}", input)
                } else if top.name == "web-research" {
                    format!("运行 web-research skill，搜索主题：{}", input)
                } else {
                    format!("运行 {} skill，输入：{}", top.name, input)
                };
                steps.push(ExecutionStep {
                    skill: top.name.clone(),
                    display: top.display.clone(),
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
    Ok(vec![ExecutionStep {
        skill: "content-writing".to_string(),
        display: "文案撰写与视频脚本".to_string(),
        prompt: format!("运行 content-writing skill，topic={}", input),
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
    let run_id = format!("run-{}", chrono::Local::now().format("%Y%m%d%H%M%S"));
    let steps_count = steps.len();
    let total_steps = steps_count as u32 * 100;
    let steps_for_thread = steps.clone();

    let _ = app_handle.emit("skill-progress", serde_json::json!({
        "step": 0, "total": total_steps, "name": "编排完成，开始执行..."
    }).to_string());

    std::thread::spawn(move || {
        let mut global_step;
        let mut all_success = true;

        for (i, step) in steps_for_thread.iter().enumerate() {
            let skill_base = i as u32 * 100;
            let _ = app_handle.emit("skill-output", &format!(
                "\n━━━ 第 {}/{} 步: {} ━━━", i + 1, steps_count, step.display
            ));
            let _ = app_handle.emit("skill-progress", serde_json::json!({
                "step": skill_base, "total": total_steps,
                "name": format!("正在执行: {}", step.display),
            }).to_string());

            let prompt = format!("{}，输出到 {}/", step.prompt, output_dir);

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

            if let Some(stdout) = child.stdout.take() {
                let mut line_count = 0u32;
                for line in BufReader::new(stdout).lines().flatten() {
                    line_count += 1;
                    // Advance progress: every ~20 lines of output = 1% within this skill
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

            if let Some(stderr) = child.stderr.take() {
                for line in BufReader::new(stderr).lines().flatten() {
                    if !line.is_empty() {
                        let _ = app_handle.emit("skill-output",
                            &format!("[stderr] {}", line));
                    }
                }
            }

            match child.wait() {
                Ok(status) if !status.success() => {
                    let _ = app_handle.emit("skill-error",
                        format!("{} 执行失败，退出码: {}",
                            step.display, status.code().unwrap_or(-1)));
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
            id: run_id, topic: input,
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
    });

    Ok(format!("编排完成，共 {} 步 → {}", steps.len(), output_dir_ret))
}

#[tauri::command]
async fn revise_output(
    app: AppHandle,
    state: State<'_, AppState>,
    output_dir: String,
    instruction: String,
) -> Result<String, String> {
    {
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

    std::thread::spawn(move || {
        let mut child = match Command::new("claude")
            .arg("-p").arg(&prompt).arg("--output-format").arg("text")
            .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()
        {
            Ok(c) => c,
            Err(e) => {
                let _ = app_handle.emit("skill-error", format!("启动修改失败: {}", e));
                return;
            }
        };

        if let Some(stdout) = child.stdout.take() {
            let mut line_count = 0u32;
            for line in BufReader::new(stdout).lines().flatten() {
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

        if let Some(stderr) = child.stderr.take() {
            for line in BufReader::new(stderr).lines().flatten() {
                if !line.is_empty() {
                    let _ = app_handle.emit("skill-output", &format!("[stderr] {}", line));
                }
            }
        }

        let success = child.wait().map(|s| s.success()).unwrap_or(false);

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
fn open_output_folder(path: String) -> Result<(), String> {
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
async fn cancel_skill(state: State<'_, AppState>) -> Result<String, String> {
    let mut running = state.running.lock().map_err(|e| e.to_string())?;
    *running = false;
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
        .manage(AppState { running: Mutex::new(false) })
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
