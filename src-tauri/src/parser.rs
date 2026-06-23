use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;
use sha1::{Sha1, Digest};

// --- 数据结构定义 ---

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImageItem {
    pub url: String,
    pub alt: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PromptEntry {
    pub id: String,
    pub title: String,
    pub title_zh: String,
    pub category: String,
    pub category_zh: String,
    pub prompt: String,
    pub summary_zh: String,
    pub image: String,
    pub images: Vec<ImageItem>,
    pub size: String,
    pub quality: String,
    pub source_id: String,
    pub source_name: String,
    pub source_repo: String,
    pub source_path: String,
    pub source_line: usize,
    pub source_url: String,
    pub external_url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SourceSummary {
    pub id: String,
    pub name: String,
    pub repo: String,
    pub branch: String,
    pub documents: usize,
    pub count: usize,
    pub errors: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse {
    pub items: Vec<PromptEntry>,
    pub sources: Vec<SourceSummary>,
    pub categories: Vec<serde_json::Value>,
    pub errors: Vec<String>,
    pub updatedAt: String,
}

#[derive(Clone, Debug)]
pub struct SourceConfig {
    pub id: &'static str,
    pub name: &'static str,
    pub repo: &'static str,
    pub branch: &'static str,
    pub entry_paths: &'static [&'static str],
}

pub const SOURCES: &[SourceConfig] = &[
    SourceConfig {
        id: "evolinkai-awesome-gpt-image-2",
        name: "EvoLinkAI Prompt Cases",
        repo: "EvoLinkAI/awesome-gpt-image-2-API-and-Prompts",
        branch: "main",
        entry_paths: &["README.md"],
    },
    SourceConfig {
        id: "gpt-image2-skill-gallery",
        name: "GPT-Image2 Skill Gallery",
        repo: "wuyoscar/GPT-Image2-Skill",
        branch: "main",
        entry_paths: &["skills/gpt-image/references/gallery.md"],
    },
];

// --- 翻译与分类中文映射表 (静态数据) ---

const CATEGORY_ZH_EXACT: &[(&str, &str)] = &[
    ("uncategorized", "未分类"),
    ("ad creative", "广告创意"),
    ("architecture and interior", "建筑与室内"),
    ("anime and manga", "动漫漫画"),
    ("beauty and lifestyle", "美妆生活"),
    ("brand systems and identity", "品牌识别"),
    ("character", "角色"),
    ("character design", "角色设计"),
    ("cinematic and animation", "电影与动画"),
    ("cinematic film references", "电影镜头参考"),
    ("comparison", "对比测试"),
    ("comparison and community examples", "对比与社区示例"),
    ("data visualization", "数据可视化"),
    ("e commerce", "电商产品"),
    ("ecommerce", "电商产品"),
    ("edit endpoint showcase", "编辑接口示例"),
    ("events and experience", "活动体验"),
    ("fashion editorial", "时尚大片"),
    ("fine art painting", "艺术绘画"),
    ("gaming", "游戏"),
    ("illustration", "插画"),
    ("infographics and field guides", "信息图与指南"),
    ("ink and chinese", "水墨与中文风格"),
    ("isometric", "等距视角"),
    ("liquid realism rules", "液态写实规则"),
    ("more illustration styles", "更多插画风格"),
    ("official openai cookbook examples", "OpenAI 官方示例"),
    ("photography", "摄影"),
    ("pixel art", "像素艺术"),
    ("portrait and photography", "人像摄影"),
    ("portrait", "人像"),
    ("poster and illustration", "海报插画"),
    ("poster", "海报"),
    ("product and food", "产品与美食"),
    ("research paper figures", "论文图表"),
    ("scientific and educational", "科学教育"),
    ("screen photography", "屏幕摄影"),
    ("tattoo design", "纹身设计"),
    ("technical illustration", "技术插画"),
    ("typography and posters", "字体与海报"),
    ("ui", "界面设计"),
    ("ui and social media mockup", "UI 与社媒样机"),
    ("ui ux mockups", "UI/UX 样机"),
    ("watercolor", "水彩"),
];

const TITLE_ZH_EXACT: &[(&str, &str)] = &[
    ("ad creative", "广告创意"),
    ("architecture and interior", "建筑与室内设计"),
    ("anime and manga", "动漫漫画风格"),
    ("beauty and lifestyle", "美妆生活方式"),
    ("brand systems and identity", "品牌系统与识别"),
    ("character design", "角色设计"),
    ("cinematic and animation", "电影与动画风格"),
    ("data visualization", "数据可视化"),
    ("ecommerce", "电商产品图"),
    ("fashion editorial", "时尚大片"),
    ("fine art painting", "艺术绘画"),
    ("infographics and field guides", "信息图与图解指南"),
    ("isometric", "等距视角插画"),
    ("official openai cookbook examples", "OpenAI 官方示例"),
    ("product and food", "产品与美食摄影"),
    ("research paper figures", "论文图表"),
    ("scientific and educational", "科学教育图"),
    ("technical illustration", "技术插画"),
    ("typography and posters", "字体与海报设计"),
    ("ui ux mockups", "UI/UX 样机"),
];

struct KeywordRule {
    keyword: &'static str,
    label: &'static str,
}

// 对应 JS 中的 TITLE_KEYWORDS_ZH
const TITLE_KEYWORDS_ZH: &[KeywordRule] = &[
    KeywordRule { keyword: "character", label: "角色" },
    KeywordRule { keyword: "avatar", label: "角色" },
    KeywordRule { keyword: "mascot", label: "角色" },
    KeywordRule { keyword: "hero", label: "角色" },
    KeywordRule { keyword: "creature", label: "角色" },
    KeywordRule { keyword: "portrait", label: "人像" },
    KeywordRule { keyword: "headshot", label: "人像" },
    KeywordRule { keyword: "selfie", label: "人像" },
    KeywordRule { keyword: "product", label: "产品" },
    KeywordRule { keyword: "packaging", label: "产品" },
    KeywordRule { keyword: "ecommerce", label: "产品" },
    KeywordRule { keyword: "e-commerce", label: "产品" },
    KeywordRule { keyword: "food", label: "美食" },
    KeywordRule { keyword: "drink", label: "美食" },
    KeywordRule { keyword: "beverage", label: "美食" },
    KeywordRule { keyword: "restaurant", label: "美食" },
    KeywordRule { keyword: "poster", label: "海报" },
    KeywordRule { keyword: "flyer", label: "海报" },
    KeywordRule { keyword: "cover", label: "海报" },
    KeywordRule { keyword: "logo", label: "品牌" },
    KeywordRule { keyword: "brand", label: "品牌" },
    KeywordRule { keyword: "identity", label: "品牌" },
    KeywordRule { keyword: "visual system", label: "品牌" },
    KeywordRule { keyword: "ui", label: "界面" },
    KeywordRule { keyword: "ux", label: "界面" },
    KeywordRule { keyword: "interface", label: "界面" },
    KeywordRule { keyword: "dashboard", label: "界面" },
    KeywordRule { keyword: "app", label: "界面" },
    KeywordRule { keyword: "mockup", label: "界面" },
    KeywordRule { keyword: "icon", label: "图标" },
    KeywordRule { keyword: "sticker", label: "图标" },
    KeywordRule { keyword: "illustration", label: "插画" },
    KeywordRule { keyword: "drawing", label: "插画" },
    KeywordRule { keyword: "sketch", label: "插画" },
    KeywordRule { keyword: "photo", label: "摄影" },
    KeywordRule { keyword: "photography", label: "摄影" },
    KeywordRule { keyword: "camera", label: "摄影" },
    KeywordRule { keyword: "cinematic", label: "电影感" },
    KeywordRule { keyword: "film", label: "电影感" },
    KeywordRule { keyword: "movie", label: "电影感" },
    KeywordRule { keyword: "animation", label: "电影感" },
    KeywordRule { keyword: "anime", label: "动漫" },
    KeywordRule { keyword: "manga", label: "动漫" },
    KeywordRule { keyword: "watercolor", label: "绘画" },
    KeywordRule { keyword: "ink", label: "绘画" },
    KeywordRule { keyword: "painting", label: "绘画" },
    KeywordRule { keyword: "fine art", label: "绘画" },
    KeywordRule { keyword: "infographic", label: "图解" },
    KeywordRule { keyword: "diagram", label: "图解" },
    KeywordRule { keyword: "chart", label: "图解" },
    KeywordRule { keyword: "data visualization", label: "图解" },
    KeywordRule { keyword: "field guide", label: "图解" },
    KeywordRule { keyword: "architecture", label: "空间" },
    KeywordRule { keyword: "interior", label: "空间" },
    KeywordRule { keyword: "room", label: "空间" },
    KeywordRule { keyword: "building", label: "空间" },
    KeywordRule { keyword: "fashion", label: "时尚生活" },
    KeywordRule { keyword: "editorial", label: "时尚生活" },
    KeywordRule { keyword: "beauty", label: "时尚生活" },
    KeywordRule { keyword: "lifestyle", label: "时尚生活" },
    KeywordRule { keyword: "pixel", label: "游戏像素" },
    KeywordRule { keyword: "game", label: "游戏像素" },
    KeywordRule { keyword: "gaming", label: "游戏像素" },
    KeywordRule { keyword: "tattoo", label: "纹身" },
];

const SUMMARY_USE_KEYWORDS_ZH: &[KeywordRule] = &[
    KeywordRule { keyword: "character", label: "角色设计" },
    KeywordRule { keyword: "avatar", label: "角色设计" },
    KeywordRule { keyword: "mascot", label: "角色设计" },
    KeywordRule { keyword: "hero", label: "角色设计" },
    KeywordRule { keyword: "creature", label: "角色设计" },
    KeywordRule { keyword: "portrait", label: "人像" },
    KeywordRule { keyword: "headshot", label: "人像" },
    KeywordRule { keyword: "selfie", label: "人像" },
    KeywordRule { keyword: "face", label: "人像" },
    KeywordRule { keyword: "product", label: "产品展示" },
    KeywordRule { keyword: "packaging", label: "产品展示" },
    KeywordRule { keyword: "ecommerce", label: "产品展示" },
    KeywordRule { keyword: "e-commerce", label: "产品展示" },
    KeywordRule { keyword: "sku", label: "产品展示" },
    KeywordRule { keyword: "food", label: "美食饮品" },
    KeywordRule { keyword: "drink", label: "美食饮品" },
    KeywordRule { keyword: "beverage", label: "美食饮品" },
    KeywordRule { keyword: "dessert", label: "美食饮品" },
    KeywordRule { keyword: "restaurant", label: "美食饮品" },
    KeywordRule { keyword: "poster", label: "海报封面" },
    KeywordRule { keyword: "flyer", label: "海报封面" },
    KeywordRule { keyword: "book cover", label: "海报封面" },
    KeywordRule { keyword: "album cover", label: "海报封面" },
    KeywordRule { keyword: "cover art", label: "海报封面" },
    KeywordRule { keyword: "logo", label: "品牌识别" },
    KeywordRule { keyword: "brand", label: "品牌识别" },
    KeywordRule { keyword: "identity", label: "品牌识别" },
    KeywordRule { keyword: "visual system", label: "品牌识别" },
    KeywordRule { keyword: "guidelines", label: "品牌识别" },
    KeywordRule { keyword: "ui", label: "界面样机" },
    KeywordRule { keyword: "ux", label: "界面样机" },
    KeywordRule { keyword: "interface", label: "界面样机" },
    KeywordRule { keyword: "dashboard", label: "界面样机" },
    KeywordRule { keyword: "mobile app", label: "界面样机" },
    KeywordRule { keyword: "web app", label: "界面样机" },
    KeywordRule { keyword: "mockup", label: "界面样机" },
    KeywordRule { keyword: "icon", label: "图标贴纸" },
    KeywordRule { keyword: "sticker", label: "图标贴纸" },
    KeywordRule { keyword: "emoji", label: "图标贴纸" },
    KeywordRule { keyword: "infographic", label: "信息图解" },
    KeywordRule { keyword: "diagram", label: "信息图解" },
    KeywordRule { keyword: "chart", label: "信息图解" },
    KeywordRule { keyword: "map", label: "信息图解" },
    KeywordRule { keyword: "field guide", label: "信息图解" },
    KeywordRule { keyword: "data visualization", label: "信息图解" },
    KeywordRule { keyword: "architecture", label: "建筑室内" },
    KeywordRule { keyword: "interior", label: "建筑室内" },
    KeywordRule { keyword: "room", label: "建筑室内" },
    KeywordRule { keyword: "building", label: "建筑室内" },
    KeywordRule { keyword: "furniture", label: "建筑室内" },
    KeywordRule { keyword: "fashion", label: "时尚生活" },
    KeywordRule { keyword: "editorial", label: "时尚生活" },
    KeywordRule { keyword: "beauty", label: "时尚生活" },
    KeywordRule { keyword: "makeup", label: "时尚生活" },
    KeywordRule { keyword: "lifestyle", label: "时尚生活" },
    KeywordRule { keyword: "game", label: "游戏视觉" },
    KeywordRule { keyword: "gaming", label: "游戏视觉" },
    KeywordRule { keyword: "pixel art", label: "游戏视觉" },
    KeywordRule { keyword: "sprite", label: "游戏视觉" },
    KeywordRule { keyword: "isometric", label: "游戏视觉" },
    KeywordRule { keyword: "scientific", label: "科教技术" },
    KeywordRule { keyword: "educational", label: "科教技术" },
    KeywordRule { keyword: "research paper", label: "科教技术" },
    KeywordRule { keyword: "technical", label: "科教技术" },
    KeywordRule { keyword: "event", label: "活动体验" },
    KeywordRule { keyword: "exhibition", label: "活动体验" },
    KeywordRule { keyword: "experience", label: "活动体验" },
    KeywordRule { keyword: "installation", label: "活动体验" },
    KeywordRule { keyword: "tattoo", label: "纹身图案" },
];

const SUMMARY_STYLE_KEYWORDS_ZH: &[KeywordRule] = &[
    KeywordRule { keyword: "photorealistic", label: "写实" },
    KeywordRule { keyword: "realistic", label: "写实" },
    KeywordRule { keyword: "photo-realistic", label: "写实" },
    KeywordRule { keyword: "cinematic", label: "电影感" },
    KeywordRule { keyword: "film still", label: "电影感" },
    KeywordRule { keyword: "movie", label: "电影感" },
    KeywordRule { keyword: "anamorphic", label: "电影感" },
    KeywordRule { keyword: "anime", label: "动漫" },
    KeywordRule { keyword: "manga", label: "动漫" },
    KeywordRule { keyword: "cel shaded", label: "动漫" },
    KeywordRule { keyword: "watercolor", label: "水彩" },
    KeywordRule { keyword: "ink", label: "水墨" },
    KeywordRule { keyword: "chinese ink", label: "水墨" },
    KeywordRule { keyword: "sumi-e", label: "水墨" },
    KeywordRule { keyword: "oil painting", label: "艺术绘画" },
    KeywordRule { keyword: "fine art", label: "艺术绘画" },
    KeywordRule { keyword: "renaissance", label: "艺术绘画" },
    KeywordRule { keyword: "baroque", label: "艺术绘画" },
    KeywordRule { keyword: "vector", label: "简洁矢量" },
    KeywordRule { keyword: "flat design", label: "简洁矢量" },
    KeywordRule { keyword: "minimal", label: "简洁矢量" },
    KeywordRule { keyword: "isometric", label: "等距视角" },
    KeywordRule { keyword: "retro", label: "复古" },
    KeywordRule { keyword: "vintage", label: "复古" },
    KeywordRule { keyword: "nostalgic", label: "复古" },
    KeywordRule { keyword: "cyberpunk", label: "赛博未来" },
    KeywordRule { keyword: "neon", label: "赛博未来" },
    KeywordRule { keyword: "futuristic", label: "赛博未来" },
    KeywordRule { keyword: "pixel art", label: "像素" },
    KeywordRule { keyword: "8-bit", label: "像素" },
    KeywordRule { keyword: "16-bit", label: "像素" },
    KeywordRule { keyword: "3d", label: "3D 渲染" },
    KeywordRule { keyword: "render", label: "3D 渲染" },
    KeywordRule { keyword: "octane", label: "3D 渲染" },
    KeywordRule { keyword: "blender", label: "3D 渲染" },
    KeywordRule { keyword: "editorial", label: "杂志感" },
    KeywordRule { keyword: "magazine", label: "杂志感" },
    KeywordRule { keyword: "technical", label: "技术图纸" },
    KeywordRule { keyword: "blueprint", label: "技术图纸" },
    KeywordRule { keyword: "schematic", label: "技术图纸" },
];

// --- 基础字符串清理辅助函数 ---

fn normalize_line_endings(s: &str) -> String {
    s.replace("\r\n", "\n").replace('\r', "\n")
}

fn previous_char_boundary(s: &str, mut index: usize) -> usize {
    index = index.min(s.len());
    while index > 0 && !s.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn slice_before_char_boundary(s: &str, end: usize, max_bytes: usize) -> &str {
    let safe_end = previous_char_boundary(s, end);
    let safe_start = previous_char_boundary(s, safe_end.saturating_sub(max_bytes));
    &s[safe_start..safe_end]
}

fn prefix_len_bytes<F>(s: &str, mut predicate: F) -> usize
where
    F: FnMut(char) -> bool,
{
    let mut end = 0;
    for (idx, ch) in s.char_indices() {
        if !predicate(ch) {
            break;
        }
        end = idx + ch.len_utf8();
    }
    end
}

fn title_prefix_separator_len(s: &str) -> Option<usize> {
    let ch = s.chars().next()?;
    if matches!(ch, ':' | '：' | '.' | '-' | '·' | '路') {
        Some(ch.len_utf8())
    } else {
        None
    }
}

fn clean_markdown_text(s: &str) -> String {
    let mut result = s.to_string();
    
    // 移除图片: ![alt](url)
    while let Some(start) = result.find("![") {
        if let Some(mid) = result[start..].find("](") {
            let actual_mid = start + mid;
            if let Some(end) = result[actual_mid..].find(')') {
                let actual_end = actual_mid + end;
                let replacement = result[start+2..actual_mid].to_string();
                result.replace_range(start..=actual_end, &replacement);
                continue;
            }
        }
        break;
    }

    // 移除链接: [text](url) => text
    while let Some(start) = result.find('[') {
        if let Some(mid) = result[start..].find("](") {
            let actual_mid = start + mid;
            if let Some(end) = result[actual_mid..].find(')') {
                let actual_end = actual_mid + end;
                let replacement = result[start+1..actual_mid].to_string();
                result.replace_range(start..=actual_end, &replacement);
                continue;
            }
        }
        break;
    }

    // 移除 HTML 标签: <...>
    while let Some(start) = result.find('<') {
        if let Some(end) = result[start..].find('>') {
            let actual_end = start + end;
            result.replace_range(start..=actual_end, " ");
            continue;
        }
        break;
    }

    // 移除字符: ` * _ ~
    result = result.replace('`', "")
                   .replace('*', "")
                   .replace('_', "")
                   .replace('~', "");

    // 合并多余空格
    let words: Vec<&str> = result.split_whitespace().collect();
    words.join(" ")
}

fn clean_title(raw_title: &str) -> String {
    // 优先取链接里的文字，如 [Case 1: Cyberpunk](...) => Case 1: Cyberpunk
    let mut title = raw_title.to_string();
    if let Some(start) = title.find('[') {
        if let Some(end) = title.find("](") {
            title = title[start + 1..end].to_string();
        }
    }
    
    let cleaned = clean_markdown_text(&title);
    
    // 移除前缀，例如: case 1:, no. 01., 01. 等
    let temp = cleaned.to_lowercase();
    let mut offset = 0;
    
    if temp.starts_with("case") {
        let skip_case = &temp["case".len()..];
        let digits_len = prefix_len_bytes(skip_case, |c| c.is_ascii_digit() || c.is_whitespace());
        let after_digits = &skip_case[digits_len..];
        if let Some(sep_len) = title_prefix_separator_len(after_digits) {
            offset = "case".len() + digits_len + sep_len;
        }
    } else if temp.starts_with("no") {
        let skip_no = &temp["no".len()..];
        let skip_dot = if skip_no.starts_with('.') { &skip_no[1..] } else { skip_no };
        let digits_len = prefix_len_bytes(skip_dot, |c| c.is_ascii_digit() || c.is_whitespace());
        let after_digits = &skip_dot[digits_len..];
        if let Some(sep_len) = title_prefix_separator_len(after_digits) {
            offset = cleaned.len() - after_digits.len() + sep_len;
        }
    } else {
        // 数字开头，如 "01. " 或 "01: "
        let digits_len = prefix_len_bytes(&temp, |c| c.is_ascii_digit() || c.is_whitespace());
        if digits_len > 0 {
            let after_digits = &temp[digits_len..];
            if let Some(sep_len) = title_prefix_separator_len(after_digits) {
                offset = digits_len + sep_len;
            }
        }
    }
    
    let mut title_part = cleaned[offset..].trim().to_string();
    
    // 移除 (by ...) 或 (via ...) 后缀
    let title_lower = title_part.to_lowercase();
    if let Some(idx) = title_lower.rfind(" (by ") {
        if title_lower.ends_with(')') {
            title_part = title_part[..idx].trim().to_string();
        }
    } else if let Some(idx) = title_lower.rfind(" (via ") {
        if title_lower.ends_with(')') {
            title_part = title_part[..idx].trim().to_string();
        }
    }
    
    title_part
}

fn category_from_path(repo_path: &str) -> String {
    let filename = Path::new(repo_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
        
    let cleaned_file = filename
        .replace("gallery-", "")
        .replace("gallery_", "")
        .replace("cases-", "")
        .replace("cases_", "")
        .replace("case-", "")
        .replace("case_", "");
        
    let parts: Vec<&str> = cleaned_file.split(|c| c == '-' || c == '_').filter(|s| !s.is_empty()).collect();
    let capitalized: Vec<String> = parts.into_iter().map(|part| {
        let mut chars = part.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }).collect();
    
    let result = capitalized.join(" ");
    if result.is_empty() { "Uncategorized".to_string() } else { result }
}

fn normalize_category(raw_category: &str, repo_path: &str) -> String {
    let mut cleaned = clean_markdown_text(raw_category);
    // 移除 range :... 等后缀
    if let Some(idx) = cleaned.to_lowercase().find("range") {
        if cleaned[idx..].contains(':') {
            cleaned = cleaned[..idx].to_string();
        }
    }
    
    // 去除 cases, prompts, gallery 等多余修饰词
    let cleaned_lower = cleaned.to_lowercase();
    let words: Vec<&str> = cleaned_lower.split_whitespace().filter(|w| {
        *w != "cases" && *w != "case" && *w != "prompts" && *w != "prompt" && *w != "gallery"
    }).collect();
    
    // 还原原始大小写或合成新名字
    let normalized = words.join(" ");
    let final_name = clean_markdown_text(&normalized);
    
    if final_name.is_empty() {
        category_from_path(repo_path)
    } else {
        // 首字母大写处理
        final_name.split_whitespace().map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<String>>().join(" ")
    }
}

// --- 汉化与中文翻译映射 ---

fn normalize_lookup_key(value: &str) -> String {
    let cleaned = clean_markdown_text(value);
    let with_and = cleaned.replace('&', " and ");
    let mut result = String::new();
    for c in with_and.chars() {
        if c.is_alphanumeric() || c.is_whitespace() {
            result.push(c);
        } else {
            result.push(' ');
        }
    }
    result.split_whitespace().collect::<Vec<&str>>().join(" ").to_lowercase()
}

fn collect_chinese_labels(text: &str, rules: &[KeywordRule], limit: usize) -> Vec<String> {
    let mut labels = Vec::new();
    let mut seen = HashSet::new();
    let text_lower = text.to_lowercase();
    
    for rule in rules {
        // 使用单词边界的简单匹配（避免子字符串误伤，在 Rust 里用简易分词或前后非字母匹配）
        let keyword_lower = rule.keyword.to_lowercase();
        let is_match = if let Some(idx) = text_lower.find(&keyword_lower) {
            let left_ok = idx == 0 || !text_lower.as_bytes()[idx - 1].is_ascii_alphabetic();
            let right_idx = idx + keyword_lower.len();
            let right_ok = right_idx == text_lower.len() || !text_lower.as_bytes()[right_idx].is_ascii_alphabetic();
            left_ok && right_ok
        } else {
            false
        };
        
        if is_match && !seen.contains(rule.label) {
            seen.insert(rule.label);
            labels.push(rule.label.to_string());
            if labels.length_matches(limit) {
                break;
            }
        }
    }
    labels
}

trait LengthMatches {
    fn length_matches(&self, limit: usize) -> bool;
}
impl LengthMatches for Vec<String> {
    fn length_matches(&self, limit: usize) -> bool {
        self.len() >= limit
    }
}

fn translate_category_zh(category: &str) -> String {
    let key = normalize_lookup_key(category);
    for &(eng, zh) in CATEGORY_ZH_EXACT {
        if eng == key {
            return zh.to_string();
        }
    }
    
    // 如果没有精确匹配，从分类名提取关键字
    let labels = collect_chinese_labels(category, SUMMARY_USE_KEYWORDS_ZH, 2);
    let style_labels = collect_chinese_labels(category, SUMMARY_STYLE_KEYWORDS_ZH, 2);
    let mut combined = [labels, style_labels].concat();
    combined.truncate(3);
    
    if combined.is_empty() {
        clean_markdown_text(category)
    } else {
        combined.join(" / ")
    }
}

fn translate_title_zh(title: &str, category_zh: &str) -> String {
    let cleaned = clean_title(title);
    let key = normalize_lookup_key(&cleaned);
    for &(eng, zh) in TITLE_ZH_EXACT {
        if eng == key {
            return zh.to_string();
        }
    }
    
    if cleaned.is_empty() || cleaned.to_lowercase().starts_with("prompt") && cleaned.len() <= 8 {
        return format!("{}提示词", category_zh);
    }
    
    let labels = collect_chinese_labels(&cleaned, TITLE_KEYWORDS_ZH, 4);
    if labels.len() >= 2 {
        return labels.join(" / ");
    }
    if labels.len() == 1 && cleaned.split_whitespace().count() <= 5 {
        return labels[0].clone();
    }
    
    cleaned
}

fn translate_quality_zh(quality: &str) -> String {
    match normalize_lookup_key(quality).as_str() {
        "low" => "低".to_string(),
        "medium" => "中".to_string(),
        "high" => "高".to_string(),
        "auto" => "自动".to_string(),
        "standard" => "标准".to_string(),
        "hd" => "高清".to_string(),
        _ => clean_markdown_text(quality),
    }
}

fn build_summary_zh(title: &str, category: &str, prompt: &str, size: &str, quality: &str, category_zh: &str) -> String {
    let combined_text = format!("{}\n{}\n{}", title, category, prompt);
    let use_labels = collect_chinese_labels(&combined_text, SUMMARY_USE_KEYWORDS_ZH, 5);
    let style_labels = collect_chinese_labels(&combined_text, SUMMARY_STYLE_KEYWORDS_ZH, 5);
    
    let mut specs = Vec::new();
    if !size.is_empty() {
        specs.push(size.to_string());
    }
    if !quality.is_empty() {
        specs.push(format!("质量{}", translate_quality_zh(quality)));
    }
    
    let mut parts = Vec::new();
    if !use_labels.is_empty() {
        parts.push(format!("要点：{}", use_labels.join("、")));
    }
    if !style_labels.is_empty() {
        parts.push(format!("风格：{}", style_labels.join("、")));
    }
    if !specs.is_empty() {
        parts.push(format!("规格：{}", specs.join(" / ")));
    }
    
    if parts.is_empty() {
        format!("用于{}的图片生成提示词，复制时保留英文原文。", category_zh)
    } else {
        format!("{}。", parts.join("；"))
    }
}

// --- 哈希与全局ID生成 ---

fn hash_string(val: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(val.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..])
}

fn stable_id(parts: &[&str]) -> String {
    let input = parts.join("\n");
    hash_string(&input)[..16].to_string()
}

// --- 深度链接与路径处理 ---

fn strip_query_and_hash(href: &str) -> &str {
    href.split('#').next().unwrap_or("").split('?').next().unwrap_or("").trim()
}

fn github_path_from_url(href: &str, source: &SourceConfig) -> Option<String> {
    if !href.starts_with("http") {
        return None;
    }
    
    let url_lower = href.to_lowercase();
    let repo_parts: Vec<&str> = source.repo.split('/').collect();
    if repo_parts.len() < 2 {
        return None;
    }
    
    // 解析 https://github.com/repo/blob/branch/path 或 raw.githubusercontent.com/repo/branch/path
    if url_lower.contains("github.com") {
        if let Some(idx) = url_lower.find(&format!("/{}/{}/blob/{}/", repo_parts[0], repo_parts[1], source.branch.to_lowercase())) {
            let path_start = idx + format!("/{}/{}/blob/{}/", repo_parts[0], repo_parts[1], source.branch).len();
            if path_start < href.len() {
                return Some(href[path_start..].to_string());
            }
        }
    } else if url_lower.contains("raw.githubusercontent.com") {
        if let Some(idx) = url_lower.find(&format!("/{}/{}/{}/", repo_parts[0], repo_parts[1], source.branch.to_lowercase())) {
            let path_start = idx + format!("/{}/{}/{}/", repo_parts[0], repo_parts[1], source.branch).len();
            if path_start < href.len() {
                return Some(href[path_start..].to_string());
            }
        }
    }
    
    None
}

fn resolve_repo_path(source: &SourceConfig, current_path: &str, href: &str) -> Option<String> {
    let clean_href = strip_query_and_hash(href);
    if clean_href.is_empty() || clean_href.starts_with('#') {
        return None;
    }
    
    if let Some(absolute_path) = github_path_from_url(clean_href, source) {
        return Some(absolute_path);
    }
    
    if clean_href.contains("://") || clean_href.starts_with("mailto:") || clean_href.starts_with("tel:") {
        return None; // 外部链接
    }
    
    // 基于当前路径解析相对路径
    let base_dir = Path::new(current_path).parent();
    let resolved = match base_dir {
        None => Path::new(clean_href).to_path_buf(),
        Some(parent) => {
            let mut p = parent.to_path_buf();
            for part in clean_href.split('/') {
                if part == "." || part.is_empty() {
                    continue;
                } else if part == ".." {
                    p.pop();
                } else {
                    p.push(part);
                }
            }
            p
        }
    };
    
    // 转换为 posix 格式斜杠
    let path_str = resolved.to_string_lossy().replace('\\', "/");
    let trimmed = path_str.strip_prefix("./").unwrap_or(&path_str).to_string();
    
    Some(trimmed)
}

fn include_linked_path(source_id: &str, repo_path: &str) -> bool {
    let repo_path_lower = repo_path.to_lowercase();
    match source_id {
        "evolinkai-awesome-gpt-image-2" => {
            if repo_path_lower.starts_with("cases/") && repo_path_lower.ends_with(".md") {
                let sub = &repo_path_lower["cases/".len()..repo_path_lower.len() - ".md".len()];
                !sub.contains('?') && !sub.contains('#')
            } else {
                false
            }
        }
        "gpt-image2-skill-gallery" => {
            let prefix = "skills/gpt-image/references/gallery-";
            if repo_path_lower.starts_with(prefix) && repo_path_lower.ends_with(".md") {
                let sub = &repo_path_lower[prefix.len()..repo_path_lower.len() - ".md".len()];
                !sub.contains('?') && !sub.contains('#')
            } else {
                false
            }
        }
        _ => false
    }
}

fn parse_markdown_links(markdown: &str) -> Vec<String> {
    let mut links = Vec::new();
    let text = markdown.to_string();
    
    // 手写简易 markdown 链接捕获 [text](url)
    let mut cursor = 0;
    while let Some(idx) = text[cursor..].find("[") {
        let start_bracket = cursor + idx;
        if start_bracket > 0 && text.as_bytes()[start_bracket - 1] == b'!' {
            cursor = start_bracket + 1; // 排除图片 ![alt](url)
            continue;
        }
        if let Some(close_bracket) = text[start_bracket..].find("](") {
            let start_paren = start_bracket + close_bracket + 1;
            if let Some(end_paren) = text[start_paren..].find(")") {
                let url_part = &text[start_paren + 1 .. start_paren + end_paren];
                // 剔除包含空格的非法 url，或提取主链接
                let url = url_part.split_whitespace().next().unwrap_or("").to_string();
                if !url.is_empty() {
                    links.push(url);
                }
                cursor = start_paren + end_paren + 1;
                continue;
            }
        }
        cursor = start_bracket + 1;
    }
    links
}

fn discover_linked_markdown_paths(source: &SourceConfig, current_path: &str, markdown: &str) -> Vec<String> {
    let mut discovered = HashSet::new();
    for href in parse_markdown_links(markdown) {
        if let Some(repo_path) = resolve_repo_path(source, current_path, &href) {
            if include_linked_path(source.id, &repo_path) {
                discovered.insert(repo_path);
            }
        }
    }
    discovered.into_iter().collect()
}

// --- 规格提取 (分辨率、质量等) ---

fn extract_external_url(raw_title: &str, content: &str) -> Option<String> {
    let combined = format!("{}\n{}", raw_title, content);
    for href in parse_markdown_links(&combined) {
        if href.starts_with("http") {
            let lower = href.to_lowercase();
            let is_image = lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".webp") || lower.ends_with(".gif") || lower.ends_with(".svg");
            if !is_image {
                return Some(href);
            }
        }
    }
    None
}

fn extract_size(content: &str, prompt: &str) -> String {
    let text = format!("{}\n{}", content, prompt).to_lowercase();
    
    // 匹配分辨率, 如 1024x1024 或 1024×1024
    let mut cursor = 0;
    while let Some(idx) = text[cursor..].find(|c: char| c.is_ascii_digit()) {
        let start_num = cursor + idx;
        let num1: String = text[start_num..].chars().take_while(|c| c.is_ascii_digit()).collect();
        if num1.len() >= 3 && num1.len() <= 4 {
            let after_num1 = &text[start_num + num1.len()..];
            let is_sep = after_num1.starts_with('x') || after_num1.starts_with('×');
            let sep_len = if after_num1.starts_with('x') { 1 } else { '×'.len_utf8() };
            if is_sep {
                let skip_sep = &after_num1[sep_len..];
                let num2_start_trim = skip_sep.trim_start();
                let num2: String = num2_start_trim.chars().take_while(|c| c.is_ascii_digit()).collect();
                if num2.len() >= 3 && num2.len() <= 4 {
                    return format!("{}x{}", num1, num2);
                }
            }
        }
        cursor = start_num + num1.len();
    }
    
    // 匹配宽高比, 如 16:9, 9:16, 1:1, 4:3, 3:4, 2:3, 3:2
    let ratios = &["1:1", "4:3", "3:4", "16:9", "9:16", "2:3", "3:2"];
    for ratio in ratios {
        if text.contains(ratio) {
            return ratio.to_string();
        }
    }
    
    // 匹配方向关键字
    let orientations = &["square", "portrait", "landscape", "wide", "tall", "horizontal", "vertical"];
    for &ori in orientations {
        if text.contains(ori) {
            return ori.to_string();
        }
    }
    
    "".to_string()
}

fn extract_quality(content: &str, prompt: &str) -> String {
    let text = format!("{}\n{}", content, prompt).to_lowercase();
    
    // 显式匹配 quality: low/medium/high
    let indicators = &["quality", "detail", "resolution"];
    for indicator in indicators {
        if let Some(idx) = text.find(indicator) {
            let after = &text[idx + indicator.len()..];
            let after_trim = after.trim_start_matches(|c: char| c == ':' || c == '：' || c.is_whitespace() || c == '`');
            let qualities = &["low", "medium", "high", "auto", "standard", "hd"];
            for &q in qualities {
                if after_trim.starts_with(q) {
                    return q.to_string();
                }
            }
        }
    }
    
    // 隐式高分辨率词汇匹配
    let high_words = &["8k", "4k", "ultra-detailed", "highly detailed", "photorealistic", "cinematic"];
    for word in high_words {
        if text.contains(word) {
            return "high".to_string();
        }
    }
    
    "".to_string()
}

fn resolve_asset_url(source: &SourceConfig, repo_path: &str, href: &str) -> Option<String> {
    let clean_href = strip_query_and_hash(href);
    if clean_href.is_empty() {
        return None;
    }
    if clean_href.starts_with("http") {
        if clean_href.contains("github.com") && clean_href.contains("/blob/") {
            return Some(clean_href
                .replace("https://github.com/", "https://raw.githubusercontent.com/")
                .replace(&format!("/blob/{}/", source.branch), &format!("/{}/", source.branch)));
        }
        return Some(clean_href.to_string());
    }
    resolve_repo_path(source, repo_path, clean_href)
        .map(|resolved| format!("https://raw.githubusercontent.com/{}/{}/{}", source.repo, source.branch, resolved))
}

fn extract_images(source: &SourceConfig, repo_path: &str, content: &str) -> Vec<ImageItem> {
    let mut images = Vec::new();
    let mut seen = HashSet::new();
    let mut add_image = |href: &str, alt: &str| {
        if let Some(url) = resolve_asset_url(source, repo_path, href) {
            if !seen.contains(&url) {
                seen.insert(url.clone());
                images.push(ImageItem { url, alt: clean_markdown_text(alt) });
            }
        }
    };

    // 匹配 markdown 图片: ![alt](url)
    let text = content.to_string();
    let mut cursor = 0;
    while let Some(idx) = text[cursor..].find("![") {
        let start_bracket = cursor + idx;
        if let Some(close_bracket) = text[start_bracket..].find("](") {
            let start_paren = start_bracket + close_bracket + 1;
            if let Some(end_paren) = text[start_paren..].find(")") {
                let alt = &text[start_bracket + 2..start_bracket + close_bracket];
                let href = &text[start_paren + 1..start_paren + end_paren].split_whitespace().next().unwrap_or("");
                add_image(href, alt);
                cursor = start_paren + end_paren + 1;
                continue;
            }
        }
        cursor = start_bracket + 1;
    }

    // 匹配 html 图片标签: <img src="url" alt="alt" />
    let mut cursor = 0;
    while let Some(idx) = text[cursor..].to_lowercase().find("<img") {
        let start_img = cursor + idx;
        if let Some(end_img) = text[start_img..].find('>') {
            let img_tag = &text[start_img..start_img + end_img + 1];
            let src = extract_html_attr(img_tag, "src");
            let alt = extract_html_attr(img_tag, "alt").unwrap_or_default();
            if let Some(url) = src {
                add_image(&url, &alt);
            }
            cursor = start_img + end_img + 1;
            continue;
        }
        cursor = start_img + 1;
    }

    // 匹配 yaml/frontmatter 写法: image/output/preview: 'url'
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('-') || trimmed.starts_with('*') {
            continue;
        }
        if let Some(idx) = trimmed.find(':') {
            let key = trimmed[..idx].trim().to_lowercase();
            if key == "image" || key == "output" || key == "preview" {
                let val = trimmed[idx+1..].trim().trim_matches(|c| c == '`' || c == '\'' || c == '"');
                add_image(val, Path::new(val).file_name().and_then(|f| f.to_str()).unwrap_or(""));
            }
        }
    }

    images
}

fn extract_html_attr(tag: &str, attr: &str) -> Option<String> {
    let lower_tag = tag.to_lowercase();
    let attr_lower = format!("{}=", attr);
    if let Some(idx) = lower_tag.find(&attr_lower) {
        let val_start = idx + attr_lower.len();
        let val_part = &tag[val_start..];
        let quote = val_part.chars().next()?;
        if quote == '"' || quote == '\'' {
            let end_quote = val_part[1..].find(quote)?;
            return Some(val_part[1..1 + end_quote].to_string());
        } else {
            let end_space = val_part.find(|c: char| c.is_whitespace() || c == '/' || c == '>')?;
            return Some(val_part[..end_space].to_string());
        }
    }
    None
}

// --- 提示词内容提炼与清理模块 ---

fn is_prompt_label_line(line: &str) -> bool {
    let trimmed = line.trim().to_lowercase();
    let clean = trimmed.strip_prefix("- ").unwrap_or(trimmed.strip_prefix("* ").unwrap_or(&trimmed)).trim();
    let clean_bold = clean.strip_prefix("**").unwrap_or(clean).strip_suffix("**").unwrap_or(clean).trim();
    
    clean_bold == "prompt" || 
    (clean_bold.starts_with("prompt") && clean_bold.len() <= 9 && clean_bold.chars().nth(6).map_or(false, |c| c.is_alphanumeric())) ||
    clean_bold.ends_with("prompt:") ||
    clean_bold == "prompt:"
}

fn strip_prompt_label_prefix(line: &str) -> String {
    let trimmed = line.trim();
    let mut clean = trimmed.strip_prefix("- ").unwrap_or(trimmed.strip_prefix("* ").unwrap_or(trimmed)).trim();
    if clean.starts_with("**") {
        if let Some(end) = clean[2..].find("**") {
            let label = &clean[2..2+end].to_lowercase();
            if label.starts_with("prompt") {
                clean = clean[2+end+2..].trim();
            }
        }
    }
    clean.strip_prefix(':').unwrap_or(clean).strip_prefix('：').unwrap_or(clean).trim().to_string()
}

fn strip_outer_prompt_syntax(prompt: &str) -> String {
    let mut s = prompt.trim();
    if s.starts_with("```") {
        if let Some(first_line_end) = s.find('\n') {
            s = &s[first_line_end + 1..];
        }
    }
    s = s.strip_prefix('`').unwrap_or(s);
    s = s.strip_suffix("```").unwrap_or(s);
    s = s.strip_suffix('`').unwrap_or(s);
    s.trim().to_string()
}

fn prompt_noise_score(prompt: &str) -> i32 {
    let mut score = 0;
    let lower = prompt.to_lowercase();
    if lower.contains("<img") { score += 1; }
    if lower.contains("<a ") { score += 1; }
    if lower.contains("target=") && (lower.contains("_blank") || lower.contains("'_blank'") || lower.contains("\"_blank\"")) { score += 1; }
    if lower.contains("utm_source=github") { score += 1; }
    if lower.contains("githubusercontent.com/") && lower.contains("/images/") { score += 1; }
    if lower.contains("**prompt") { score += 1; }
    if lower.contains("```") { score += 1; }
    score
}

fn clean_prompt_text(prompt: &str) -> String {
    let lines = normalize_line_endings(prompt);
    let mut cleaned_lines = Vec::new();
    
    for line in lines.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || (trimmed.starts_with("```") && trimmed.len() <= 12) {
            continue;
        }
        if is_prompt_label_line(line) {
            continue;
        }
        // 排除 markdown 表格分割线
        if trimmed.starts_with('|') && trimmed.contains('-') && trimmed.len() >= 5 {
            let table_sep = trimmed.replace('|', "").replace(':', "").replace('-', "").trim().is_empty();
            if table_sep {
                continue;
            }
        }
        // 排除 Output 表头
        if trimmed.to_lowercase().replace('|', "").trim() == "output" {
            continue;
        }
        if trimmed.contains("<img") || trimmed.contains("<a ") || trimmed.contains("target=") || trimmed.contains("utm_source=") {
            continue;
        }
        // 排除包含表格格式和其它噪点的行
        if trimmed.starts_with('|') && (trimmed.to_lowercase().contains("output") || trimmed.to_lowercase().contains("image") || trimmed.contains("src=") || trimmed.contains("href=")) {
            continue;
        }
        
        let mut clean_line = strip_prompt_label_prefix(line).replace("```", "");
        // 去除简单的 html 标签
        while let Some(start) = clean_line.find('<') {
            if let Some(end) = clean_line[start..].find('>') {
                clean_line.replace_range(start..=start+end, " ");
                continue;
            }
            break;
        }
        
        let final_line = clean_line.trim_end().to_string();
        if !final_line.trim().is_empty() {
            cleaned_lines.push(final_line);
        }
    }
    
    let result = cleaned_lines.join("\n");
    let outer_cleaned = strip_outer_prompt_syntax(&result);
    // 合并多余空行
    let mut final_text = String::new();
    let mut empty_count = 0;
    for line in outer_cleaned.lines() {
        if line.trim().is_empty() {
            empty_count += 1;
            if empty_count < 2 {
                final_text.push('\n');
            }
        } else {
            empty_count = 0;
            if !final_text.is_empty() && !final_text.ends_with('\n') {
                final_text.push('\n');
            }
            final_text.push_str(line);
        }
    }
    
    final_text.trim().to_string()
}

