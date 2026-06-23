const state = {
  items: [],
  sources: [],
  categories: [],
  errors: [],
  query: "",
  sourceId: "",
  category: "",
  selectedItem: null,
};

window.onerror = function(message, source, lineno, colno, error) {
  alert(`Global Error: ${message}\nLine: ${lineno}\nError: ${error?.message}`);
};
window.addEventListener('unhandledrejection', function(event) {
  alert(`Unhandled Promise Rejection: ${event.reason}`);
});

const elements = {
  statusText: document.querySelector("#statusText"),
  refreshButton: document.querySelector("#refreshButton"),
  clearButton: document.querySelector("#clearButton"),
  searchInput: document.querySelector("#searchInput"),
  sourceSelect: document.querySelector("#sourceSelect"),
  categoryStrip: document.querySelector("#categoryStrip"),
  summaryGrid: document.querySelector("#summaryGrid"),
  notice: document.querySelector("#notice"),
  cardsGrid: document.querySelector("#cardsGrid"),
  emptyState: document.querySelector("#emptyState"),
  toast: document.querySelector("#toast"),
  dialog: document.querySelector("#promptDialog"),
  dialogClose: document.querySelector("#dialogClose"),
  dialogTitle: document.querySelector("#dialogTitle"),
  dialogMeta: document.querySelector("#dialogMeta"),
  dialogPrompt: document.querySelector("#dialogPrompt"),
  dialogImage: document.querySelector("#dialogImage"),
  dialogSource: document.querySelector("#dialogSource"),
  dialogCopy: document.querySelector("#dialogCopy"),
};

function normalizeText(value) {
  return String(value || "").toLowerCase().trim();
}

function formatDate(value) {
  if (!value) {
    return "";
  }
  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(value));
}

function shortenPrompt(prompt, maxLength = 270) {
  const normalized = String(prompt || "").replace(/\s+/g, " ").trim();
  if (normalized.length <= maxLength) {
    return normalized;
  }
  return `${normalized.slice(0, maxLength).trim()}...`;
}

function getDisplayTitle(item) {
  return item.titleZh || item.title || "未命名提示词";
}

function getDisplayCategory(item) {
  return item.categoryZh || item.category || "未分类";
}

function getCategoryLabel(category) {
  return category.labelZh || category.name || "未分类";
}

function getDisplaySummary(item) {
  return item.summaryZh || shortenPrompt(item.prompt);
}

function showToast(message) {
  elements.toast.textContent = message;
  elements.toast.classList.add("show");
  window.clearTimeout(showToast.timer);
  showToast.timer = window.setTimeout(() => {
    elements.toast.classList.remove("show");
  }, 1800);
}

async function copyPrompt(prompt) {
  try {
    await navigator.clipboard.writeText(prompt);
    showToast("Prompt 已复制");
  } catch {
    const textarea = document.createElement("textarea");
    textarea.value = prompt;
    textarea.style.position = "fixed";
    textarea.style.opacity = "0";
    document.body.append(textarea);
    textarea.select();
    document.execCommand("copy");
    textarea.remove();
    showToast("Prompt 已复制");
  }
}

function getFilteredItems() {
  const query = normalizeText(state.query);
  return state.items.filter((item) => {
    if (state.sourceId && item.sourceId !== state.sourceId) {
      return false;
    }
    if (state.category && item.category !== state.category) {
      return false;
    }
    if (!query) {
      return true;
    }
    const haystack = normalizeText(
      [
        item.titleZh,
        item.categoryZh,
        item.summaryZh,
        item.title,
        item.category,
        item.prompt,
        item.size,
        item.quality,
        item.sourceName,
      ].join(" ")
    );
    return haystack.includes(query);
  });
}

function countFilteredCategories() {
  const counts = new Map();
  for (const item of state.items) {
    if (state.sourceId && item.sourceId !== state.sourceId) {
      continue;
    }
    if (!counts.has(item.category)) {
      counts.set(item.category, {
        name: item.category,
        labelZh: getDisplayCategory(item),
        count: 0,
      });
    }
    counts.get(item.category).count += 1;
  }
  return [...counts.values()].sort((a, b) => b.count - a.count || a.name.localeCompare(b.name));
}

