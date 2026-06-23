# Prompt Library Viewer

一个独立本地网页应用，用来读取 GitHub Markdown 提示词库，缓存后解析成统一卡片。

## 第一版能力

- 数据源：
  - `EvoLinkAI/awesome-gpt-image-2-API-and-Prompts`
  - `wuyoscar/GPT-Image2-Skill`
- 后端：
  - 读取 GitHub Markdown。
  - 自动发现入口 Markdown 中链接的分类 Markdown。
  - 缓存到 `.cache/github-markdown`，默认 6 小时有效。
  - 解析为统一内部格式：标题、分类、提示词、图片、尺寸、质量、来源链接。
- 前端：
  - 分类筛选。
  - 来源筛选。
  - 搜索标题、分类和 prompt。
  - 卡片展示、预览图、详情弹窗。
  - 一键复制 prompt。

## 中文浏览层

应用会保留英文原始 prompt，同时追加中文浏览字段：

- `titleZh`：中文标题，识别不到时回退到原始标题。
- `categoryZh`：中文分类，来自本地分类词典。
- `summaryZh`：中文摘要，用本地关键词规则生成，只用于浏览、搜索和筛选。

中文字段不是完整翻译；复制按钮始终复制英文原始 prompt，避免影响图片生成效果。

## 运行

```powershell
node server.js
```

打开：

```text
http://127.0.0.1:5177
```

刷新远程 Markdown 缓存：

```powershell
node server.js --refresh-cache
```

指定端口：

```powershell
node server.js --port=5180
```

## 桌面版

构建便携版：

```powershell
cmd /c npm install
cmd /c npm run build:portable
```

运行：

```text
dist/win-unpacked/Prompt Library Viewer.exe
```

桌面版使用 Electron 打包，缓存会写入系统用户数据目录，不会写入应用安装目录。

## API

```text
GET /api/prompts
GET /api/prompts?refresh=1
GET /api/health
```

`/api/prompts` 返回：

```json
{
  "updatedAt": "2026-06-22T00:00:00.000Z",
  "itemCount": 0,
  "categories": [],
  "sources": [],
  "errors": [],
  "items": [
    {
      "id": "stable-id",
      "title": "标题",
      "titleZh": "中文标题",
      "category": "分类",
      "categoryZh": "中文分类",
      "prompt": "提示词正文",
      "summaryZh": "中文浏览摘要",
      "image": "第一张图片 URL",
      "images": [],
      "size": "1024x1024",
      "quality": "high",
      "sourceUrl": "GitHub Markdown 来源链接"
    }
  ]
}

```

后续可以在这个结构上加收藏、本地标签、离线缓存和导出 JSON/Markdown。