fn is_generic_prompt_label(label: &str) -> bool {
    let cleaned = clean_markdown_text(label).to_lowercase();
    cleaned.is_empty() || cleaned == "prompt"
}

fn is_likely_prompt(prompt: &str, label: &str) -> bool {
    let cleaned = clean_prompt_text(prompt);
    if cleaned.len() < 40 {
        return false;
    }
    
    // 排除命令行代码
    let is_command = cleaned.lines().any(|line| {
        let t = line.trim();
        t.starts_with("curl ") || t.starts_with("npm ") || t.starts_with("pnpm ") || t.starts_with("yarn ") || t.starts_with("python ") || t.starts_with("pip ") || t.starts_with("git ") || t.starts_with("export ") || t.starts_with("$ ")
    });
    
    // 排除纯程序代码
    let is_code = (cleaned.contains("function ") || cleaned.contains("const ") || cleaned.contains("let ") || cleaned.contains("var ") || cleaned.contains("class ") || cleaned.contains("import ") || cleaned.contains("require("))
                  && !["image", "scene", "photo", "style", "subject", "composition", "lighting", "prompt"].iter().any(|&w| cleaned.to_lowercase().contains(w));
                  
    if (is_command || is_code) && !label.to_lowercase().contains("prompt") {
        return false;
    }
    
    // 单词数统计
    let mut word_count = 0;
    let mut in_word = false;
    for c in cleaned.chars() {
        if c.is_alphabetic() {
            if !in_word {
                in_word = true;
                word_count += 1;
            }
        } else {
            in_word = false;
        }
    }
    
    word_count >= 8
}

