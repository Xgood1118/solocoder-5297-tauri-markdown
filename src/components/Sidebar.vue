<template>
  <div class="sidebar">
    <div class="sidebar-tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="['sidebar-tab', { active: activeTab === tab.id }]"
        @click="activeTab = tab.id"
        :title="tab.label"
      >
        <component :is="tab.icon" />
      </button>
    </div>

    <div class="sidebar-content">
      <div v-show="activeTab === 'files'" class="panel">
        <div class="panel-header">
          <span class="panel-title">{{ t('sidebar.files') }}</span>
          <div class="panel-actions">
            <button class="icon-btn" @click="refreshFiles" title="Refresh">
              ↻
            </button>
            <button class="icon-btn" @click="openFolder" title="Open folder">
              📁
            </button>
          </div>
        </div>
        <div class="panel-body file-list">
          <div v-if="!currentFolder" class="empty-state">
            <p>No folder selected</p>
            <button class="primary-btn" @click="openFolder">
              Open Folder
            </button>
          </div>
          <template v-else>
            <div class="current-folder" @click="goUp">
              <span class="folder-icon">📂</span>
              <span class="folder-name" :title="currentFolder">
                {{ folderDisplayName }}
              </span>
            </div>
            <div class="file-tree">
              <div
                v-for="file in files"
                :key="file.path"
                :class="['file-item', { dir: file.is_dir }]"
                @click="handleFileClick(file)"
                @dblclick="handleFileDblClick(file)"
              >
                <span class="file-icon">
                  {{ file.is_dir ? '📁' : '📄' }}
                </span>
                <span class="file-name" :title="file.name">
                  {{ file.name }}
                </span>
              </div>
            </div>
          </template>
        </div>
      </div>

      <div v-show="activeTab === 'search'" class="panel">
        <div class="panel-header">
          <span class="panel-title">{{ t('sidebar.search') }}</span>
        </div>
        <div class="panel-body">
          <div class="search-input-wrap">
            <input
              v-model="searchQuery"
              type="text"
              :placeholder="t('search.placeholder')"
              @input="handleSearch"
              class="search-input"
            />
          </div>
          <div class="search-results">
            <div v-if="searching" class="searching">Searching...</div>
            <div v-else-if="searchResults.length === 0 && searchQuery" class="empty-state">
              {{ t('search.no_results') }}
            </div>
            <div
              v-for="(result, index) in searchResults"
              :key="index"
              class="search-result-item"
              @click="handleSearchResultClick(result)"
            >
              <div class="result-file">{{ result.file_path }}</div>
              <div class="result-line">
                <span class="line-num">{{ result.line_number }}</span>
                <span class="line-content">{{ result.line_content }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div v-show="activeTab === 'settings'" class="panel">
        <div class="panel-header">
          <span class="panel-title">{{ t('sidebar.settings') }}</span>
        </div>
        <div class="panel-body settings-panel">
          <div class="settings-group">
            <div class="settings-label">{{ t('view.theme') }}</div>
            <div class="theme-options">
              <button
                v-for="theme in themeOptions"
                :key="theme"
                :class="['theme-btn', { active: currentTheme === theme }]"
                @click="setTheme(theme)"
              >
                {{ t(`theme.${theme}`) }}
              </button>
            </div>
          </div>

          <div class="settings-group">
            <div class="settings-label">{{ t('view.font') }}</div>
            <div class="font-options">
              <button
                v-for="font in fontOptions"
                :key="font"
                :class="['font-btn', { active: currentFont === font }]"
                @click="setFont(font)"
              >
                {{ t(`font.${font}`) }}
              </button>
            </div>
          </div>

          <div class="settings-group">
            <div class="settings-label">
              {{ t('view.font_size') }}: {{ fontSize }}px
            </div>
            <input
              type="range"
              :value="fontSize"
              min="10"
              max="32"
              @input="handleFontSizeChange"
              class="font-size-slider"
            />
          </div>

          <div class="settings-group">
            <div class="settings-label">{{ t('sync.title') }}</div>
            <div class="sync-settings">
              <select :value="syncProvider" @change="handleSyncProviderChange" class="select-input">
                <option value="disabled">{{ t('sync.disabled') }}</option>
                <option value="webdav">{{ t('sync.webdav') }}</option>
                <option value="s3">{{ t('sync.s3') }}</option>
              </select>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import type { FileInfo, SearchResult, ThemeName, FontFamily } from '@/types'
import { useThemeStore } from '@/store/theme'
import { useSyncStore } from '@/store/sync'
import { searchContent } from '@/services/search'

const emit = defineEmits<{
  (e: 'openFile', path: string): void
  (e: 'searchResult', path: string, line: number): void
}>()

const { t } = useI18n()
const themeStore = useThemeStore()
const syncStore = useSyncStore()

const tabs = [
  { id: 'files', label: 'Files', icon: '📁' },
  { id: 'search', label: 'Search', icon: '🔍' },
  { id: 'settings', label: 'Settings', icon: '⚙️' },
]

const activeTab = ref('files')
const files = ref<FileInfo[]>([])
const currentFolder = ref<string>('')
const searchQuery = ref('')
const searchResults = ref<SearchResult[]>([])
const searching = ref(false)
let searchTimer: number | null = null

const themeOptions: ThemeName[] = ['light', 'dark', 'sepia', 'solarized']
const fontOptions: FontFamily[] = ['system', 'serif', 'mono', 'sans']

const currentTheme = computed(() => themeStore.theme)
const currentFont = computed(() => themeStore.fontFamily)
const fontSize = computed(() => themeStore.fontSize)
const syncProvider = computed(() => syncStore.config.provider)

const folderDisplayName = computed(() => {
  if (!currentFolder.value) return ''
  const parts = currentFolder.value.split(/[/\\]/)
  return parts[parts.length - 1] || currentFolder.value
})

function setTheme(theme: ThemeName) {
  themeStore.setTheme(theme)
}

function setFont(font: FontFamily) {
  themeStore.setFontFamily(font)
}

function handleFontSizeChange(event: Event) {
  const target = event.target as HTMLInputElement
  themeStore.setFontSize(parseInt(target.value, 10))
}

function handleSyncProviderChange(event: Event) {
  const target = event.target as HTMLSelectElement
  syncStore.setProvider(target.value as any)
  syncStore.saveConfig()
}

async function openFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
  })

  if (selected && typeof selected === 'string') {
    currentFolder.value = selected
    await loadFiles()
  }
}

