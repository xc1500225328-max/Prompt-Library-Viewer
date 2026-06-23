const crypto = require("node:crypto");
const fs = require("node:fs/promises");
const http = require("node:http");
const path = require("node:path");

const ROOT_DIR = __dirname;
const PUBLIC_DIR = path.join(ROOT_DIR, "public");
const CACHE_DIR = process.env.PROMPT_LIBRARY_CACHE_DIR
  ? path.resolve(process.env.PROMPT_LIBRARY_CACHE_DIR)
  : path.join(ROOT_DIR, ".cache", "github-markdown");
const CACHE_TTL_MS = Number(process.env.CACHE_TTL_MS || 6 * 60 * 60 * 1000);
const DEFAULT_PORT = Number(process.env.PORT || 5177);

const SOURCES = [
  {
    id: "evolinkai-awesome-gpt-image-2",
    name: "EvoLinkAI Prompt Cases",
    repo: "EvoLinkAI/awesome-gpt-image-2-API-and-Prompts",
    branch: "main",
    entryPaths: ["README.md"],
    includeLinkedPath: (repoPath) => /^cases\/[^?#]+\.md$/i.test(repoPath),
  },
  {
    id: "gpt-image2-skill-gallery",
    name: "GPT-Image2 Skill Gallery",
    repo: "wuyoscar/GPT-Image2-Skill",
    branch: "main",
    entryPaths: ["skills/gpt-image/references/gallery.md"],
    includeLinkedPath: (repoPath) =>
      /^skills\/gpt-image\/references\/gallery-[^?#]+\.md$/i.test(repoPath),
  },
];

const MIME_TYPES = new Map([
  [".html", "text/html; charset=utf-8"],
  [".css", "text/css; charset=utf-8"],
  [".js", "text/javascript; charset=utf-8"],
  [".json", "application/json; charset=utf-8"],
  [".svg", "image/svg+xml"],
  [".png", "image/png"],
  [".jpg", "image/jpeg"],
  [".jpeg", "image/jpeg"],
  [".webp", "image/webp"],
  [".ico", "image/x-icon"],
]);

let memoryCache = null;

function rawUrl(source, repoPath) {
  return `https://raw.githubusercontent.com/${source.repo}/${source.branch}/${repoPath}`;
}

function sourceBlobUrl(source, repoPath, lineNumber) {
  const anchor = lineNumber ? `#L${lineNumber}` : "";
  return `https://github.com/${source.repo}/blob/${source.branch}/${repoPath}${anchor}`;
}

function nowIso() {
  return new Date().toISOString();
}

function slugify(value) {
  return String(value || "")
    .normalize("NFKD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/[^a-zA-Z0-9\u4e00-\u9fff]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 90)
    .toLowerCase();
}

function hash(value) {
  return crypto.createHash("sha1").update(value).digest("hex");
}

function stableId(parts) {
  return hash(parts.filter(Boolean).join("\n")).slice(0, 16);
}

function normalizeLineEndings(markdown) {
  return markdown.replace(/\r\n?/g, "\n");
}

function cleanMarkdownText(value) {
  return String(value || "")
    .replace(/!\[([^\]]*)\]\([^)]+\)/g, "$1")
    .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
    .replace(/<[^>]+>/g, " ")
    .replace(/[`*_~]/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

function cleanTitle(rawTitle) {
  const linkedTitle = String(rawTitle || "").match(/\[([^\]]+)\]\([^)]+\)/);
  const base = linkedTitle ? linkedTitle[1] : rawTitle;
  return cleanMarkdownText(base)
    .replace(/^case\s*\d+\s*(?:[:\uFF1A.\-\u00B7]\s*)?/i, "")
    .replace(/^no\.?\s*\d+\s*(?:[.\-\u00B7]\s*|[:\uFF1A]\s+)?/i, "")
    .replace(/^\d+\s*(?:[.\-\u00B7]\s+|[:\uFF1A]\s+)/i, "")
    .replace(/\s*\((?:by|via)\s+[^)]+\)\s*$/i, "")
    .trim();
}

function categoryFromPath(repoPath) {
  const file = path.posix.basename(repoPath, ".md");
  return file
    .replace(/^gallery-/i, "")
    .replace(/^cases?-/i, "")
    .split(/[-_]+/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function normalizeCategory(rawCategory, repoPath) {
  const cleaned = cleanMarkdownText(rawCategory)
    .replace(/range\s*:.*$/i, "")
    .replace(/\bcases?\b/gi, "")
    .replace(/\bprompts?\b/gi, "")
    .replace(/\bgallery\b/gi, "")
    .replace(/\s+/g, " ")
    .trim();
  return cleaned || categoryFromPath(repoPath) || "Uncategorized";
}

const CATEGORY_ZH_EXACT = new Map([
  ["uncategorized", "未分类"],
  ["ad creative", "广告创意"],
  ["architecture and interior", "建筑与室内"],
  ["anime and manga", "动漫漫画"],
  ["beauty and lifestyle", "美妆生活"],
  ["brand systems and identity", "品牌识别"],
  ["character", "角色"],
  ["character design", "角色设计"],
  ["cinematic and animation", "电影与动画"],
  ["cinematic film references", "电影镜头参考"],
  ["comparison", "对比测试"],
  ["comparison and community examples", "对比与社区示例"],
  ["data visualization", "数据可视化"],
  ["e commerce", "电商产品"],
  ["ecommerce", "电商产品"],
  ["edit endpoint showcase", "编辑接口示例"],
  ["events and experience", "活动体验"],
  ["fashion editorial", "时尚大片"],
  ["fine art painting", "艺术绘画"],
  ["gaming", "游戏"],
  ["illustration", "插画"],
  ["infographics and field guides", "信息图与指南"],
  ["ink and chinese", "水墨与中文风格"],
  ["isometric", "等距视角"],
  ["liquid realism rules", "液态写实规则"],
  ["more illustration styles", "更多插画风格"],
  ["official openai cookbook examples", "OpenAI 官方示例"],
  ["photography", "摄影"],
  ["pixel art", "像素艺术"],
  ["portrait and photography", "人像摄影"],
  ["portrait", "人像"],
  ["poster and illustration", "海报插画"],
  ["poster", "海报"],
  ["product and food", "产品与美食"],
  ["research paper figures", "论文图表"],
  ["scientific and educational", "科学教育"],
  ["screen photography", "屏幕摄影"],
  ["tattoo design", "纹身设计"],
  ["technical illustration", "技术插画"],
  ["typography and posters", "字体与海报"],
  ["ui", "界面设计"],
  ["ui and social media mockup", "UI 与社媒样机"],
  ["ui ux mockups", "UI/UX 样机"],
  ["watercolor", "水彩"],
]);

const TITLE_ZH_EXACT = new Map([
  ["ad creative", "广告创意"],
  ["architecture and interior", "建筑与室内设计"],
  ["anime and manga", "动漫漫画风格"],
  ["beauty and lifestyle", "美妆生活方式"],
  ["brand systems and identity", "品牌系统与识别"],
  ["character design", "角色设计"],
  ["cinematic and animation", "电影与动画风格"],
  ["data visualization", "数据可视化"],
  ["ecommerce", "电商产品图"],
  ["fashion editorial", "时尚大片"],
  ["fine art painting", "艺术绘画"],
  ["infographics and field guides", "信息图与图解指南"],
  ["isometric", "等距视角插画"],
  ["official openai cookbook examples", "OpenAI 官方示例"],
  ["product and food", "产品与美食摄影"],
  ["research paper figures", "论文图表"],
  ["scientific and educational", "科学教育图"],
  ["technical illustration", "技术插画"],
  ["typography and posters", "字体与海报设计"],
  ["ui ux mockups", "UI/UX 样机"],
]);

const TITLE_KEYWORDS_ZH = [
  { pattern: /\b(character|avatar|mascot|hero|creature)\b/i, label: "角色" },
  { pattern: /\b(portrait|headshot|selfie)\b/i, label: "人像" },
  { pattern: /\b(product|packaging|ecommerce|e-commerce)\b/i, label: "产品" },
  { pattern: /\b(food|drink|beverage|restaurant)\b/i, label: "美食" },
  { pattern: /\b(poster|flyer|cover)\b/i, label: "海报" },
  { pattern: /\b(logo|brand|identity|visual system)\b/i, label: "品牌" },
  { pattern: /\b(ui|ux|interface|dashboard|app|mockup)\b/i, label: "界面" },
  { pattern: /\b(icon|sticker)\b/i, label: "图标" },
  { pattern: /\b(illustration|drawing|sketch)\b/i, label: "插画" },
  { pattern: /\b(photo|photography|camera)\b/i, label: "摄影" },
  { pattern: /\b(cinematic|film|movie|animation)\b/i, label: "电影感" },
  { pattern: /\b(anime|manga)\b/i, label: "动漫" },
  { pattern: /\b(watercolor|ink|painting|fine art)\b/i, label: "绘画" },
  { pattern: /\b(infographic|diagram|chart|data visualization|field guide)\b/i, label: "图解" },
  { pattern: /\b(architecture|interior|room|building)\b/i, label: "空间" },
  { pattern: /\b(fashion|editorial|beauty|lifestyle)\b/i, label: "时尚生活" },
  { pattern: /\b(pixel|game|gaming)\b/i, label: "游戏像素" },
  { pattern: /\b(tattoo)\b/i, label: "纹身" },
];

const SUMMARY_USE_KEYWORDS_ZH = [
  { pattern: /\b(character|avatar|mascot|hero|creature)\b/i, label: "角色设计" },
  { pattern: /\b(portrait|headshot|selfie|face)\b/i, label: "人像" },
  { pattern: /\b(product|packaging|ecommerce|e-commerce|sku)\b/i, label: "产品展示" },
  { pattern: /\b(food|drink|beverage|dessert|restaurant)\b/i, label: "美食饮品" },
  { pattern: /\b(poster|flyer|book cover|album cover|cover art)\b/i, label: "海报封面" },
  { pattern: /\b(logo|brand|identity|visual system|guidelines)\b/i, label: "品牌识别" },
  { pattern: /\b(ui|ux|interface|dashboard|mobile app|web app|mockup)\b/i, label: "界面样机" },
  { pattern: /\b(icon|sticker|emoji)\b/i, label: "图标贴纸" },
  { pattern: /\b(infographic|diagram|chart|map|field guide|data visualization)\b/i, label: "信息图解" },
  { pattern: /\b(architecture|interior|room|building|furniture)\b/i, label: "建筑室内" },
  { pattern: /\b(fashion|editorial|beauty|makeup|lifestyle)\b/i, label: "时尚生活" },
  { pattern: /\b(game|gaming|pixel art|sprite|isometric)\b/i, label: "游戏视觉" },
  { pattern: /\b(scientific|educational|research paper|technical)\b/i, label: "科教技术" },
  { pattern: /\b(event|exhibition|experience|installation)\b/i, label: "活动体验" },
  { pattern: /\b(tattoo)\b/i, label: "纹身图案" },
];

const SUMMARY_STYLE_KEYWORDS_ZH = [
  { pattern: /\b(photorealistic|realistic|photo-realistic)\b/i, label: "写实" },
  { pattern: /\b(cinematic|film still|movie|anamorphic)\b/i, label: "电影感" },
  { pattern: /\b(anime|manga|cel shaded)\b/i, label: "动漫" },
  { pattern: /\b(watercolor)\b/i, label: "水彩" },
  { pattern: /\b(ink|chinese ink|sumi-e)\b/i, label: "水墨" },
  { pattern: /\b(oil painting|fine art|renaissance|baroque)\b/i, label: "艺术绘画" },
  { pattern: /\b(vector|flat design|minimal)\b/i, label: "简洁矢量" },
  { pattern: /\b(isometric)\b/i, label: "等距视角" },
  { pattern: /\b(retro|vintage|nostalgic)\b/i, label: "复古" },
  { pattern: /\b(cyberpunk|neon|futuristic)\b/i, label: "赛博未来" },
  { pattern: /\b(pixel art|8-bit|16-bit)\b/i, label: "像素" },
  { pattern: /\b(3d|render|octane|blender)\b/i, label: "3D 渲染" },
  { pattern: /\b(editorial|magazine)\b/i, label: "杂志感" },
  { pattern: /\b(technical|blueprint|schematic)\b/i, label: "技术图纸" },
];

function normalizeLookupKey(value) {
  return cleanMarkdownText(value)
    .replace(/&/g, " and ")
    .replace(/[^\p{L}\p{N}]+/gu, " ")
    .replace(/\s+/g, " ")
    .trim()
    .toLowerCase();
}

function collectChineseLabels(text, rules, limit = 6) {
  const labels = [];
  const seen = new Set();
  for (const rule of rules) {
    if (rule.pattern.test(text) && !seen.has(rule.label)) {
      seen.add(rule.label);
      labels.push(rule.label);
      if (labels.length >= limit) {
        break;
      }
    }
  }
  return labels;
}

function translateCategoryZh(category) {
  const key = normalizeLookupKey(category);
  if (CATEGORY_ZH_EXACT.has(key)) {
    return CATEGORY_ZH_EXACT.get(key);
  }

  const labels = collectChineseLabels(category, [...SUMMARY_USE_KEYWORDS_ZH, ...SUMMARY_STYLE_KEYWORDS_ZH], 3);
  return labels.length ? labels.join(" / ") : cleanMarkdownText(category);
}

function translateTitleZh(title, categoryZh) {
  const cleaned = cleanTitle(title);
  const key = normalizeLookupKey(cleaned);
  if (TITLE_ZH_EXACT.has(key)) {
    return TITLE_ZH_EXACT.get(key);
  }

  if (!cleaned || /^prompt\s*[a-z0-9]?$/i.test(cleaned)) {
    return `${categoryZh}提示词`;
  }

  const labels = collectChineseLabels(cleaned, TITLE_KEYWORDS_ZH, 4);
  if (labels.length >= 2) {
    return labels.join(" / ");
  }
  if (labels.length === 1 && cleaned.split(/\s+/).length <= 5) {
    return labels[0];
  }
  return cleaned;
}

function translateQualityZh(quality) {
  const key = normalizeLookupKey(quality);
  const qualityMap = new Map([
    ["low", "低"],
    ["medium", "中"],
    ["high", "高"],
    ["auto", "自动"],
    ["standard", "标准"],
    ["hd", "高清"],
  ]);
  return qualityMap.get(key) || cleanMarkdownText(quality);
}

function buildSummaryZh(item) {
  const text = [item.title, item.category, item.prompt].filter(Boolean).join("\n");
  const useLabels = collectChineseLabels(text, SUMMARY_USE_KEYWORDS_ZH, 5);
  const styleLabels = collectChineseLabels(text, SUMMARY_STYLE_KEYWORDS_ZH, 5);
  const specs = [];

  if (item.size) {
    specs.push(item.size);
  }
  if (item.quality) {
    specs.push(`质量${translateQualityZh(item.quality)}`);
  }

  const parts = [];
  if (useLabels.length) {
    parts.push(`要点：${useLabels.join("、")}`);
  }
  if (styleLabels.length) {
    parts.push(`风格：${styleLabels.join("、")}`);
  }
  if (specs.length) {
    parts.push(`规格：${specs.join(" / ")}`);
  }

  if (parts.length === 0) {
    return `用于${item.categoryZh}的图片生成提示词，复制时保留英文原文。`;
  }
  return `${parts.join("；")}。`;
}

function addChineseDisplayFields(item) {
  const categoryZh = translateCategoryZh(item.category);
  const localizedItem = {
    ...item,
    titleZh: translateTitleZh(item.title, categoryZh),
    categoryZh,
  };
  return {
    ...localizedItem,
    summaryZh: buildSummaryZh(localizedItem),
  };
}

function cachePaths(source, repoPath) {
  const key = hash(`${source.repo}@${source.branch}:${repoPath}`);
  const dir = path.join(CACHE_DIR, source.id);
  return {
    dir,
    body: path.join(dir, `${key}.md`),
    meta: path.join(dir, `${key}.json`),
  };
}

async function fileExists(filePath) {
  try {
    await fs.access(filePath);
    return true;
  } catch {
    return false;
  }
}

async function readJson(filePath) {
  try {
    return JSON.parse(await fs.readFile(filePath, "utf8"));
  } catch {
    return null;
  }
}

async function readMarkdownWithCache(source, repoPath, forceRefresh) {
  const paths = cachePaths(source, repoPath);
  const meta = await readJson(paths.meta);
  const hasBody = await fileExists(paths.body);
  const ageMs = meta?.fetchedAt ? Date.now() - Date.parse(meta.fetchedAt) : Infinity;

  if (!forceRefresh && hasBody && ageMs >= 0 && ageMs < CACHE_TTL_MS) {
    return {
      markdown: await fs.readFile(paths.body, "utf8"),
      cache: "fresh",
      fetchedAt: meta.fetchedAt,
      url: meta.url || rawUrl(source, repoPath),
    };
  }

  const url = rawUrl(source, repoPath);
  try {
    const response = await fetch(url, {
      headers: {
        "User-Agent": "Prompt-Library-Viewer/0.1",
        Accept: "text/markdown,text/plain,*/*",
      },
    });

    if (!response.ok) {
      throw new Error(`GitHub 返回 HTTP ${response.status}`);
    }

    const markdown = await response.text();
    const fetchedAt = nowIso();
    await fs.mkdir(paths.dir, { recursive: true });
    await fs.writeFile(paths.body, markdown, "utf8");
    await fs.writeFile(
      paths.meta,
      JSON.stringify({ repoPath, url, fetchedAt, sourceId: source.id }, null, 2),
      "utf8"
    );
    return { markdown, cache: "network", fetchedAt, url };
  } catch (error) {
    if (hasBody) {
      return {
        markdown: await fs.readFile(paths.body, "utf8"),
        cache: "stale",
        fetchedAt: meta?.fetchedAt || null,
        url,
        warning: error.message,
      };
    }
    throw error;
  }
}

function parseMarkdownLinks(markdown) {
  const links = [];
  const linkPattern = /(?<!!)\[[^\]]+\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g;
  let match;
  while ((match = linkPattern.exec(markdown))) {
    links.push(match[1]);
  }
  return links;
}

function stripQueryAndHash(href) {
  return String(href || "").split("#")[0].split("?")[0].trim();
}

function githubPathFromUrl(href, source) {
  try {
    const url = new URL(href);
    const repoParts = source.repo.split("/");
    const host = url.hostname.toLowerCase();
    const pathParts = url.pathname.split("/").filter(Boolean);

    if (
      host === "github.com" &&
      pathParts[0]?.toLowerCase() === repoParts[0].toLowerCase() &&
      pathParts[1]?.toLowerCase() === repoParts[1].toLowerCase() &&
      pathParts[2] === "blob" &&
      pathParts[3] === source.branch
    ) {
      return pathParts.slice(4).join("/");
    }

    if (
      host === "raw.githubusercontent.com" &&
      pathParts[0]?.toLowerCase() === repoParts[0].toLowerCase() &&
      pathParts[1]?.toLowerCase() === repoParts[1].toLowerCase() &&
      pathParts[2] === source.branch
    ) {
      return pathParts.slice(3).join("/");
    }
  } catch {
    return null;
  }
  return null;
}

function resolveRepoPath(source, currentPath, href) {
  const cleanHref = stripQueryAndHash(href);
  if (!cleanHref || cleanHref.startsWith("#")) {
    return null;
  }

  const absoluteRepoPath = githubPathFromUrl(cleanHref, source);
  if (absoluteRepoPath) {
    return absoluteRepoPath;
  }

  if (/^[a-z][a-z0-9+.-]*:/i.test(cleanHref)) {
    return null;
  }

  const baseDir = path.posix.dirname(currentPath);
  return path.posix.normalize(path.posix.join(baseDir, cleanHref)).replace(/^\.\//, "");
}

function discoverLinkedMarkdownPaths(source, currentPath, markdown) {
  const discovered = new Set();
  for (const href of parseMarkdownLinks(markdown)) {
    const repoPath = resolveRepoPath(source, currentPath, href);
    if (repoPath && source.includeLinkedPath(repoPath)) {
      discovered.add(repoPath);
    }
  }
  return [...discovered];
}

function resolveAssetUrl(source, repoPath, href) {
  const cleanHref = stripQueryAndHash(href);
  if (!cleanHref) {
    return null;
  }
  try {
    const url = new URL(cleanHref);
    if (url.hostname === "github.com" && url.pathname.includes("/blob/")) {
      return cleanHref
        .replace("https://github.com/", "https://raw.githubusercontent.com/")
        .replace(`/blob/${source.branch}/`, `/${source.branch}/`);
    }
    return cleanHref;
  } catch {
    const resolved = resolveRepoPath(source, repoPath, cleanHref);
    return resolved ? rawUrl(source, resolved) : null;
  }
}

function extractImages(source, repoPath, content) {
  const images = [];
  const seen = new Set();
  const addImage = (href, alt = "") => {
    const url = resolveAssetUrl(source, repoPath, href);
    if (!url || seen.has(url)) {
      return;
    }
    seen.add(url);
    images.push({ url, alt: cleanMarkdownText(alt) });
  };

  let match;
  const markdownImagePattern = /!\[([^\]]*)\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g;
  while ((match = markdownImagePattern.exec(content))) {
    addImage(match[2], match[1]);
  }

  const htmlImagePattern = /<img\b[^>]*\bsrc=["']([^"']+)["'][^>]*>/gi;
  while ((match = htmlImagePattern.exec(content))) {
    const alt = match[0].match(/\balt=["']([^"']*)["']/i)?.[1] || "";
    addImage(match[1], alt);
  }

  const imageFieldPattern = /^\s*(?:[-*]\s*)?(?:image|output|preview)\s*:\s*`?([^`\s]+)`?\s*$/gim;
  while ((match = imageFieldPattern.exec(content))) {
    addImage(match[1], path.posix.basename(match[1]));
  }

  return images;
}

function findPromptLabel(beforeFence) {
  const lines = beforeFence.split("\n").slice(-6).reverse();
  for (const line of lines) {
    const cleaned = cleanMarkdownText(line);
    const match = cleaned.match(/\b(prompt\s*[A-Z0-9]?)\b(?:\s*[—:-]\s*(.+))?/i);
    if (match) {
      const suffix = match[2] ? ` - ${match[2]}` : "";
      return `${match[1].replace(/\s+/g, " ")}${suffix}`.trim();
    }
  }
  return "";
}

function stripOuterPromptSyntax(prompt) {
  return String(prompt || "")
    .trim()
    .replace(/^```[a-zA-Z0-9_-]*\s*/, "")
    .replace(/^`+/, "")
    .replace(/```$/g, "")
    .replace(/`+$/, "")
    .trim();
}

function isPromptLabelLine(line) {
  return /^\s*(?:[-*]\s*)?(?:\*\*)?\s*prompt(?:\s+[A-Z0-9])?\b\s*(?:[:\uFF1A])?\s*(?:\*\*)?\s*(?:[:\uFF1A])?\s*$/i.test(
    line
  );
}

function stripPromptLabelPrefix(line) {
  return line.replace(
    /^\s*(?:[-*]\s*)?(?:\*\*)?\s*prompt(?:\s+[A-Z0-9])?\b\s*(?:[:\uFF1A])?\s*(?:\*\*)?\s*(?:[:\uFF1A])?\s*/i,
    ""
  );
}

function promptNoiseScore(prompt) {
  const text = String(prompt || "");
  const patterns = [
    /<img\b/i,
    /<a\s/i,
    /target=["']_blank["']/i,
    /utm_source=github/i,
    /githubusercontent\.com\/[^)\s]+\/images\//i,
    /\*\*Prompt\b/i,
    /^```/m,
    /```$/m,
  ];
  return patterns.reduce((score, pattern) => score + (pattern.test(text) ? 1 : 0), 0);
}

function cleanPromptText(prompt) {
  const lines = normalizeLineEndings(stripOuterPromptSyntax(prompt)).split("\n");
  const cleanedLines = [];

  for (const originalLine of lines) {
    let line = originalLine;
    const trimmed = line.trim();

    if (!trimmed || /^```[a-zA-Z0-9_-]*\s*$/.test(trimmed)) {
      continue;
    }
    if (isPromptLabelLine(line)) {
      continue;
    }
    if (/^\s*\|?\s*:?-{2,}:?\s*\|?\s*$/.test(line)) {
      continue;
    }
    if (/^\s*\|?\s*Output\s*\|?\s*$/i.test(line)) {
      continue;
    }
    if (/<img\b|<a\s|target=["']_blank["']|utm_source=github/i.test(line)) {
      continue;
    }
    if (/^\s*\|/.test(line) && /\b(Output|image|src=|href=)\b|:?-{2,}:?/i.test(line)) {
      continue;
    }

    line = stripPromptLabelPrefix(line).replace(/```/g, "").replace(/<[^>]+>/g, " ").trimEnd();
    if (line.trim()) {
      cleanedLines.push(line);
    }
  }

  return stripOuterPromptSyntax(cleanedLines.join("\n"))
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function isGenericPromptLabel(label) {
  const cleaned = cleanMarkdownText(label).toLowerCase();
  return cleaned === "" || cleaned === "prompt";
}

function promptEntryScore(entry) {
  const sourceScore = {
    fenced: 30,
    indented: 20,
    inline: 10,
  }[entry.source] || 0;
  const labelScore = isGenericPromptLabel(entry.label) ? 0 : 5;
  const lengthScore = Math.min(entry.prompt.length, 1200) / 1200;
  return sourceScore + labelScore + lengthScore - entry.noiseScore * 10;
}

function makePromptEntry({ label, prompt, index, source }) {
  const rawPrompt = String(prompt || "");
  const cleanedPrompt = cleanPromptText(rawPrompt);
  if (!isLikelyPrompt(cleanedPrompt, label)) {
    return null;
  }
  if (promptNoiseScore(cleanedPrompt) > 0) {
    return null;
  }
  const entry = {
    label: cleanMarkdownText(label || "Prompt") || "Prompt",
    prompt: cleanedPrompt,
    index,
    source,
    noiseScore: promptNoiseScore(rawPrompt),
  };
  return {
    ...entry,
    score: promptEntryScore(entry),
  };
}

function isLikelyPrompt(prompt, label = "") {
  const cleaned = cleanPromptText(prompt);
  if (cleaned.length < 40) {
    return false;
  }
  const commandLike = /(^|\n)\s*(curl|npm|pnpm|yarn|python|pip|git|export|set\s+\w+=|\$)\b/i.test(cleaned);
  const codeLike =
    /\b(function|const|let|var|class|import\s+.+from|module\.exports|require\()\b/.test(cleaned) &&
    !/\bimage|scene|photo|style|subject|composition|lighting|prompt\b/i.test(cleaned);
  if ((commandLike || codeLike) && !/\bprompt\b/i.test(label)) {
    return false;
  }
  const wordCount = cleaned.match(/[A-Za-z\u4e00-\u9fff]{2,}/g)?.length || 0;
  return wordCount >= 8;
}

function extractFencedPromptEntries(content) {
  const entries = [];
  const fencePattern = /```([a-zA-Z0-9_-]*)\n([\s\S]*?)```/g;
  let match;
  while ((match = fencePattern.exec(content))) {
    const language = match[1].toLowerCase();
    const before = content.slice(Math.max(0, match.index - 600), match.index);
    const label = findPromptLabel(before);
    const promptLanguage = ["", "text", "txt", "md", "markdown", "json"].includes(language);
    if (label || promptLanguage) {
      const entry = makePromptEntry({
        label: label || "Prompt",
        prompt: match[2],
        index: match.index,
        source: "fenced",
      });
      if (entry) {
        entries.push(entry);
      }
    }
  }
  return entries;
}

function extractIndentedPromptEntries(content) {
  const entries = [];
  const lines = content.split("\n");
  for (let index = 0; index < lines.length; index += 1) {
    const label = findPromptLabel(lines.slice(Math.max(0, index - 2), index + 1).join("\n"));
    if (!label) {
      continue;
    }

    const collected = [];
    let cursor = index + 1;
    while (cursor < lines.length) {
      const line = lines[cursor];
      if (/^\s{4,}\S/.test(line) || /^\s*>\s{0,3}`?[^`]+`?\s*$/.test(line)) {
        collected.push(line.replace(/^\s{4}/, "").replace(/^\s*>\s?/, ""));
        cursor += 1;
        continue;
      }
      if (collected.length > 0 && line.trim() === "") {
        collected.push("");
        cursor += 1;
        continue;
      }
      break;
    }

    const prompt = stripOuterPromptSyntax(collected.join("\n"));
    const entry = makePromptEntry({ label, prompt, index, source: "indented" });
    if (entry) {
      entries.push(entry);
    }
  }
  return entries;
}

function extractInlinePromptEntries(content) {
  const entries = [];
  const inlinePattern =
    /(?:\*\*)?prompt(?:\s*[A-Z0-9])?(?:\s*[—:-]\s*[^:\n]+)?(?:\*\*)?\s*[:：]?\s*([\s\S]{80,}?)(?=\n{2,}(?:#{2,6}\s|[-*]\s+\*\*|Prompt\s*[A-Z0-9]|!\[)|$)/gi;
  let match;
  while ((match = inlinePattern.exec(content))) {
    const prompt = stripOuterPromptSyntax(
      match[1]
        .split("\n")
        .filter((line) => !/^\s*\|/.test(line))
        .join("\n")
    );
    if (isLikelyPrompt(prompt, "Prompt")) {
      entries.push({ label: "Prompt", prompt, index: match.index });
    }
  }
  return entries;
}

function extractInlinePromptEntriesStrict(content) {
  const entries = [];
  const inlinePattern =
    /(?:^|\n)\s*(?:[-*]\s*)?(?:\*\*)?\s*prompt(?:\s+[A-Z0-9])?\b\s*(?:[:\uFF1A])?\s*(?:\*\*)?\s*(?:[:\uFF1A])?\s*([\s\S]{40,}?)(?=\n{2,}(?:#{2,6}\s|[-*]\s+\*\*|(?:\*\*)?\s*prompt(?:\s+[A-Z0-9])?\b|!\[|\|\s*Output\b)|$)/gi;
  let match;
  while ((match = inlinePattern.exec(content))) {
    const entry = makePromptEntry({
      label: "Prompt",
      prompt: match[1],
      index: match.index,
      source: "inline",
    });
    if (entry) {
      entries.push(entry);
    }
  }
  return entries;
}

function dedupePromptEntries(entries) {
  const byPrompt = new Map();
  for (const entry of entries) {
    const key = hash(entry.prompt.replace(/\s+/g, " ").toLowerCase());
    const previous = byPrompt.get(key);
    if (!previous || entry.score > previous.score) {
      byPrompt.set(key, entry);
    }
  }

  const uniqueEntries = [...byPrompt.values()];
  const genericEntries = uniqueEntries.filter((entry) => isGenericPromptLabel(entry.label));
  const specificEntries = uniqueEntries.filter((entry) => !isGenericPromptLabel(entry.label));

  if (specificEntries.length === 0 && genericEntries.length > 1) {
    return [
      genericEntries.reduce((best, entry) => (entry.score > best.score ? entry : best), genericEntries[0]),
    ];
  }

  return uniqueEntries.sort((a, b) => a.index - b.index || b.score - a.score);
}

function extractPromptEntries(content) {
  const entries = [
    ...extractFencedPromptEntries(content),
    ...extractIndentedPromptEntries(content),
    ...extractInlinePromptEntriesStrict(content),
  ];
  return dedupePromptEntries(entries);
}

function extractExternalUrl(rawTitle, content) {
  const combined = `${rawTitle}\n${content}`;
  const linkPattern = /\[([^\]]+)\]\((https?:\/\/[^)\s]+)(?:\s+"[^"]*")?\)/g;
  let match;
  while ((match = linkPattern.exec(combined))) {
    const url = match[2];
    if (!/\.(png|jpe?g|webp|gif|svg)(\?|#|$)/i.test(url)) {
      return url;
    }
  }
  return null;
}

function extractSize(content, prompt) {
  const text = `${content}\n${prompt}`;
  const dimension = text.match(/\b(\d{3,4})\s*[x×]\s*(\d{3,4})\b/i);
  if (dimension) {
    return `${dimension[1]}x${dimension[2]}`;
  }
  const ratio = text.match(/\b(?:1:1|4:3|3:4|16:9|9:16|2:3|3:2)\b/);
  if (ratio) {
    return ratio[0];
  }
  const orientation = text.match(/\b(square|portrait|landscape|wide|tall|horizontal|vertical)\b/i);
  return orientation ? orientation[1].toLowerCase() : "";
}

function extractQuality(content, prompt) {
  const text = `${content}\n${prompt}`;
  const explicit = text.match(/\b(?:quality|detail|resolution)\s*[:：]?\s*`?(low|medium|high|auto|standard|hd)`?\b/i);
  if (explicit) {
    return explicit[1].toLowerCase();
  }
  if (/\b(8k|4k|ultra[-\s]?detailed|highly detailed|photorealistic|cinematic)\b/i.test(text)) {
    return "high";
  }
  return "";
}

function shouldUseAsCategory(level, title, repoPath) {
  if (level > 2) {
    return false;
  }
  const cleaned = cleanMarkdownText(title).toLowerCase();
  if (!cleaned) {
    return false;
  }
  if (/^(table of contents|contents|overview|getting started|usage|api|news|contribute)/i.test(cleaned)) {
    return false;
  }
  return repoPath !== "README.md" || /\b(case|prompt|gallery|range|image|style|e-commerce|design)\b/i.test(cleaned);
}

function splitPromptBlocks(markdown, repoPath) {
  const lines = normalizeLineEndings(markdown).split("\n");
  let currentCategory = normalizeCategory(categoryFromPath(repoPath), repoPath);
  let activeBlock = null;
  const blocks = [];

  const closeBlock = () => {
    if (activeBlock) {
      activeBlock.content = activeBlock.lines.join("\n");
      delete activeBlock.lines;
      blocks.push(activeBlock);
      activeBlock = null;
    }
  };

  lines.forEach((line, index) => {
    const heading = line.match(/^(#{1,6})\s+(.+?)\s*#*\s*$/);
    if (heading) {
      const level = heading[1].length;
      const title = heading[2].trim();

      if (level <= 2) {
        closeBlock();
        if (shouldUseAsCategory(level, title, repoPath)) {
          currentCategory = normalizeCategory(title, repoPath);
        }
        return;
      }

      if (level <= 4) {
        closeBlock();
        activeBlock = {
          rawTitle: title,
          title: cleanTitle(title),
          category: currentCategory,
          startLine: index + 1,
          lines: [line],
        };
        return;
      }
    }

    if (activeBlock) {
      activeBlock.lines.push(line);
    }
  });

  closeBlock();
  return blocks;
}

function parseMarkdownDocument(source, repoPath, markdown) {
  const blocks = splitPromptBlocks(markdown, repoPath);
  const items = [];

  for (const block of blocks) {
    const promptEntries = extractPromptEntries(block.content);
    if (promptEntries.length === 0) {
      continue;
    }

    const images = extractImages(source, repoPath, block.content);
    const fallbackTitle = block.title || `${block.category} Prompt`;
    const externalUrl = extractExternalUrl(block.rawTitle, block.content);

    promptEntries.forEach((entry, entryIndex) => {
      const promptTitle =
        promptEntries.length > 1 && entry.label && !isGenericPromptLabel(entry.label)
          ? `${fallbackTitle} / ${cleanMarkdownText(entry.label)}`
          : fallbackTitle;
      const sourceUrl = sourceBlobUrl(source, repoPath, block.startLine);
      const prompt = entry.prompt;

      items.push(addChineseDisplayFields({
        id: stableId([source.id, repoPath, block.startLine, entryIndex, prompt]),
        title: promptTitle,
        category: block.category || categoryFromPath(repoPath) || "Uncategorized",
        prompt,
        image: images[0]?.url || "",
        images,
        size: extractSize(block.content, prompt),
        quality: extractQuality(block.content, prompt),
        sourceId: source.id,
        sourceName: source.name,
        sourceRepo: source.repo,
        sourcePath: repoPath,
        sourceLine: block.startLine,
        sourceUrl,
        externalUrl,
      }));
    });
  }

  return items;
}

async function loadSource(source, forceRefresh) {
  const errors = [];
  const docs = new Map();
  const queue = [...source.entryPaths];

  while (queue.length > 0) {
    const repoPath = queue.shift();
    if (docs.has(repoPath)) {
      continue;
    }

    try {
      const loaded = await readMarkdownWithCache(source, repoPath, forceRefresh);
      docs.set(repoPath, loaded);
      for (const linkedPath of discoverLinkedMarkdownPaths(source, repoPath, loaded.markdown)) {
        if (!docs.has(linkedPath) && !queue.includes(linkedPath)) {
          queue.push(linkedPath);
        }
      }
      if (loaded.warning) {
        errors.push(`${repoPath}: 使用旧缓存，远程刷新失败：${loaded.warning}`);
      }
    } catch (error) {
      errors.push(`${repoPath}: ${error.message}`);
    }
  }

  const items = [];
  for (const [repoPath, doc] of docs.entries()) {
    items.push(...parseMarkdownDocument(source, repoPath, doc.markdown));
  }

  return {
    source: {
      id: source.id,
      name: source.name,
      repo: source.repo,
      branch: source.branch,
      documents: docs.size,
    },
    items,
    errors,
  };
}

function buildCategorySummary(items) {
  const counts = new Map();
  for (const item of items) {
    if (!counts.has(item.category)) {
      counts.set(item.category, {
        name: item.category,
        labelZh: item.categoryZh || translateCategoryZh(item.category),
        count: 0,
      });
    }
    counts.get(item.category).count += 1;
  }
  return [...counts.values()].sort((a, b) => b.count - a.count || a.name.localeCompare(b.name));
}

function buildSourceSummary(loadedSources, items) {
  const counts = new Map();
  for (const item of items) {
    counts.set(item.sourceId, (counts.get(item.sourceId) || 0) + 1);
  }
  return loadedSources.map((entry) => ({
    ...entry.source,
    count: counts.get(entry.source.id) || 0,
    errors: entry.errors,
  }));
}

function itemDedupeScore(item) {
  const noisePenalty = promptNoiseScore(item.prompt) * 10000;
  const imageBonus = item.image ? 100 : 0;
  const textScore = Math.min(item.prompt.length, 2000) / 10;
  return imageBonus + textScore - noisePenalty;
}

function dedupeItems(items) {
  const bestByCase = new Map();
  for (const item of items) {
    if (promptNoiseScore(item.prompt) > 0) {
      continue;
    }

    const caseKey = hash(
      [
        item.sourceId,
        item.sourcePath,
        item.sourceLine || item.sourceUrl,
        normalizeLookupKey(item.title),
      ].join("\n")
    );
    const previous = bestByCase.get(caseKey);
    if (!previous || itemDedupeScore(item) > itemDedupeScore(previous)) {
      bestByCase.set(caseKey, item);
    }
  }

  const exactSeen = new Set();
  return [...bestByCase.values()].filter((item) => {
    const exactKey = hash(`${item.sourceId}\n${item.title}\n${item.prompt.replace(/\s+/g, " ").toLowerCase()}`);
    if (exactSeen.has(exactKey)) {
      return false;
    }
    exactSeen.add(exactKey);
    return true;
  });
}

async function loadAllPrompts(forceRefresh = false) {
  if (!forceRefresh && memoryCache && Date.now() - memoryCache.loadedAtMs < 60 * 1000) {
    return memoryCache.data;
  }

  const loadedSources = [];
  for (const source of SOURCES) {
    loadedSources.push(await loadSource(source, forceRefresh));
  }

  const items = dedupeItems(loadedSources.flatMap((entry) => entry.items)).sort((a, b) => {
    const categoryOrder = a.category.localeCompare(b.category);
    return categoryOrder || a.title.localeCompare(b.title);
  });

  const data = {
    updatedAt: nowIso(),
    cacheTtlMs: CACHE_TTL_MS,
    itemCount: items.length,
    categories: buildCategorySummary(items),
    sources: buildSourceSummary(loadedSources, items),
    errors: loadedSources.flatMap((entry) =>
      entry.errors.map((message) => ({ sourceId: entry.source.id, message }))
    ),
    items,
  };

  memoryCache = { loadedAtMs: Date.now(), data };
  return data;
}

function sendJson(response, statusCode, payload) {
  const body = JSON.stringify(payload, null, 2);
  response.writeHead(statusCode, {
    "Content-Type": "application/json; charset=utf-8",
    "Cache-Control": "no-store",
  });
  response.end(body);
}

function sendText(response, statusCode, text) {
  response.writeHead(statusCode, { "Content-Type": "text/plain; charset=utf-8" });
  response.end(text);
}

async function serveStatic(requestUrl, response) {
  const requestedPath = requestUrl.pathname === "/" ? "/index.html" : decodeURIComponent(requestUrl.pathname);
  const normalized = path.normalize(requestedPath).replace(/^([/\\])+/, "");
  const filePath = path.join(PUBLIC_DIR, normalized);
  const relative = path.relative(PUBLIC_DIR, filePath);

  if (relative.startsWith("..") || path.isAbsolute(relative)) {
    sendText(response, 403, "Forbidden");
    return;
  }

  try {
    const body = await fs.readFile(filePath);
    const type = MIME_TYPES.get(path.extname(filePath).toLowerCase()) || "application/octet-stream";
    response.writeHead(200, {
      "Content-Type": type,
      "Cache-Control": "no-cache",
    });
    response.end(body);
  } catch {
    sendText(response, 404, "Not found");
  }
}

async function handleRequest(request, response) {
  const requestUrl = new URL(request.url, `http://${request.headers.host || "localhost"}`);

  if (requestUrl.pathname === "/api/health") {
    sendJson(response, 200, { ok: true, time: nowIso(), sources: SOURCES.length });
    return;
  }

  if (requestUrl.pathname === "/api/prompts") {
    try {
      const forceRefresh = requestUrl.searchParams.get("refresh") === "1";
      const data = await loadAllPrompts(forceRefresh);
      sendJson(response, 200, data);
    } catch (error) {
      sendJson(response, 500, { error: error.message });
    }
    return;
  }

  if (request.method !== "GET" && request.method !== "HEAD") {
    sendText(response, 405, "Method not allowed");
    return;
  }

  await serveStatic(requestUrl, response);
}

function startServer(port = DEFAULT_PORT, options = {}) {
  const server = http.createServer((request, response) => {
    handleRequest(request, response).catch((error) => {
      sendJson(response, 500, { error: error.message });
    });
  });

  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(port, "127.0.0.1", () => {
      server.off("error", reject);
      const address = server.address();
      const actualPort = typeof address === "object" && address ? address.port : port;
      server.localUrl = `http://127.0.0.1:${actualPort}`;
      if (options.log !== false) {
        console.log(`Prompt Library Viewer 已启动：${server.localUrl}`);
      }
      resolve(server);
    });
  });
}

async function runRefreshCli() {
  const data = await loadAllPrompts(true);
  console.log(
    JSON.stringify(
      {
        itemCount: data.itemCount,
        categories: data.categories.length,
        sources: data.sources.map((source) => ({
          id: source.id,
          documents: source.documents,
          count: source.count,
          errors: source.errors.length,
        })),
        errors: data.errors,
      },
      null,
      2
    )
  );
}

if (require.main === module) {
  if (process.argv.includes("--refresh-cache")) {
    runRefreshCli().catch((error) => {
      console.error(error);
      process.exitCode = 1;
    });
  } else {
    const portArg = process.argv.find((arg) => arg.startsWith("--port="));
    const port = portArg ? Number(portArg.slice("--port=".length)) : DEFAULT_PORT;
    startServer(port).catch((error) => {
      console.error(error);
      process.exitCode = 1;
    });
  }
}

module.exports = {
  loadAllPrompts,
  runRefreshCli,
  startServer,
};