function renderSummary(filteredItems) {
  const withImages = filteredItems.filter((item) => item.images?.length).length;
  const sourceCount = new Set(filteredItems.map((item) => item.sourceId)).size;
  const categoryCount = new Set(filteredItems.map((item) => item.category)).size;
  const metrics = [
    ["提示词", filteredItems.length],
    ["分类", categoryCount],
    ["来源", sourceCount],
    ["含预览图", withImages],
  ];

  elements.summaryGrid.replaceChildren(
    ...metrics.map(([label, value]) => {
      const node = document.createElement("article");
      node.className = "metric";
      const strong = document.createElement("strong");
      strong.textContent = value;
      const span = document.createElement("span");
      span.textContent = label;
      node.append(strong, span);
      return node;
    })
  );
}

function renderSourceOptions() {
  const current = state.sourceId;
  const options = [new Option("全部来源", "")];
  for (const source of state.sources) {
    options.push(new Option(`${source.name} (${source.count})`, source.id));
  }
  elements.sourceSelect.replaceChildren(...options);
  elements.sourceSelect.value = current;
}

function renderCategories() {
  const categories = countFilteredCategories();
  const buttons = [];

  const allButton = document.createElement("button");
  allButton.type = "button";
  allButton.className = `chip ${state.category ? "" : "active"}`;
  allButton.textContent = "全部分类";
  allButton.addEventListener("click", () => {
    state.category = "";
    render();
  });
  buttons.push(allButton);

  for (const category of categories) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = `chip ${state.category === category.name ? "active" : ""}`;
    button.textContent = `${getCategoryLabel(category)} ${category.count}`;
    button.addEventListener("click", () => {
      state.category = state.category === category.name ? "" : category.name;
      render();
    });
    buttons.push(button);
  }

  elements.categoryStrip.replaceChildren(...buttons);
}

function renderNotice() {
  if (!state.errors.length) {
    elements.notice.hidden = true;
    elements.notice.textContent = "";
    return;
  }
  elements.notice.hidden = false;
  elements.notice.textContent = state.errors
    .slice(0, 3)
    .map((error) => error.message)
    .join("；");
}

function makeTag(text, className = "") {
  const tag = document.createElement("span");
  tag.className = `tag ${className}`.trim();
  tag.textContent = text;
  return tag;
}

function openDialog(item) {
  state.selectedItem = item;
  elements.dialogTitle.textContent = getDisplayTitle(item);
  elements.dialogMeta.textContent = `${getDisplayCategory(item)} · ${item.sourceName}`;
  elements.dialogPrompt.textContent = item.prompt;
  elements.dialogImage.alt = getDisplayTitle(item);
  if (item.image) {
    elements.dialogImage.hidden = false;
    elements.dialogImage.src = item.image;
  } else {
    elements.dialogImage.hidden = true;
    elements.dialogImage.removeAttribute("src");
  }
  elements.dialogSource.href = item.sourceUrl;
  elements.dialog.showModal();
}

function createCard(item) {
  const article = document.createElement("article");
  article.className = "prompt-card";

  const preview = document.createElement("div");
  preview.className = item.image ? "preview" : "preview missing";

  if (item.image) {
    const image = document.createElement("img");
    image.src = item.image;
    image.alt = item.images?.[0]?.alt || getDisplayTitle(item);
    image.loading = "lazy";
    image.addEventListener("error", () => {
      image.remove();
      preview.classList.add("missing");
      preview.append(document.createTextNode("暂无预览图"));
    });
    preview.append(image);
  } else {
    preview.textContent = "暂无预览图";
  }

  const badge = document.createElement("span");
  badge.className = "source-badge";
  badge.textContent = item.sourceName;
  preview.append(badge);

  const body = document.createElement("div");
  body.className = "card-body";

  const meta = document.createElement("div");
  meta.className = "card-meta";
  meta.append(makeTag(getDisplayCategory(item), "category"));
  if (item.size) {
    meta.append(makeTag(item.size));
  }
  if (item.quality) {
    meta.append(makeTag(item.quality));
  }

  const title = document.createElement("h2");
  title.textContent = getDisplayTitle(item);

  const prompt = document.createElement("p");
  prompt.className = "prompt-preview";
  prompt.textContent = getDisplaySummary(item);

  const actions = document.createElement("div");
  actions.className = "card-actions";

  const previewButton = document.createElement("button");
  previewButton.type = "button";
  previewButton.className = "button button-secondary";
  previewButton.textContent = "查看";
  previewButton.addEventListener("click", () => openDialog(item));

  const copyButton = document.createElement("button");
  copyButton.type = "button";
  copyButton.className = "button";
  copyButton.textContent = "复制";
  copyButton.addEventListener("click", () => copyPrompt(item.prompt));

  actions.append(previewButton, copyButton);
  body.append(meta, title, prompt, actions);
  article.append(preview, body);
  return article;
}

