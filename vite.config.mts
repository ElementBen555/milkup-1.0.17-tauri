import path from 'node:path'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'
import vitePluginsAutoI18n, { EmptyTranslator } from 'vite-auto-i18n-plugin'

const alias = {
  '@': path.resolve(__dirname, './src'),
  '@renderer': path.resolve(__dirname, './src/renderer'),
  '@ui': path.resolve(__dirname, './src/renderer/components/ui'),
}

const i18nPlugin = vitePluginsAutoI18n({
  deepScan: true,
  globalPath: './lang',
  namespace: 'lang',
  distPath: './dist/assets',
  distKey: 'index',
  targetLangList: ['ja', 'ko', 'ru', 'en', 'fr'],
  originLang: 'zh-cn',
  translator: new EmptyTranslator(),
})

// ─── 手动代码分割 ────────────────────────────────────────
function manualChunks(id: string): string | undefined {
  if (id.includes('node_modules/mermaid')) return 'vendor-mermaid'
  if (id.includes('node_modules/katex')) return 'vendor-katex'
  if (id.includes('node_modules/prosemirror-')) return 'vendor-prosemirror'
  if (
    id.includes('node_modules/@codemirror/state') ||
    id.includes('node_modules/@codemirror/view') ||
    id.includes('node_modules/@codemirror/language') ||
    id.includes('node_modules/@codemirror/commands') ||
    id.includes('node_modules/@codemirror/autocomplete') ||
    id.includes('node_modules/@codemirror/search') ||
    id.includes('node_modules/@codemirror/lint') ||
    id.includes('node_modules/codemirror')
  ) return 'vendor-codemirror'
  if (id.includes('node_modules/@codemirror/lang-') || id.includes('node_modules/@codemirror/legacy-modes'))
    return 'vendor-codemirror-langs'
  if (
    id.includes('node_modules/unified') ||
    id.includes('node_modules/remark-') ||
    id.includes('node_modules/rehype-') ||
    id.includes('node_modules/mdast') ||
    id.includes('node_modules/unist')
  ) return 'vendor-unified'
  if (id.includes('node_modules/vue') || id.includes('node_modules/@vue'))
    return 'vendor-vue'
  return undefined
}

export default defineConfig({
  // Electron 插件已移除，改由 Tauri 管理
  plugins: [vue(), i18nPlugin],
  server: {
    port: 5173,
    strictPort: true,
  },
  root: 'src/renderer',
  base: './',
  build: {
    outDir: '../../dist',
    emptyOutDir: true,
    target: 'es2020',
    cssCodeSplit: false,
    assetsInlineLimit: 4096,
    rollupOptions: {
      input: {
        'main': path.resolve(__dirname, 'src/renderer/index.html'),
        'theme-editor': path.resolve(__dirname, 'src/renderer/theme-editor.html'),
      },
      output: { manualChunks },
    },
  },
  resolve: { alias },
  // 防止 Tauri 中 HMR 问题
  clearScreen: false,
})