fn prompt_entry_score(label: &str, prompt: &str, noise_score: i32, source_type: &str) -> f64 {
    let source_score = match source_type {
        "fenced" => 30.0,
        "indented" => 20.0,
        "inline" => 10.0,
        _ => 0.0,
    };
    let label_score = if is_generic_prompt_label(label) { 0.0 } else { 5.0 };
    let len_score = (prompt.len() as f64).min(1200.0) / 1200.0;
    
    source_score + label_score + len_score - (noise_score as f64) * 10.0
}

struct TempEntry {
    label: String,
    prompt: String,
    index: usize,
    source: String,
    noise_score: i32,
    score: f64,
}

fn make_prompt_entry(label: &str, raw_prompt: &str, index: usize, source_type: &str) -> Option<TempEntry> {
    let cleaned = clean_prompt_text(raw_prompt);
    if !is_likely_prompt(&cleaned, label) {
        return None;
    }
    
    let noise = prompt_noise_score(&cleaned);
    if noise > 0 {
        return None;
    }
    
    let label_text = if clean_markdown_text(label).is_empty() { "Prompt".to_string() } else { clean_markdown_text(label) };
    let score = prompt_entry_score(&label_text, &cleaned, noise, source_type);
    
    Some(TempEntry {
        label: label_text,
        prompt: cleaned,
        index,
        source: source_type.to_string(),
        noise_score: noise,
        score,
    })
}

