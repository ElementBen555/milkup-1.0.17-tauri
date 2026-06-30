/**
 * Tauri <-> ElectronAPI 兼容桥接层
 *
 * 将原有 window.electronAPI.* 调用透明映射到 Tauri invoke() / listen()
 * 前端代码零改动即可在 Tauri 环境中运行
 *
 * 用法：在 main.ts 中最顶部导入此文件
 *   import "./electronApiCompat";
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// ─── 全局错误展示（Tauri release 模式下无 devtools，错误打到 DOM 上） ──
const errorBox = typeof document !== "undefined" ? document.createElement("div") : null;
if (errorBox) {
  errorBox.id = "__tauri_error_box";
  errorBox.style.cssText =
    "position:fixed;z-index:99999;bottom:8px;left:8px;right:8px;max-height:40vh;overflow:auto;" +
    "background:#1a1a2e;color:#f7768e;padding:10px 14px;border-radius:8px;" +
    "font:12px/1.6 monospace;white-space:pre-wrap;display:none;" +
    "box-shadow:0 8px 24px rgba(0,0,0,.5);border:1px solid #f7768e";
  document.addEventListener("DOMContentLoaded", () => document.body?.append(errorBox));
}

let errorCount = 0;
function showError(source: string, e: unknown): void {
  errorCount++;
  const msg = `[${errorCount}] ${source}: ${String(e)}\n`;
  console.error(msg);
  if (errorBox) {
    errorBox.style.display = "block";
    errorBox.textContent += msg;
    errorBox.scrollTop = errorBox.scrollHeight;
  }
}

// 捕获全局未处理异常
window.addEventListener("error", (e) => showError("window.error", e.error || e.message));
window.addEventListener("unhandledrejection", (e) => showError("unhandledrejection", e.reason));

// ── 拖拽文件路径缓存（Tauri 中 File 对象无 .path，用 file-name → abs-path 映射） ──
const _droppedPathCache = new Map<string, string>();
listen("file-dropped", async (event: { payload: unknown }) => {
  const paths = event.payload as string[];
  for (const absPath of paths) {
    const name = absPath.split(/[/\\]/).pop() || absPath;
    _droppedPathCache.set(name, absPath);
  }
  for (const absPath of paths) {
    try {
      const result = await invoke("read_file_by_path", { filePath: absPath });
      if (result) {
        window.dispatchEvent(new CustomEvent("tauri:file-dropped", { detail: result }));
      }
    } catch { /* ignore individual file read errors */ }
  }
});

/** 安全 invoke：出错时显示到 DOM 错误面板 */
function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(cmd, args).catch((e: unknown) => {
    showError(`invoke:${cmd}`, e);
    throw e;
  });
}

/** 事件监听映射：cleanup 函数按 channel 分组 */
const listenerMap = new Map<Function, Map<string, UnlistenFn[]>>();

/** 安全设置 window 属性 */
function setApi<K extends keyof typeof electronAPI>(key: K, value: (typeof electronAPI)[K]): void {
  (window as Record<string, unknown>).__electronAPI = undefined;
}

// ─── 兼容 API 对象 ────────────────────────────────────────

