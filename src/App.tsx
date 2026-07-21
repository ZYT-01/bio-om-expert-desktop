import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import "./App.css";

interface LogEntry { type: "output" | "error" | "done" | "system" | "plan"; text: string; time: string; }

interface HistoryEntry {
  id: string; topic: string; output_dir: string;
  status: string; files: string[]; created_at: string;
}

interface ExecutionStep { skill: string; display: string; prompt: string; }

interface ProgressInfo { step: number; total: number; name: string; pct: number; }

function App() {
  const [input, setInput] = useState("");
  const [running, setRunning] = useState(false);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [outputDir, setOutputDir] = useState<string | null>(null);
  const [docxPath, setDocxPath] = useState<string | null>(null);
  const [done, setDone] = useState(false);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [progress, setProgress] = useState<ProgressInfo | null>(null);
  const [plan, setPlan] = useState<ExecutionStep[] | null>(null);
  const [previewFiles, setPreviewFiles] = useState<string[]>([]);
  const [previewFile, setPreviewFile] = useState<string | null>(null);
  const [previewContent, setPreviewContent] = useState<string>("");
  const [showSource, setShowSource] = useState(false);
  const logEndRef = useRef<HTMLDivElement>(null);

  const loadHistory = useCallback(async () => {
    try { setHistory(await invoke<HistoryEntry[]>("get_history")); } catch { /* */ }
  }, []);

  useEffect(() => { loadHistory(); }, [loadHistory]);
  useEffect(() => { logEndRef.current?.scrollIntoView({ behavior: "smooth" }); }, [logs]);

  const lastOutputRef = useRef("");
  const lastErrorRef = useRef("");
  const doneFiredRef = useRef(false);
  useEffect(() => {
    const uns: UnlistenFn[] = [];
    listen<string>("skill-output", (e) => {
      // Skip exact duplicates (caused by React StrictMode double-mounting listeners)
      if (e.payload === lastOutputRef.current) return;
      lastOutputRef.current = e.payload;
      setLogs((p) => [...p, { type: "output", text: e.payload, time: new Date().toLocaleTimeString() }]);
    }).then((f) => uns.push(f));
    listen<string>("skill-progress", (e) => {
      try {
        const d = JSON.parse(e.payload);
        setProgress({ step: d.step, total: d.total, name: d.name, pct: Math.round((d.step / d.total) * 100) });
      } catch { /* */ }
    }).then((f) => uns.push(f));
    listen<string>("skill-error", (e) => {
      if (e.payload === lastErrorRef.current) return;
      lastErrorRef.current = e.payload;
      setLogs((p) => [...p, { type: "error", text: e.payload, time: new Date().toLocaleTimeString() }]);
      setRunning(false); setProgress(null);
    }).then((f) => uns.push(f));
    listen<string>("skill-done", (e) => {
      if (doneFiredRef.current) return;
      doneFiredRef.current = true;
      let outDir = "";
      try {
        const d = JSON.parse(e.payload);
        outDir = d.output_dir || "";
        setOutputDir(d.output_dir || null);
        setDocxPath(d.docx_path || null);
      } catch { /* */ }
      setLogs((p) => [...p, { type: "done", text: "✅ 全部完成！", time: new Date().toLocaleTimeString() }]);
      setRunning(false); setDone(true); setProgress(null); setPlan(null); loadHistory();
      if (outDir) loadPreviewFiles(outDir);
    }).then((f) => uns.push(f));
    return () => { uns.forEach((f) => f()); };
  }, [loadHistory]);

  const handleRun = async () => {
    if (!input.trim() || running) return;
    doneFiredRef.current = false;
    setRunning(true); setDone(false); setOutputDir(null); setDocxPath(null); setPlan(null);
    setPreviewFiles([]); setPreviewFile(null); setPreviewContent("");
    setProgress({ step: 0, total: 100, name: "分析需求...", pct: 0 });
    setLogs([{ type: "system", text: `🔍 分析需求: ${input}`, time: new Date().toLocaleTimeString() }]);

    try {
      // Step 1: Orchestrate
      const steps = await invoke<ExecutionStep[]>("orchestrate", { input: input.trim() });
      setPlan(steps);

      const planText = steps.map((s, i) => `  ${i + 1}. ${s.display}`).join("\n");
      setLogs((p) => [
        ...p,
        { type: "plan", text: `📋 执行计划:\n${planText}`, time: new Date().toLocaleTimeString() },
        { type: "system", text: "开始执行 pipeline...", time: new Date().toLocaleTimeString() },
      ]);

      // Step 2: Run pipeline
      const msg = await invoke<string>("run_pipeline", { input: input.trim(), steps });
      setLogs((p) => [...p, { type: "system", text: msg, time: new Date().toLocaleTimeString() }]);
    } catch (err) {
      setLogs((p) => [...p, { type: "error", text: `错误: ${err}`, time: new Date().toLocaleTimeString() }]);
      setRunning(false); setProgress(null);
    }
  };

  const handleOpenFolder = async () => {
    if (!outputDir) return;
    try { await invoke("open_output_folder", { path: outputDir }); } catch { /* */ }
  };

  const loadPreviewFiles = useCallback(async (dir: string) => {
    try {
      const files = await invoke<string[]>("list_output_files", { dir });
      setPreviewFiles(files);
      if (files.length > 0 && !previewFile) {
        handleSelectPreviewFile(dir, files[0]);
      }
    } catch { /* */ }
  }, []);

  const handleSelectPreviewFile = async (dir: string, filename: string) => {
    setPreviewFile(filename);
    try {
      const content = await invoke<string>("read_output_file", { path: `${dir}/${filename}` });
      setPreviewContent(content);
    } catch { setPreviewContent("加载失败"); }
  };

  const handleSelectHistory = async (entry: HistoryEntry) => {
    setSelectedId(entry.id); setOutputDir(entry.output_dir);
    setDone(entry.status === "done"); setDocxPath(null); setPlan(null);
    setShowSource(false);
    if (entry.status === "done") loadPreviewFiles(entry.output_dir);
    try {
      const d = await invoke<HistoryEntry>("get_history_detail", { id: entry.id });
      setLogs([
        { type: "system", text: `📋 ${d.topic}`, time: d.created_at },
        { type: "system", text: `状态: ${d.status === "done" ? "✅ 完成" : "❌ 失败"}`, time: "" },
        { type: "system", text: `输出: ${d.output_dir}`, time: "" },
        ...(d.files.length > 0 ? [{ type: "system" as const, text: `文件:\n${d.files.join("\n")}`, time: "" }] : []),
      ]);
    } catch { /* */ }
  };

  const handleDeleteHistory = async (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    try {
      await invoke("delete_history", { id });
      setHistory((p) => p.filter((h) => h.id !== id));
      if (selectedId === id) {
        setSelectedId(null);
        setLogs([]);
        setOutputDir(null);
        setDocxPath(null);
        setDone(false);
        setProgress(null);
        setPlan(null);
        setPreviewFiles([]);
        setPreviewFile(null);
        setPreviewContent("");
      }
    } catch { /* */ }
  };

  const handleNewConversation = () => {
    lastOutputRef.current = "";
    lastErrorRef.current = "";
    doneFiredRef.current = false;
    setInput(""); setLogs([]); setOutputDir(null); setDocxPath(null);
    setDone(false); setProgress(null); setSelectedId(null); setPlan(null);
    setPreviewFiles([]); setPreviewFile(null); setPreviewContent("");
  };

  const handleRevise = async () => {
    if (!input.trim() || !outputDir || running) return;
    setRunning(true); setDone(false);
    setProgress({ step: 0, total: 11, name: "分析修改需求...", pct: 0 });
    setLogs((p) => [
      ...p,
      { type: "system", text: `🔧 修改: ${input}`, time: new Date().toLocaleTimeString() },
    ]);
    try {
      const msg = await invoke<string>("revise_output", {
        outputDir: outputDir,
        instruction: input.trim(),
      });
      setLogs((p) => [...p, { type: "system", text: msg, time: new Date().toLocaleTimeString() }]);
      setInput("");
    } catch (err) {
      setLogs((p) => [...p, { type: "error", text: `修改失败: ${err}`, time: new Date().toLocaleTimeString() }]);
      setRunning(false); setProgress(null);
    }
  };

  return (
    <div className="app">
      <header className="app-header">
        <button className="sidebar-toggle" onClick={() => setSidebarOpen(!sidebarOpen)}>
          {sidebarOpen ? "◁" : "▷"}
        </button>
        <h1>Bio-OM Expert</h1>
        <span className="subtitle">内容运营工作台</span>
        {plan && running && (
          <span className="plan-badge">{plan.map((s) => s.display).join(" → ")}</span>
        )}
        <span className={`status-badge ${running ? "running" : "idle"}`}>
          {running ? "⚡ 运行中..." : "✓ 就绪"}
        </span>
        <button
          className="feedback-link"
          onClick={() => invoke("open_url", { url: "https://github.com/ZYT-01/bio-om-expert-desktop/issues/new?template=bug_report.md" }).catch(() => {})}
          title="反馈问题或建议"
        >
          💬 反馈
        </button>
      </header>

      <div className="app-body">
        {sidebarOpen && (
          <aside className="sidebar">
            <div className="sidebar-header">
              <h3>📋 历史记录</h3>
              <span className="history-count">{history.length} 条</span>
              <button className="new-chat-btn" onClick={handleNewConversation} title="新建对话">＋</button>
            </div>
            <div className="history-list">
              {history.length === 0 && <p className="history-empty">暂无记录</p>}
              {history.map((entry) => (
                <div key={entry.id}
                  className={`history-item ${selectedId === entry.id ? "selected" : ""}`}
                  onClick={() => handleSelectHistory(entry)}>
                  <div className="history-item-header">
                    <span className={`history-status ${entry.status}`}>{entry.status === "done" ? "✅" : "❌"}</span>
                    <span className="history-topic">{entry.topic}</span>
                    <button className="history-delete" onClick={(e) => handleDeleteHistory(e, entry.id)} title="删除">✕</button>
                  </div>
                  <div className="history-meta">
                    <span>{entry.created_at}</span>
                    <span>{entry.files.length} 个文件</span>
                  </div>
                </div>
              ))}
            </div>
          </aside>
        )}

        <main className="main">
          {progress && (
            <div className="progress-bar-container">
              <div className="progress-bar">
                <div className="progress-fill" style={{ width: `${progress.pct}%` }} />
              </div>
              <span className="progress-text">{progress.name} ({progress.step}/{progress.total})</span>
            </div>
          )}

          <div className="log-panel">
            {logs.length === 0 && (
              <div className="empty-state">
                <p>输入你的需求，AI 自动编排 skill 执行。</p>
                <p className="hint">例如：写一篇关于 SOD 抗氧化机制的科普推文</p>
                <p className="hint">例如：搜索 AAV 基因治疗的最新研究进展</p>
              </div>
            )}
            {logs.map((entry, i) => (
              <div key={i} className={`log-entry log-${entry.type}`}>
                <span className="log-time">{entry.time ? `[${entry.time}]` : ""}</span>
                <span className="log-text" style={{ whiteSpace: "pre-wrap" }}>{entry.text}</span>
              </div>
            ))}
            <div ref={logEndRef} />
          </div>

          <div className="input-panel">
            <input type="text" value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) {
                  e.preventDefault();
                  done ? handleRevise() : handleRun();
                }
              }}
              placeholder={done ? "输入修改意见，如：太技术了，改通俗一点" : "描述你的需求，回车发送..."}
              disabled={running} className="topic-input" />
            {done && outputDir && (
              <>
                <button onClick={handleOpenFolder} className="folder-button">📂 全部文件</button>
                {docxPath && (
                  <button onClick={handleOpenFolder} className="docx-button">📄 Word 文档</button>
                )}
                <button onClick={handleNewConversation} className="new-task-button">＋ 新任务</button>
              </>
            )}
            {!done && (
              <>
                <button onClick={handleRun} disabled={running || !input.trim()} className="run-button">
                  {running ? "⏳ 执行中..." : "▶ 运行"}
                </button>
                {running && (
                  <button onClick={async () => {
                    await invoke("cancel_skill").catch(() => {});
                    setRunning(false);
                    setProgress(null);
                    setLogs((p) => [...p, { type: "system", text: "⏹ 任务已中止", time: new Date().toLocaleTimeString() }]);
                  }} className="cancel-button">
                    ⏹ 中止
                  </button>
                )}
              </>
            )}
            {done && (
              <button onClick={handleRevise} disabled={running || !input.trim()} className="revise-button">
                {running ? "⏳ 修改中..." : "🔧 修改"}
              </button>
            )}
          </div>
        </main>

        {done && previewFiles.length > 0 && (
          <aside className="preview-panel">
            <div className="preview-header">
              <h3>📄 文件预览</h3>
              <button className="preview-close" onClick={() => { setPreviewFiles([]); setPreviewFile(null); }}>
                ✕
              </button>
            </div>
            <div className="preview-tabs">
              {previewFiles.map((f) => (
                <button
                  key={f}
                  className={`preview-tab ${previewFile === f ? "active" : ""}`}
                  onClick={() => outputDir && handleSelectPreviewFile(outputDir, f)}
                >
                  {f.replace(".md", "")}
                </button>
              ))}
            </div>
            <div className="preview-toolbar">
              <button
                className={`preview-toggle ${!showSource ? "active" : ""}`}
                onClick={() => setShowSource(false)}
              >
                预览
              </button>
              <button
                className={`preview-toggle ${showSource ? "active" : ""}`}
                onClick={() => setShowSource(true)}
              >
                源码
              </button>
            </div>
            <div className="preview-content">
              {showSource ? (
                <pre className="preview-source">{previewContent}</pre>
              ) : (
                <div className="markdown-body">
                  <ReactMarkdown remarkPlugins={[remarkGfm]}>
                    {previewContent}
                  </ReactMarkdown>
                </div>
              )}
            </div>
          </aside>
        )}
      </div>
    </div>
  );
}

export default App;