function renderCards(filteredItems) {
  elements.cardsGrid.replaceChildren(...filteredItems.map(createCard));
  elements.emptyState.hidden = filteredItems.length !== 0;
}

function render() {
  renderSourceOptions();
  renderCategories();
  renderNotice();
  const filteredItems = getFilteredItems();
  renderSummary(filteredItems);
  renderCards(filteredItems);
  elements.statusText.textContent = `${state.items.length} 条提示词 · ${state.categories.length} 个分类`;
}

async function loadPrompts({ refresh = false } = {}) {
  elements.refreshButton.disabled = true;
  elements.statusText.textContent = refresh ? "正在刷新 GitHub Markdown..." : "正在载入提示词库...";
  try {
    let data;
    if (checkIsTauri()) {
      data = await tauriInvoke("get_prompts", { refresh });
    } else {
      const response = await fetch(`/api/prompts${refresh ? "?refresh=1" : ""}`);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }
      data = await response.json();
    }
    state.items = data.items || [];
    state.sources = data.sources || [];
    state.categories = data.categories || [];
    state.errors = data.errors || [];
    render();
    elements.statusText.textContent = `${state.items.length} 条提示词 · 更新 ${formatDate(data.updatedAt)}`;
  } catch (error) {
    elements.statusText.textContent = "载入失败";
    elements.notice.hidden = false;
    elements.notice.textContent = `无法载入提示词库：${error.message}`;
  } finally {
    elements.refreshButton.disabled = false;
  }
}

elements.searchInput.addEventListener("input", (event) => {
  state.query = event.target.value;
  render();
});

elements.sourceSelect.addEventListener("change", (event) => {
  state.sourceId = event.target.value;
  state.category = "";
  render();
});

elements.refreshButton.addEventListener("click", () => {
  loadPrompts({ refresh: true });
});

elements.clearButton.addEventListener("click", () => {
  state.query = "";
  state.sourceId = "";
  state.category = "";
  elements.searchInput.value = "";
  render();
});

elements.dialogClose.addEventListener("click", () => {
  elements.dialog.close();
});

elements.dialogCopy.addEventListener("click", () => {
  if (state.selectedItem) {
    copyPrompt(state.selectedItem.prompt);
  }
});

elements.dialog.addEventListener("click", (event) => {
  if (event.target === elements.dialog) {
    elements.dialog.close();
  }
});

// 辅助函数：安全的 Tauri invoke
function tauriInvoke(cmd, args) {
  if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
    return window.__TAURI__.core.invoke(cmd, args);
  } else if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
    return window.__TAURI_INTERNALS__.invoke(cmd, args);
  }
  return Promise.reject(new Error("Tauri IPC not found"));
}

function checkIsTauri() {
  return typeof window.__TAURI__ !== "undefined" || typeof window.__TAURI_INTERNALS__ !== "undefined";
}

// 检测运行环境并标记 body 样式类
function detectEnvironment() {
  const userAgent = navigator.userAgent.toLowerCase();
  const isElectron = userAgent.includes("electron");
  const isTauri = checkIsTauri();

  if (isElectron) {
    document.body.classList.add("is-electron");
  } else if (isTauri) {
    document.body.classList.add("is-tauri");
  }

  if (userAgent.includes("macintosh") || userAgent.includes("mac os x")) {
    document.body.classList.add("is-mac");
  }

  // 如果在 Tauri 环境中，绑定自定义窗口控制事件
  if (isTauri) {
    document.querySelector("#winMinimize")?.addEventListener("click", () => {
      tauriInvoke('minimize_window');
    });
    document.querySelector("#winMaximize")?.addEventListener("click", () => {
      tauriInvoke('maximize_window');
    });
    document.querySelector("#winClose")?.addEventListener("click", () => {
      tauriInvoke('close_window');
    });

    // 监听拖动区域
    document.addEventListener("mousedown", (e) => {
      if (e.target.hasAttribute("data-custom-drag")) {
        // 遇到拖拽区域时，通知 Tauri 开始原生拖拽
        tauriInvoke('start_dragging').catch(err => {
            console.error("Drag failed:", err);
        });
      }
    });
  }
}

// 在加载数据前执行环境检测（避免脚本加载时机问题导致丢失全局变量）
setTimeout(() => {
  detectEnvironment();
  loadPrompts();
}, 100);