// --- 各种格式 Prompt 抓取逻辑 ---

fn find_prompt_label(before_fence: &str) -> String {
    let lines: Vec<&str> = before_fence.lines().collect();
    // 取最后 6 行
    let start_idx = if lines.len() > 6 { lines.len() - 6 } else { 0 };
    let check_lines = &lines[start_idx..];
    
    for &line in check_lines.iter().rev() {
        let cleaned = clean_markdown_text(line);
        let cleaned_lower = cleaned.to_lowercase();
        // 匹配 prompt 或 prompt 1
        if let Some(idx) = cleaned_lower.find("prompt") {
            let label_start = idx;
            // 简单切分 prompt 后的后缀
            let suffix = &cleaned[label_start..];
            if suffix.to_lowercase().starts_with("prompt") {
                let parts: Vec<&str> = suffix.split(|c| c == ':' || c == '：' || c == '—' || c == '-').collect();
                if !parts.is_empty() {
                    let main_label = parts[0].trim();
                    let desc = if parts.len() > 1 { format!(" - {}", parts[1].trim()) } else { "".to_string() };
                    return format!("{}{}", main_label, desc).trim().to_string();
                }
            }
        }
    }
    "".to_string()
}

fn extract_fenced_prompt_entries(content: &str) -> Vec<TempEntry> {
    let mut entries = Vec::new();
    let text = content.to_string();
    let mut cursor = 0;
    
    while let Some(idx) = text[cursor..].find("```") {
        let start_fence = cursor + idx;
        let rest = &text[start_fence + 3..];
        if let Some(end_fence) = rest.find("```") {
            let block_content = &rest[..end_fence];
            let first_line: String = block_content.chars().take_while(|&c| c != '\n').collect();
            let lang = first_line.trim().to_lowercase();
            let code = &block_content[first_line.len()..];
            
            // 往前半段文本获取 label
            let before = slice_before_char_boundary(&text, start_fence, 600);
            let label = find_prompt_label(before);
            
            let is_text_lang = lang.is_empty() || lang == "text" || lang == "txt" || lang == "md" || lang == "markdown" || lang == "json";
            if !label.is_empty() || is_text_lang {
                if let Some(entry) = make_prompt_entry(
                    if label.is_empty() { "Prompt" } else { &label },
                    code,
                    start_fence,
                    "fenced"
                ) {
                    entries.push(entry);
                }
            }
            cursor = start_fence + 3 + end_fence + 3;
            continue;
        }
        cursor = start_fence + 3;
    }
    entries
}

