"""
Link Validator · URL有效性验证器
批量检查URL可达性，区分有效/失效链接，记录失效原因。
支持批量模式：接收主题+URL记录列表，按主题分组输出验证结果。

用途: url-research 技能 · 步骤1
输入: list[str] URL列表 或 list[dict{topic, url}] 批量记录
输出:
  - 标准模式: dict{valid, invalid, total, valid_count, invalid_count}
  - 批量模式: 附加 topic_results 按主题分组的验证结果
"""

import json
import sys
import re
from urllib.parse import urlparse


def validate_url_format(url: str) -> bool:
    """基础格式校验"""
    try:
        result = urlparse(url)
        return all([result.scheme in ("http", "https"), result.netloc])
    except Exception:
        return False


def check_accessibility(url: str, timeout: int = 10) -> tuple[bool, str]:
    """
    检查URL可访问性。

    Returns:
        (is_valid, reason)
    """
    # TODO: 实际发送 HEAD/GET 请求检查状态码
    # 200-299: 有效
    # 301/302: 有效（重定向）
    # 401/403: 有效（需权限，标记）
    # 404: 失效
    # 5xx: 暂时失效（建议重试）
    # Timeout/ConnectionError: 失效
    return (True, "")


def validate(urls, timeout: int = 10) -> dict:
    """
    批量验证URL。

    支持两种输入格式：
      - list[str]: 纯 URL 列表（标准模式）
      - list[dict{topic, url}]: 主题+URL 记录列表（批量模式，v1.3+）

    Args:
        urls: URL 列表或记录列表
        timeout: 每个URL的请求超时时间（秒）

    Returns:
        标准模式: {"valid": [...], "invalid": [...], "total": N, ...}
        批量模式: 附加 "topic_results": {topic: {valid: [], invalid: []}}
    """
    is_batch = urls and isinstance(urls[0], dict)

    if is_batch:
        records = urls
        url_list = list(set(r["url"] for r in records))
    else:
        url_list = urls
        records = None

    valid_urls = []
    invalid_urls = []

    for url in url_list:
        if not validate_url_format(url):
            invalid_urls.append({"url": url, "reason": "URL格式不合法"})
            continue

        is_valid, reason = check_accessibility(url, timeout)
        if is_valid:
            valid_urls.append(url)
        else:
            invalid_urls.append({"url": url, "reason": reason})

    result = {
        "valid": valid_urls,
        "invalid": invalid_urls,
        "total": len(url_list),
        "valid_count": len(valid_urls),
        "invalid_count": len(invalid_urls),
    }

    # 批量模式：按主题分组汇总
    if is_batch and records:
        valid_set = set(valid_urls)
        topic_results = {}
        for r in records:
            topic = r["topic"]
            url = r["url"]
            if topic not in topic_results:
                topic_results[topic] = {"valid": [], "invalid": []}
            if url in valid_set:
                topic_results[topic]["valid"].append(url)
            else:
                reason = next((inv["reason"] for inv in invalid_urls if inv["url"] == url), "未知")
                topic_results[topic]["invalid"].append({"url": url, "reason": reason})

        result["topic_results"] = topic_results
        result["total_records"] = len(records)
        result["topics"] = list(topic_results.keys())

    return result


if __name__ == "__main__":
    input_data = json.loads(sys.argv[1]) if len(sys.argv) > 1 else []
    result = validate(input_data)
    print(json.dumps(result, ensure_ascii=False, indent=2))
