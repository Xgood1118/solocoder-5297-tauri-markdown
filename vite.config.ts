import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  clearScreen: false,
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: 'es2021',
    minify: 'esbuild',
    sourcemap: false,
    chunkSizeWarningLimit: 1500,
    rollupOptions: {
      output: {
        manualChunks: {
          codemirror: [
            '@codemirror/state',
            '@codemirror/view',
            '@codemirror/commands',
            '@codemirror/language',
            '@codemirror/search',
            '@codemirror/autocomplete',
            '@lezer/highlight',
            '@lezer/markdown',
          ],
          markdown: [
            'markdown-it',
            'markdown-it-anchor',
            'markdown-it-highlightjs',
            'markdown-it-task-lists',
            'markdown-it-toc-done-right',
            'highlight.js',
          ],
          katex: ['katex'],
          mermaid: ['mermaid'],
        },
      },
    },
  },
})