async function loadFiles() {
  if (!currentFolder.value) return

  try {
    const result = await invoke<FileInfo[]>('list_files', {
      dirPath: currentFolder.value,
      recursive: false,
    })
    files.value = result
  } catch (e) {
    console.error('Failed to list files:', e)
  }
}

function refreshFiles() {
  loadFiles()
}

async function goUp() {
  if (!currentFolder.value) return

  const parts = currentFolder.value.split(/[/\\]/)
  if (parts.length <= 1) return

  parts.pop()
  currentFolder.value = parts.join('/') || '/'
  await loadFiles()
}

function handleFileClick(file: FileInfo) {
  // 单击选中
}

function handleFileDblClick(file: FileInfo) {
  if (file.is_dir) {
    currentFolder.value = file.path
    loadFiles()
  } else {
    emit('openFile', file.path)
  }
}

function handleSearch() {
  if (searchTimer) {
    clearTimeout(searchTimer)
  }

  if (!searchQuery.value.trim()) {
    searchResults.value = []
    return
  }

  searchTimer = window.setTimeout(async () => {
    if (!currentFolder.value) {
      searchResults.value = []
      return
    }

    searching.value = true
    try {
      const results = await searchContent(currentFolder.value, searchQuery.value)
      searchResults.value = results.slice(0, 100)
    } catch (e) {
      console.error('Search failed:', e)
      searchResults.value = []
    } finally {
      searching.value = false
    }
  }, 300)
}

function handleSearchResultClick(result: SearchResult) {
  emit('searchResult', result.file_path, result.line_number)
}