const electronAPI = {
  // ── 文件 I/O ──
  openFile: (): Promise<{ filePath: string; content: string; fileTraits?: unknown } | null> =>
    safeInvoke("open_file"),
  getIsReadOnly: (filePath: string): Promise<boolean> => safeInvoke("is_read_only", { filePath }),
  saveFile: (
    filePath: string | null,
    content: string,
    fileTraits?: unknown,
    imageLocalPath?: string,
    imageUseFileNameFolder?: boolean
  ): Promise<{ filePath: string; content: string } | null> =>
    safeInvoke("save_file", { params: { filePath, content, fileTraits, imageLocalPath, imageUseFileNameFolder } }),
  saveFileAs: (
    content: string,
    fileTraits?: unknown,
    imageLocalPath?: string,
    imageUseFileNameFolder?: boolean
  ): Promise<{ filePath: string; content: string } | null> =>
    safeInvoke("save_file_as", { params: { content, fileTraits, imageLocalPath, imageUseFileNameFolder } }),
  readFileByPath: (filePath: string): Promise<unknown> => safeInvoke("read_file_by_path", { filePath }),
  cleanupLocalImages: (content: string): Promise<void> =>
    safeInvoke("cleanup_local_images", { content }),

  // ── 事件监听 ──
  on: (channel: string, listener: (...args: unknown[]) => void): void => {
    // Tauri 用 `-` 替代 `:`，小写命名
    const eventName = channel.replace(/:/g, "-").toLowerCase();
    listen(eventName, (event: { payload: unknown }) => {
      listener(event.payload);
    }).then((unlisten) => {
      if (!listenerMap.has(listener)) listenerMap.set(listener, new Map());
      const chMap = listenerMap.get(listener)!;
      if (!chMap.has(channel)) chMap.set(channel, []);
      chMap.get(channel)!.push(unlisten);
    });
  },
  removeListener: (channel: string, listener: (...args: unknown[]) => void): void => {
    const chMap = listenerMap.get(listener);
    if (!chMap) return;
    const unlisteners = chMap.get(channel);
    if (unlisteners) {
      for (const u of unlisteners) u();
      chMap.delete(channel);
    }
    if (chMap.size === 0) listenerMap.delete(listener);
  },

  // ── 窗口控制 ──
  setTitle: (filePath: string | null): void => {
    const title = filePath
      ? `milkup - ${filePath.split(/[/\\]/).pop() || "Untitled"}`
      : "milkup - Untitled";
    document.title = title;
    invoke("set_title", { title }).catch(() => {});
  },
  changeSaveStatus: (_isSaved: boolean): void => {
    /* Tauri 中由文档状态自然管理 */
  },
  windowControl: (action: "minimize" | "maximize" | "close"): Promise<void> =>
    safeInvoke("window_control", { action }),
  closeDiscard: (): Promise<void> => safeInvoke("close_discard"),
  getWindowBounds: (): Promise<{ x: number; y: number; width: number; height: number }> =>
    safeInvoke("get_window_bounds"),

  // ── 打开文件（启动时/拖入） ──
  onOpenFileAtLaunch: (cb: (payload: { filePath: string; content: string; fileTraits?: unknown }) => void): void => {
    // Tauri 中通过 open-file-at-launch 事件传递（需后端发送）
    listen("open-file-at-launch", (event: { payload: unknown }) => {
      cb(event.payload as { filePath: string; content: string; fileTraits?: unknown });
    });
  },

  // ── Shell ──
  openExternal: (url: string): Promise<void> => safeInvoke("open_external", { url }),
  openLink: (href: string, currentFilePath?: string | null): Promise<void> =>
    safeInvoke("open_link", { href, currentFilePath }),

  // ── 图片预览 ──
  openImagePreview: (src: string, alt?: string): Promise<void> =>
    safeInvoke("open_image_preview", { src, alt }),

  // ── 剪贴板 ──
  getFilePathInClipboard: (): Promise<string | null> => safeInvoke("get_clipboard_file_path"),
  writeTempImage: (
    file: Uint8Array,
    targetPath: string,
    currentFilePath?: string | null,
    fileName?: string,
    mimeType?: string,
    useFileNameFolder?: boolean
  ): Promise<string> =>
    safeInvoke("write_temp_image", { params: {
      file: Array.from(file), // Tauri IPC 需要普通数组而非 TypedArray
      targetPath,
      currentFilePath,
      fileName,
      mimeType,
      useFileNameFolder,
    } }),
  showImageUnsavedChoice: (): Promise<string> => safeInvoke("show_image_unsaved_choice"),
  showOverwriteConfirm: (fileName: string): Promise<string> => safeInvoke("show_overwrite_confirm", { fileName }),

  // ── 导出 ──
  exportAsPDF: (_selector: string, _outputName: string, _options?: unknown): Promise<void> =>
    safeInvoke("export_pdf"),
  exportAsWord: (_blocks: unknown, _outputName: string): Promise<void> =>
    Promise.resolve(), // TODO: Rust 实现

  // ── 字体 ──
  getSystemFonts: (): Promise<string[]> => safeInvoke("get_system_fonts"),

  // ── 工作区 ──
  getDirectoryFiles: (dirPath: string): Promise<unknown[]> => safeInvoke("get_directory_files", { dirPath }),
  workspaceExists: (dirPath: string): Promise<boolean> => safeInvoke("workspace_exists", { dirPath }),
  watchFiles: (filePaths: string[]): void => {
    safeInvoke("watch_files", { filePaths }).catch(() => {});
  },
  watchDirectory: (dirPath: string): void => {
    safeInvoke("watch_directory", { dirPath }).catch(() => {});
  },
  unwatchDirectory: (): void => {
    safeInvoke("unwatch_directory").catch(() => {});
  },
  createFile: (dirPath: string, fileName: string): Promise<string> =>
    safeInvoke("create_file", { params: { dirPath, fileName } }),
  createFolder: (dirPath: string, folderName: string): Promise<string> =>
    safeInvoke("create_folder", { params: { dirPath, folderName } }),
  deleteFile: (filePath: string): Promise<void> => safeInvoke("delete_file", { filePath }),
  renameFile: (oldPath: string, newName: string): Promise<string> =>
    safeInvoke("rename_file", { params: { oldPath, newName } }),

  // ── 主题编辑器 ──
  openThemeEditor: (_theme?: unknown): void => {
    safeInvoke("open_theme_editor").catch(() => {});
  },
  themeEditorWindowControl: (_action: string): void => {
    /* TODO: 新窗口 API */
  },
  saveCustomTheme: (_theme: unknown): void => {
    /* TODO: 持久化 */
  },

  // ── 平台 ──
  platform: (() => {
    if (typeof navigator === "undefined") return "linux";
    const p = navigator.platform.toLowerCase();
    if (p.includes("mac")) return "darwin";
    if (p.includes("win")) return "win32";
    return "linux";
  })(),

  // ── 渲染进程就绪 ──
  rendererReady: (): void => {
    /* Tauri 不需要显式通知 */
  },

  // ── Tab 拖拽分离（暂存根） ──
  tearOffTabStart: (tabData: unknown, screenX: number, screenY: number): Promise<string> =>
    safeInvoke("tear_off_tab", { tabData, x: screenX, y: screenY }),
  tearOffTabEnd: (_screenX: number, _screenY: number): Promise<{ action: string }> =>
    Promise.resolve({ action: "created" }),
  tearOffTabCancel: (): Promise<void> => Promise.resolve(),
  focusFileIfOpen: (_filePath: string): Promise<{ found: boolean }> =>
    Promise.resolve({ found: false }),
  getInitialTabData: (): Promise<unknown> => Promise.resolve(null),
  startWindowDrag: (..._args: unknown[]): void => {},
  stopWindowDrag: (): void => {},
  dropMerge: (..._args: unknown[]): Promise<{ action: string }> =>
    Promise.resolve({ action: "failed" }),

  // ── 自动更新（暂无后端，待后续实现） ──
  checkForUpdates: (): Promise<void> => Promise.resolve(),
  downloadUpdate: (): Promise<void> => Promise.resolve(),
  cancelUpdate: (): Promise<void> => Promise.resolve(),
  quitAndInstall: (): Promise<void> => safeInvoke("quit_and_install"),
  onUpdateStatus: (_cb: (status: unknown) => void): void => {},
  onDownloadProgress: (_cb: (progress: unknown) => void): void => {},

  // ── 显示打开对话框 ──
  showOpenDialog: (options: unknown): Promise<unknown> =>
    safeInvoke("show_open_dialog", { options }),
  showCloseConfirm: (_fileName: string): Promise<unknown> => Promise.resolve(null),

  // ── 拖拽文件路径（Tauri 中通过 file-dropped 事件获取） ──
  getPathForFile: (file: File): string | undefined => {
    // 优先用浏览器原生的 file.path（WebKit 可能提供）
    if ((file as any).path) return (file as any).path;
    // 回退到 Tauri 拖放缓存（file-dropped 事件预填充）
    return _droppedPathCache.get(file.name);
  },

  // ── 新增强功能 ──
  exportHtml: (content: string, title?: string): Promise<string> =>
    safeInvoke("export_html", { content, title }),
  generateToc: (content: string): Promise<string> =>
    safeInvoke("generate_toc", { content }),
  exportEpub: (content: string, title?: string): Promise<string> =>
    safeInvoke("export_epub", { content, title }),
  gitDiff: (filePath: string): Promise<string> =>
    safeInvoke("git_diff", { filePath }),
  checkLinks: (content: string): Promise<string[]> =>
    safeInvoke("check_links", { content }),
  searchFiles: (dirPath: string, query: string, caseSensitive?: boolean): Promise<unknown[]> =>
    safeInvoke("search_files", { params: { dirPath, query, caseSensitive } }),

  // 聚焦模式切换
  isFocusMode: false,
  toggleFocusMode: function(this: { isFocusMode: boolean }) {
    this.isFocusMode = !this.isFocusMode;
    document.body.classList.toggle("focus-mode", this.isFocusMode);
  },

  // 演示模式
  isPresentationMode: false,
  togglePresentationMode: function(this: { isPresentationMode: boolean }) {
    this.isPresentationMode = !this.isPresentationMode;
    document.body.classList.toggle("presentation-mode", this.isPresentationMode);
  },

  // 深色模式跟随系统
  watchSystemTheme: () => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = (e: MediaQueryListEvent | MediaQueryList) => {
      const dark = e.matches;
      document.documentElement.classList.toggle("system-dark", dark);
      // 如果用户没手动设主题，自动切换
      const manual = localStorage.getItem("theme-name");
      if (!manual || manual === "system") {
        document.documentElement.classList.toggle("dark", dark);
      }
    };
    apply(mq);
    mq.addEventListener("change", apply as (e: MediaQueryListEvent) => void);
  },

  // 自定义 CSS
  applyCustomCSS: (css: string) => {
    let el = document.getElementById("milkup-custom-css") as HTMLStyleElement | null;
    if (!el) {
      el = document.createElement("style");
      el.id = "milkup-custom-css";
      document.head.appendChild(el);
    }
    el.textContent = css;
  },
};

// ─── 注入全局 ─────────────────────────────────────────────

if (typeof window !== "undefined") {
  (window as Record<string, unknown>).electronAPI = electronAPI;
  // 自动激活深色模式跟随 + 加载自定义 CSS
  setTimeout(() => {
    electronAPI.watchSystemTheme();
    const customCSS = localStorage.getItem("milkup-custom-css");
    if (customCSS) electronAPI.applyCustomCSS(customCSS);
  }, 100);
}

export { electronAPI };