fn extract_indented_prompt_entries(content: &str) -> Vec<TempEntry> {
    let mut entries = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    
    let mut index = 0;
    while index < lines.len() {
        // 往前看 2 行找 label
        let start_look = if index > 2 { index - 2 } else { 0 };
        let before_text = lines[start_look ..= index].join("\n");
        let label = find_prompt_label(&before_text);
        
        if label.is_empty() {
            index += 1;
            continue;
        }
        
        // 收集缩进行或引用行
        let mut collected = Vec::new();
        let mut cursor = index + 1;
        while cursor < lines.len() {
            let line = lines[cursor];
            let trimmed = line.trim();
            
            // 4个空格缩进，或者 > 开头的引用块
            let is_indented = line.starts_with("    ") || line.starts_with("\t");
            let is_quote = trimmed.starts_with('>') && !trimmed.contains("![") && !trimmed.contains("<img");
            
            if is_indented {
                let strip_indent = line.strip_prefix("    ").unwrap_or(line.strip_prefix('\t').unwrap_or(line));
                collected.push(strip_indent);
                cursor += 1;
            } else if is_quote {
                let strip_quote = trimmed.strip_prefix('>').unwrap_or(trimmed).trim();
                collected.push(strip_quote);
                cursor += 1;
            } else if collected.len() > 0 && trimmed.is_empty() {
                collected.push("");
                cursor += 1;
            } else {
                break;
            }
        }
        
        if !collected.is_empty() {
            let code = collected.join("\n");
            let cleaned_code = strip_outer_prompt_syntax(&code);
            if let Some(entry) = make_prompt_entry(&label, &cleaned_code, index, "indented") {
                entries.push(entry);
            }
            index = cursor;
        } else {
            index += 1;
        }
    }
    entries
}

