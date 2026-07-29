import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import "./Dashboard.css";

interface DashboardAsset {
  path: string; name: string; size_kb: number;
  category: string; date: string;
  title: string; summary: string;
}

interface Props {
  onClose: () => void;
}

const CATEGORY_LABELS: Record<string, { label: string; icon: string }> = {
  report:  { label: "研究报告", icon: "📊" },
  article: { label: "推文草稿", icon: "📝" },
  script:  { label: "视频脚本", icon: "🎬" },
  image:   { label: "配图建议", icon: "🖼️" },
  other:   { label: "其他", icon: "📄" },
};

function Dashboard({ onClose }: Props) {
  const [assets, setAssets] = useState<DashboardAsset[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [activeCat, setActiveCat] = useState("all");
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [contentCache, setContentCache] = useState<Record<string, string>>({});
  const [readerAsset, setReaderAsset] = useState<DashboardAsset | null>(null);
  const [baseOutput, setBaseOutput] = useState("");

  const loadData = () => {
    setLoading(true);
    Promise.all([
      invoke<DashboardAsset[]>("scan_dashboard"),
      invoke<string>("get_base_output_dir"),
    ]).then(([data, dir]) => {
      setAssets(data);
      setBaseOutput(dir);
      setLoading(false);
    }).catch(() => setLoading(false));
  };

  useEffect(() => { loadData(); }, []);
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  const filtered = useMemo(() => {
    let list = assets;
    if (activeCat !== "all") list = list.filter((a) => a.category === activeCat);
    if (search.trim()) {
      const q = search.toLowerCase();
      list = list.filter((a) => a.title.toLowerCase().includes(q) || a.summary.toLowerCase().includes(q));
    }
    return list;
  }, [assets, activeCat, search]);

  const counts = useMemo(() => {
    const c: Record<string, number> = { all: assets.length };
    assets.forEach((a) => { c[a.category] = (c[a.category] || 0) + 1; });
    return c;
  }, [assets]);

  const loadContent = async (asset: DashboardAsset) => {
    if (contentCache[asset.path]) return;
    try {
      const content = await invoke<string>("read_output_file", { path: `${baseOutput}/${asset.path}` });
      setContentCache((prev) => ({ ...prev, [asset.path]: content }));
    } catch { /* ignore — file may be inaccessible */ }
  };

  const toggleExpand = async (asset: DashboardAsset) => {
    if (expandedId === asset.path) {
      setExpandedId(null);
      return;
    }
    setExpandedId(asset.path);
    await loadContent(asset);
  };

  const openReader = async (asset: DashboardAsset) => {
    setReaderAsset(asset);
    await loadContent(asset);
  };

  const catTabs = [
    { key: "all", label: "全部" },
    { key: "report", label: "研究报告" },
    { key: "article", label: "推文草稿" },
    { key: "script", label: "视频脚本" },
    { key: "image", label: "配图建议" },
  ];

  return (
    <div className="dashboard-overlay">
      <div className="dashboard">
        <header className="dash-header">
          <div>
            <h1>Bio-OM <span>Expert</span> · 资产仪表盘</h1>
            <p className="dash-sub">内容产出概览 · {assets.length} 份文档</p>
          </div>
          <div className="dash-header-actions">
            <button className="dash-btn-scan" onClick={loadData}>🔄 刷新</button>
            <button className="dash-btn-close" onClick={onClose}>✕</button>
          </div>
        </header>

        {loading ? (
          <div className="dash-loading">⏳ 扫描中...</div>
        ) : (
          <>
            <div className="dash-stats">
              {catTabs.map((tab) => (
                <div key={tab.key}
                  className={`dash-stat ${activeCat === tab.key ? "active" : ""}`}
                  onClick={() => setActiveCat(tab.key)}>
                  <div className="dash-stat-num">{counts[tab.key] || 0}</div>
                  <div className="dash-stat-label">{tab.label}</div>
                </div>
              ))}
            </div>

            <div className="dash-search">
              <input type="text" placeholder="搜索文档..." value={search}
                onChange={(e) => setSearch(e.target.value)} />
            </div>

            {filtered.length === 0 ? (
              <div className="dash-empty">暂无匹配文档</div>
            ) : (
              <div className="dash-grid">
                {filtered.map((asset) => {
                  const cat = CATEGORY_LABELS[asset.category] || CATEGORY_LABELS.other;
                  const isExp = expandedId === asset.path;
                  return (
                    <div key={asset.path} className={`dash-card ${isExp ? "expanded" : ""}`}
                      onDoubleClick={() => openReader(asset)}>
                      <div className="dash-card-header">
                        <span className="dash-card-type">{cat.icon} {cat.label}</span>
                        <span className="dash-card-meta">{asset.size_kb}KB · {asset.date}</span>
                      </div>
                      <h3 className="dash-card-title">{asset.title}</h3>
                      <p className="dash-card-summary">{asset.summary}</p>
                      <div className="dash-card-footer">
                        <span className="dash-card-path">{asset.path}</span>
                        <button className="dash-btn-expand"
                          onClick={() => toggleExpand(asset)}>
                          {isExp ? "收起 ▲" : "展开 ▼"}
                        </button>
                      </div>
                      {isExp && contentCache[asset.path] && (
                        <div className="dash-card-content">
                          <ReactMarkdown remarkPlugins={[remarkGfm]}>
                            {contentCache[asset.path]}
                          </ReactMarkdown>
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            )}
          </>
        )}
      </div>

      {/* Reader overlay */}
      {readerAsset && (
        <div className="dash-reader-overlay show" onClick={(e) => {
          if (e.target === e.currentTarget) setReaderAsset(null);
        }}>
          <div className="dash-reader">
            <button className="dash-reader-close" onClick={() => setReaderAsset(null)}>✕</button>
            <div className="dash-reader-meta">
              {CATEGORY_LABELS[readerAsset.category]?.icon} {readerAsset.title} · {readerAsset.size_kb}KB · {readerAsset.date}
            </div>
            <div className="dash-reader-body">
              {contentCache[readerAsset.path] ? (
                <ReactMarkdown remarkPlugins={[remarkGfm]}>
                  {contentCache[readerAsset.path]}
                </ReactMarkdown>
              ) : (
                <div className="dash-loading">⏳ 加载中...</div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default Dashboard;