onMounted(() => {
  syncStore.loadConfig()
})
</script>

<style scoped>
.sidebar {
  display: flex;
  flex-direction: row;
  height: 100%;
  width: 100%;
  background: var(--bg-secondary);
}

.sidebar-tabs {
  display: flex;
  flex-direction: column;
  width: 48px;
  border-right: 1px solid var(--border-color);
  background: var(--bg-tertiary);
  flex-shrink: 0;
}

.sidebar-tab {
  width: 48px;
  height: 48px;
  border: none;
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  color: var(--text-secondary);
  transition: background 0.15s, color 0.15s;
  position: relative;
}

.sidebar-tab:hover {
  background: var(--hover-color);
}

.sidebar-tab.active {
  color: var(--accent-color);
}

.sidebar-tab.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 8px;
  bottom: 8px;
  width: 2px;
  background: var(--accent-color);
}

.sidebar-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.panel-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.panel-actions {
  display: flex;
  gap: 4px;
}

.icon-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  cursor: pointer;
  border-radius: 4px;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s;
}

.icon-btn:hover {
  background: var(--hover-color);
}

.panel-body {
  flex: 1;
  overflow-y: auto;
}

.file-list {
  display: flex;
  flex-direction: column;
}

.current-folder {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  cursor: pointer;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-color);
  gap: 8px;
}

.current-folder:hover {
  background: var(--hover-color);
}

.folder-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.folder-name {
  font-size: 13px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-tree {
  display: flex;
  flex-direction: column;
}

.file-item {
  display: flex;
  align-items: center;
  padding: 6px 16px;
  cursor: pointer;
  gap: 8px;
  transition: background 0.15s;
}

.file-item:hover {
  background: var(--hover-color);
}

.file-icon {
  font-size: 14px;
  flex-shrink: 0;
}

.file-name {
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--text-muted);
  text-align: center;
  gap: 12px;
}

.empty-state p {
  margin: 0;
  font-size: 13px;
}

.primary-btn {
  padding: 8px 16px;
  background: var(--accent-color);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  transition: opacity 0.15s;
}

.primary-btn:hover {
  opacity: 0.9;
}

.search-input-wrap {
  padding: 8px 16px;
  border-bottom: 1px solid var(--border-color);
}

.search-input {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-color);
  color: var(--text-color);
  font-size: 13px;
  box-sizing: border-box;
}

.search-input:focus {
  outline: none;
  border-color: var(--accent-color);
}

.search-results {
  flex: 1;
  overflow-y: auto;
}

.searching {
  padding: 16px;
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}

.search-result-item {
  padding: 8px 16px;
  cursor: pointer;
  border-bottom: 1px solid var(--border-color);
  transition: background 0.15s;
}

.search-result-item:hover {
  background: var(--hover-color);
}

.result-file {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.result-line {
  display: flex;
  gap: 8px;
  font-size: 13px;
  font-family: monospace;
}

.line-num {
  color: var(--text-muted);
  flex-shrink: 0;
  min-width: 24px;
  text-align: right;
}

.line-content {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.settings-panel {
  padding: 16px;
}

.settings-group {
  margin-bottom: 24px;
}

.settings-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-color);
  margin-bottom: 8px;
}

.theme-options,
.font-options {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.theme-btn,
.font-btn {
  padding: 6px 12px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-color);
  color: var(--text-color);
  cursor: pointer;
  font-size: 12px;
  transition: border-color 0.15s, background 0.15s;
}

.theme-btn:hover,
.font-btn:hover {
  border-color: var(--accent-color);
}

.theme-btn.active,
.font-btn.active {
  border-color: var(--accent-color);
  background: var(--active-color);
  color: var(--accent-color);
}

.font-size-slider {
  width: 100%;
  cursor: pointer;
}

.select-input {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background: var(--bg-color);
  color: var(--text-color);
  font-size: 13px;
}

.select-input:focus {
  outline: none;
  border-color: var(--accent-color);
}
</style>