fn extract_inline_prompt_entries_strict(content: &str) -> Vec<TempEntry> {
    let mut entries = Vec::new();
    let text = content.to_string();
    
    // 采用特征切分法替代正则匹配: 寻找带冒号的 Prompt 块
    let mut cursor = 0;
    let text_lower = text.to_lowercase();
    while let Some(idx) = text_lower[cursor..].find("prompt") {
        let prompt_start = cursor + idx;
        let rest = &text[prompt_start..];
        
        // 验证 Prompt 冒号
        let mut valid_prefix = false;
        let mut val_offset = 0;
        let next_chars: String = rest.chars().take(20).collect();
        for &sep in &[":", "：", "**:", "**："] {
            if let Some(sep_idx) = next_chars.find(sep) {
                valid_prefix = true;
                val_offset = sep_idx + sep.len();
                break;
            }
        }
        
        if !valid_prefix {
            cursor = prompt_start + "prompt".len();
            continue;
        }
        
        // 收集后续正文直到双换行或遇到新的标题、表格或图片
        let body_part = &rest[val_offset..];
        let mut collected_lines = Vec::new();
        for line in body_part.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break; // 遇到空行结束
            }
            let is_next_indicator = trimmed.starts_with('#') || 
                                     trimmed.starts_with("- ") || 
                                     trimmed.starts_with("* ") || 
                                     trimmed.to_lowercase().starts_with("prompt") ||
                                     trimmed.starts_with("![") || 
                                     trimmed.starts_with('|');
            if is_next_indicator && !collected_lines.is_empty() {
                break;
            }
            collected_lines.push(line);
        }
        
        if !collected_lines.is_empty() {
            let prompt_text = collected_lines.join("\n");
            let cleaned_prompt = strip_outer_prompt_syntax(&prompt_text);
            if let Some(entry) = make_prompt_entry("Prompt", &cleaned_prompt, prompt_start, "inline") {
                entries.push(entry);
            }
            cursor = prompt_start + val_offset + prompt_text.len();
        } else {
            cursor = prompt_start + "prompt".len();
        }
    }
    
    entries
}

