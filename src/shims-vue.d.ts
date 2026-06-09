declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

declare module 'markdown-it-katex' {
  import type MarkdownIt from 'markdown-it'
  const plugin: (md: MarkdownIt) => void
  export default plugin
}

declare module 'markdown-it-task-lists' {
  import type MarkdownIt from 'markdown-it'
  const plugin: (md: MarkdownIt, options?: any) => void
  export default plugin
}

declare module 'markdown-it-toc-done-right' {
  import type MarkdownIt from 'markdown-it'
  const plugin: (md: MarkdownIt, options?: any) => void
  export default plugin
}

declare module '@tauri-apps/plugin-dialog' {
  export function open(options?: any): Promise<string | string[] | null>
  export function save(options?: any): Promise<string | null>
  export function message(message: string, options?: any): Promise<void>
  export function ask(message: string, options?: any): Promise<boolean>
  export function confirm(message: string, options?: any): Promise<boolean>
}

declare module '@tauri-apps/plugin-fs' {
  // 文件系统插件类型
}

declare module '@tauri-apps/plugin-shell' {
  // Shell 插件类型
}

declare module '@tauri-apps/api/core' {
  export function invoke<T>(cmd: string, args?: Record<string, any>): Promise<T>
  export function convertFileSrc(filePath: string, protocol?: string): string
}
