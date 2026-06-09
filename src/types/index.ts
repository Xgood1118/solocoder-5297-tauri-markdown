export interface Document {
  id: string
  path: string
  title: string
  content: string
  created_at: string
  updated_at: string
  is_dirty: boolean
  is_encrypted: boolean
  size_bytes: number
}

export interface FileInfo {
  path: string
  name: string
  is_dir: boolean
  size_bytes: number
  created_at: string
  modified_at: string
}

export interface Snapshot {
  id: string
  file_path: string
  content_hash: string
  content: string
  created_at: string
  size_bytes: number
}

export interface SearchResult {
  file_path: string
  line_number: number
  column_start: number
  column_end: number
  line_content: string
  match_text: string
}

export interface OutlineItem {
  level: number
  text: string
  anchor: string
  line: number
}

export type ThemeName = 'light' | 'dark' | 'sepia' | 'solarized'
export type FontFamily = 'system' | 'serif' | 'mono' | 'sans'

export interface ThemeConfig {
  name: ThemeName
  bg_color: string
  text_color: string
  accent_color: string
  font_family: FontFamily
  font_size: number
}

export interface SyncConfig {
  provider: SyncProvider
  endpoint: string
  username: string
  password: string
  bucket?: string
  region?: string
  remote_path: string
  auto_sync: boolean
  sync_interval_secs: number
}

export type SyncProvider = 'webdav' | 's3' | 'disabled'

export interface RemoteFile {
  path: string
  name: string
  size_bytes: number
  modified_at: string
  is_dir: boolean
}

export interface PluginInfo {
  id: string
  name: string
  version: string
  description: string
  author: string
  plugin_type: PluginType
  enabled: boolean
}

export type PluginType = 'command' | 'renderer' | 'theme'