fn dedupe_prompt_entries(entries: Vec<TempEntry>) -> Vec<TempEntry> {
    let mut by_prompt: HashMap<String, TempEntry> = HashMap::new();
    for entry in entries {
        // 使用去掉空格转小写的 key 做去重
        let key = hash_string(&entry.prompt.split_whitespace().collect::<Vec<&str>>().join(" ").to_lowercase());
        let is_better = match by_prompt.get(&key) {
            None => true,
            Some(prev_entry) => entry.score > prev_entry.score,
        };
        if is_better {
            by_prompt.insert(key, entry);
        }
    }
    
    let mut unique_entries: Vec<TempEntry> = by_prompt.into_values().collect();
    let generic_count = unique_entries.iter().filter(|e| is_generic_prompt_label(&e.label)).count();
    let specific_count = unique_entries.iter().filter(|e| !is_generic_prompt_label(&e.label)).count();
    
    if specific_count == 0 && generic_count > 1 {
        // 如果只有泛指，仅保留分最高的那一个
        let mut best = unique_entries.remove(0);
        for entry in unique_entries {
            if entry.score > best.score {
                best = entry;
            }
        }
        return vec![best];
    }
    
    // 按原始 index 升序，score 降序排序
    unique_entries.sort_by(|a, b| {
        a.index.cmp(&b.index).then_with(|| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal))
    });
    
    unique_entries
}

fn extract_prompt_entries(content: &str) -> Vec<TempEntry> {
    let mut entries = Vec::new();
    entries.extend(extract_fenced_prompt_entries(content));
    entries.extend(extract_indented_prompt_entries(content));
    entries.extend(extract_inline_prompt_entries_strict(content));
    dedupe_prompt_entries(entries)
}

// --- 模块分割 (Block 分割) ---

struct PromptBlock {
    raw_title: String,
    title: String,
    category: String,
    start_line: usize,
    content: String,
}

fn should_use_as_category(level: usize, title: &str, repo_path: &str) -> bool {
    if level > 2 {
        return false;
    }
    let cleaned = clean_markdown_text(title).to_lowercase();
    if cleaned.is_empty() {
        return false;
    }
    let is_system_heading = ["table of contents", "contents", "overview", "getting started", "usage", "api", "news", "contribute"].iter().any(|&sys| cleaned.starts_with(sys));
    if is_system_heading {
        return false;
    }
    
    // 如果是 README.md，需要包含某些关键字才当做有效的提示词分类，防止把整个仓库介绍误识别为分类
    if repo_path == "README.md" {
        let kws = &["case", "prompt", "gallery", "range", "image", "style", "e-commerce", "design"];
        kws.iter().any(|&kw| cleaned.contains(kw))
    } else {
        true
    }
}

fn split_prompt_blocks(markdown: &str, repo_path: &str) -> Vec<PromptBlock> {
    let text = normalize_line_endings(markdown);
    let mut current_category = normalize_category(&category_from_path(repo_path), repo_path);
    let mut blocks = Vec::new();
    
    struct TempBlock {
        raw_title: String,
        title: String,
        category: String,
        start_line: usize,
        lines: Vec<String>,
    }
    
    let mut active_block: Option<TempBlock> = None;
    
    let mut close_block = |block_opt: &mut Option<TempBlock>, blocks_list: &mut Vec<PromptBlock>| {
        if let Some(b) = block_opt.take() {
            blocks_list.push(PromptBlock {
                raw_title: b.raw_title,
                title: b.title,
                category: b.category,
                start_line: b.start_line,
                content: b.lines.join("\n"),
            });
        }
    };
    
    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        // 匹配 heading, 如 ### Title
        let is_heading = trimmed.starts_with('#') && trimmed.contains(' ');
        if is_heading {
            let heading_chars: String = trimmed.chars().take_while(|&c| c == '#').collect();
            let level = heading_chars.len();
            if level >= 1 && level <= 6 {
                let title = trimmed[level..].trim().trim_end_matches('#').trim().to_string();
                
                if level <= 2 {
                    close_block(&mut active_block, &mut blocks);
                    if should_use_as_category(level, &title, repo_path) {
                        current_category = normalize_category(&title, repo_path);
                    }
                    continue;
                }
                
                if level <= 4 {
                    close_block(&mut active_block, &mut blocks);
                    active_block = Some(TempBlock {
                        raw_title: title.clone(),
                        title: clean_title(&title),
                        category: current_category.clone(),
                        start_line: idx + 1,
                        lines: vec![line.to_string()],
                    });
                    continue;
                }
            }
        }
        
        if let Some(ref mut b) = active_block {
            b.lines.push(line.to_string());
        }
    }
    
    close_block(&mut active_block, &mut blocks);
    blocks
}

// --- 最终的 Markdown 解析集成 ---

