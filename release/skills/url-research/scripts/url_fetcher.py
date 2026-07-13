"""
URL Fetcher · 网页内容抓取器
逐个抓取URL的完整HTML内容，支持动态页面。

用途: url-research 技能 · 步骤2
输入: list[str] 有效URL列表
输出: list[dict{url, html, status_code, fetch_time, content_type}]
"""

import json
import sys
import time
import hashlib

# 尝试加载 user_agents 配置
try:
    with open("../assets/user_agents.json", "r", encoding="utf-8") as f:
        UA_CONFIG = json.load(f)
except (FileNotFoundError, json.JSONDecodeError):
    UA_CONFIG = {
        "default": "Bio-OM-Expert/1.0 (URL Research; compliance@bio-om-expert.com)"
    }


def fetch_static(url: str, headers: dict, timeout: int = 30) -> dict:
    """抓取静态页面"""
    # TODO: 实际使用 requests/httpx 发送 GET 请求
    return {
        "url": url,
        "html": "",
        "status_code": 200,
        "fetch_time": "",
        "content_type": "text/html",
    }


def fetch_dynamic(url: str, timeout: int = 30) -> dict:
    """使用 Selenium/Playwright 抓取动态渲染页面"""
    # TODO: 实际启动 headless browser 抓取 JS 渲染后的 DOM
    return {
        "url": url,
        "html": "",
        "status_code": 200,
        "fetch_time": "",
        "content_type": "text/html",
        "rendered": True,
    }


def needs_dynamic_rendering(url: str) -> bool:
    """
    判断页面是否可能需要动态渲染。
    规则: SPA框架常见特征、已知需JS渲染的平台等。
    """
    dynamic_patterns = [
        "zhihu.com", "weixin.qq.com", "mp.weixin.qq.com",
        "toutiao.com", "xiaohongshu.com",
    ]
    return any(p in url for p in dynamic_patterns)


def fetch(urls: list[str], use_dynamic: bool = True) -> list[dict]:
    """
    批量抓取URL内容。

    Args:
        urls: 有效URL列表
        use_dynamic: 是否对JS渲染页面启用动态抓取

    Returns:
        抓取结果列表
    """
    results = []
    headers = {"User-Agent": UA_CONFIG.get("default", "")}

    for url in urls:
        start = time.time()

        if use_dynamic and needs_dynamic_rendering(url):
            result = fetch_dynamic(url)
        else:
            result = fetch_static(url, headers)

        result["fetch_time"] = f"{time.time() - start:.2f}s"
        result["content_hash"] = hashlib.md5(
            result.get("html", "").encode()
        ).hexdigest()

        results.append(result)

    return results


if __name__ == "__main__":
    input_urls = json.loads(sys.argv[1]) if len(sys.argv) > 1 else []
    result = fetch(input_urls)
    print(json.dumps(result, ensure_ascii=False, indent=2))
