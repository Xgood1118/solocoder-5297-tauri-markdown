import { invoke } from '@tauri-apps/api/core'
import type { SearchResult } from '@/types'

export async function searchFiles(dirPath: string, pattern: string): Promise<string[]> {
  return invoke<string[]>('search_files', { dirPath, pattern })
}

export async function searchContent(
  dirPath: string,
  pattern: string,
  caseSensitive = false,
): Promise<SearchResult[]> {
  return invoke<SearchResult[]>('search_content', {
    dirPath,
    pattern,
    caseSensitive,
  })
}