fn parse_markdown_document(source: &SourceConfig, repo_path: &str, markdown: &str) -> Vec<PromptEntry> {
    let blocks = split_prompt_blocks(markdown, repo_path);
    let mut items = Vec::new();
    
    for block in blocks {
        let prompt_entries = extract_prompt_entries(&block.content);
        if prompt_entries.is_empty() {
            continue;
        }
        
        let images = extract_images(source, repo_path, &block.content);
        let fallback_title = if block.title.is_empty() { format!("{} Prompt", block.category) } else { block.title.clone() };
        let external_url = extract_external_url(&block.raw_title, &block.content);
        let default_category_fallback = category_from_path(repo_path);
        
        for (entry_idx, entry) in prompt_entries.iter().enumerate() {
            let is_specific_label = !is_generic_prompt_label(&entry.label);
            let prompt_title = if prompt_entries.len() > 1 && is_specific_label {
                format!("{} / {}", fallback_title, clean_markdown_text(&entry.label))
            } else {
                fallback_title.clone()
            };
            
            // 生成 GitHub 代码浏览源 URL，如 https://github.com/repo/blob/branch/path#Lline
            let source_url = format!("https://github.com/{}/blob/{}/{}#L{}", source.repo, source.branch, repo_path, block.start_line);
            let category = if block.category.is_empty() { default_category_fallback.clone() } else { block.category.clone() };
            let category_zh = translate_category_zh(&category);
            let title_zh = translate_title_zh(&prompt_title, &category_zh);
            
            let size = extract_size(&block.content, &entry.prompt);
            let quality = extract_quality(&block.content, &entry.prompt);
            let summary_zh = build_summary_zh(&prompt_title, &category, &entry.prompt, &size, &quality, &category_zh);
            
            let first_image = images.first().map(|img| img.url.clone()).unwrap_or_default();
            
            items.push(PromptEntry {
                id: stable_id(&[source.id, repo_path, &block.start_line.to_string(), &entry_idx.to_string(), &entry.prompt]),
                title: prompt_title,
                title_zh,
                category,
                category_zh,
                prompt: entry.prompt.clone(),
                summary_zh,
                image: first_image,
                images: images.clone(),
                size,
                quality,
                source_id: source.id.to_string(),
                source_name: source.name.to_string(),
                source_repo: source.repo.to_string(),
                source_path: repo_path.to_string(),
                source_line: block.start_line,
                source_url,
                external_url: external_url.clone(),
            });
        }
    }
    
    items
}

// --- 缓存网络与数据抓取逻辑 ---

fn cache_paths(cache_dir: &Path, source: &SourceConfig, repo_path: &str) -> (PathBuf, PathBuf) {
    let key = hash_string(&format!("{}@{}:{}", source.repo, source.branch, repo_path));
    let dir = cache_dir.join(source.id);
    let body_path = dir.join(format!("{}.md", key));
    let meta_path = dir.join(format!("{}.json", key));
    (body_path, meta_path)
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CacheMeta {
    repo_path: String,
    url: String,
    fetched_at: String,
    source_id: String,
}

async fn read_markdown_with_cache(
    cache_dir: &Path,
    source: &SourceConfig,
    repo_path: &str,
    force_refresh: bool
) -> Result<(String, Option<String>), String> {
    let (body_path, meta_path) = cache_paths(cache_dir, source, repo_path);
    let has_body = body_path.exists();

    // 普通启动优先返回本地缓存，避免首屏被 GitHub 网络请求阻塞。
    if !force_refresh && has_body {
        if let Ok(markdown) = fs::read_to_string(&body_path) {
            return Ok((markdown, None));
        }
    }
    
    // 从 GitHub 获取最新内容
    let url = format!("https://raw.githubusercontent.com/{}/{}/{}", source.repo, source.branch, repo_path);
    
    let client = reqwest::Client::builder()
        .user_agent("Prompt-Library-Viewer/0.2 (Tauri)")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
        
    match client.get(&url).send().await {
        Ok(res) if res.status().is_success() => {
            if let Ok(markdown) = res.text().await {
                // 保存缓存
                let now_iso = chrono::Utc::now().to_rfc3339();
                let _ = fs::create_dir_all(body_path.parent().unwrap());
                let _ = fs::write(&body_path, &markdown);
                
                let meta = CacheMeta {
                    repo_path: repo_path.to_string(),
                    url: url.clone(),
                    fetched_at: now_iso,
                    source_id: source.id.to_string(),
                };
                if let Ok(meta_json) = serde_json::to_string_pretty(&meta) {
                    let _ = fs::write(&meta_path, &meta_json);
                }
                
                Ok((markdown, None))
            } else {
                Err("无法解析网络响应".to_string())
            }
        }
        Ok(res) => {
            let err_msg = format!("GitHub 返回 HTTP {}", res.status());
            // 如果拉取失败但本地有老缓存，进行降级返回
            if has_body {
                let markdown = fs::read_to_string(&body_path).map_err(|e| e.to_string())?;
                Ok((markdown, Some(err_msg)))
            } else {
                Err(err_msg)
            }
        }
        Err(e) => {
            let err_msg = e.to_string();
            if has_body {
                let markdown = fs::read_to_string(&body_path).map_err(|e| e.to_string())?;
                Ok((markdown, Some(err_msg)))
            } else {
                Err(err_msg)
            }
        }
    }
}

// --- 数据总入口 ---

pub async fn load_and_parse_prompts(cache_dir: PathBuf, force_refresh: bool) -> Result<ApiResponse, String> {
    let mut all_items = Vec::new();
    let mut all_sources = Vec::new();
    let mut all_errors = Vec::new();
    
    // 初始化本地缓存总文件夹
    let _ = fs::create_dir_all(&cache_dir);
    let response_cache_path = cache_dir.join("api-response.json");

    // 普通启动优先读取解析后的快照，避免每次重新解析全部 Markdown。
    if !force_refresh {
        if let Ok(cache_content) = fs::read_to_string(&response_cache_path) {
            if let Ok(response) = serde_json::from_str::<ApiResponse>(&cache_content) {
                return Ok(response);
            }
        }
    }
    
    for source in SOURCES {
        let mut queue = Vec::new();
        for &p in source.entry_paths {
            queue.push(p.to_string());
        }
        
        let mut docs = HashMap::new();
        let mut source_errors = Vec::new();
        
        // 广度探索所有 linked markdown 文件
        let mut visited = HashSet::new();
        while let Some(repo_path) = queue.pop() {
            if visited.contains(&repo_path) {
                continue;
            }
            visited.insert(repo_path.clone());
            
            match read_markdown_with_cache(&cache_dir, source, &repo_path, force_refresh).await {
                Ok((markdown, warning)) => {
                    // 探索文档中所有关联的其它 Markdown 文件并加入队列
                    let links = discover_linked_markdown_paths(source, &repo_path, &markdown);
                    for link in links {
                        if !visited.contains(&link) && !queue.contains(&link) {
                            queue.push(link);
                        }
                    }
                    docs.insert(repo_path.clone(), markdown);
                    if let Some(warn) = warning {
                        let warn_msg = format!("{}: 使用老缓存（刷新失败：{}）", repo_path, warn);
                        source_errors.push(warn_msg.clone());
                        all_errors.push(warn_msg);
                    }
                }
                Err(e) => {
                    let err_msg = format!("{}: 抓取错误 - {}", repo_path, e);
                    source_errors.push(err_msg.clone());
                    all_errors.push(err_msg);
                }
            }
        }
        
        // 对该源中抓取的所有 markdown 文件进行内容解析
        let mut source_items = Vec::new();
        for (repo_path, markdown) in &docs {
            let parsed = parse_markdown_document(source, repo_path, markdown);
            source_items.extend(parsed);
        }
        
        let docs_count = docs.len();
        let items_count = source_items.len();
        all_items.extend(source_items);
        
        all_sources.push(SourceSummary {
            id: source.id.to_string(),
            name: source.name.to_string(),
            repo: source.repo.to_string(),
            branch: source.branch.to_string(),
            documents: docs_count,
            count: items_count,
            errors: source_errors,
        });
    }
    
    // 生成分类统计列表
    let mut category_counts = HashMap::new();
    for item in &all_items {
        let entry = category_counts.entry(item.category.clone()).or_insert_with(|| {
            serde_json::json!({
                "name": item.category,
                "labelZh": item.category_zh,
                "count": 0
            })
        });
        if let Some(count_val) = entry.get_mut("count") {
            if let Some(c) = count_val.as_u64() {
                *count_val = serde_json::Value::from(c + 1);
            }
        }
    }
    
    let mut categories: Vec<serde_json::Value> = category_counts.into_values().collect();
    // 按统计数量降序，名字升序排列
    categories.sort_by(|a, b| {
        let cnt_a = a.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
        let cnt_b = b.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
        let name_a = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let name_b = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        cnt_b.cmp(&cnt_a).then_with(|| name_a.cmp(name_b))
    });
    
    let now_str = chrono::Utc::now().to_rfc3339();
    
    let response = ApiResponse {
        items: all_items,
        sources: all_sources,
        categories,
        errors: all_errors,
        updatedAt: now_str,
    };

    if let Ok(cache_content) = serde_json::to_string(&response) {
        let _ = fs::write(response_cache_path, cache_content);
    }

    Ok(response)
}
