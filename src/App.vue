<template>
  <div class="app-container" :class="{ 'sidebar-hidden': !showSidebar }">
    <TitleBar
      :title="windowTitle"
      @new="handleNew"
      @open="handleOpen"
      @save="handleSave"
      @save-as="handleSaveAs"
      @toggle-sidebar="toggleSidebar"
      @toggle-outline="toggleOutline"
      @toggle-preview="togglePreview"
    />

    <div class="main-content">
      <div v-if="showSidebar" class="sidebar-wrap">
        <Sidebar
          @open-file="handleOpenFile"
          @search-result="handleSearchResult"
        />
      </div>

      <div class="editor-area">
        <TabBar
          :docs="docList"
          :active-doc-id="activeDocId"
          @select="handleTabSelect"
          @close="handleTabClose"
          @new-tab="handleNew"
        />

        <div class="editor-preview-container">
          <div class="editor-pane" v-if="activeDoc">
            <Editor
              ref="editorRef"
              v-model="editorContent"
              :theme="theme"
              :font-size="fontSize"
              :font-family="editorFontFamily"
              @cursor-change="handleCursorChange"
              @scroll="handleEditorScroll"
            />
          </div>
          <div v-else class="empty-state-wrap">
            <div class="empty-state">
              <div class="empty-icon">📝</div>
              <h2>{{ t('app.title') }}</h2>
              <p>Start writing or open a file</p>
              <button class="primary-btn" @click="handleNew">
                {{ t('menu.new') }}
              </button>
            </div>
          </div>

          <div v-if="showPreview && activeDoc" class="preview-pane">
            <Preview
              ref="previewRef"
              :content="editorContent"
              :theme="theme"
              :font-size="fontSize"
              :font-family="previewFontFamily"
              @outline-change="handleOutlineChange"
            />
          </div>

          <div v-if="showOutline && activeDoc" class="outline-pane">
            <Outline :items="outline" @navigate="handleOutlineNavigate" />
          </div>
        </div>

        <StatusBar
          :cursor-line="cursorLine"
          :cursor-column="cursorColumn"
          :word-count="wordCount"
          :char-count="charCount"
          :read-time="readingTimeText"
          :is-dirty="activeDoc?.is_dirty"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import TitleBar from '@/components/TitleBar.vue'
import TabBar from '@/components/TabBar.vue'
import Editor from '@/components/Editor.vue'
import Preview from '@/components/Preview.vue'
import Outline from '@/components/Outline.vue'
import Sidebar from '@/components/Sidebar.vue'
import StatusBar from '@/components/StatusBar.vue'
import { useDocsStore } from '@/store/docs'
import { useThemeStore } from '@/store/theme'
import { useEditorStore } from '@/store/editor'
import type { Document, OutlineItem } from '@/types'
import { handleImageDrop, handleImagePaste } from '@/services/imageUploader'
import { createSnapshot } from '@/services/snapshot'

const { t } = useI18n()
const docsStore = useDocsStore()
const themeStore = useThemeStore()
const editorStore = useEditorStore()

const editorRef = ref<InstanceType<typeof Editor> | null>(null)
const previewRef = ref<InstanceType<typeof Preview> | null>(null)

const editorContent = ref('')
const cursorLine = ref(1)
const cursorColumn = ref(1)

const docList = computed(() => docsStore.docList)
const activeDocId = computed(() => docsStore.activeDocId)
const activeDoc = computed(() => docsStore.activeDoc)
const showSidebar = computed(() => themeStore.showSidebar)
const showPreview = computed(() => themeStore.showPreview)
const showOutline = computed(() => themeStore.showOutline)
const theme = computed(() => themeStore.theme)
const fontSize = computed(() => themeStore.fontSize)
const outline = computed(() => editorStore.outline)
const wordCount = computed(() => editorStore.wordCount)
const charCount = computed(() => editorStore.charCount)
const readingTimeText = computed(() => editorStore.readingTimeText)

const editorFontFamily = computed(() => {
  if (themeStore.fontFamily === 'mono') return themeStore.currentFontFamily
  return '"SF Mono", Menlo, Monaco, Consolas, "Courier New", monospace'
})

const previewFontFamily = computed(() => themeStore.currentFontFamily)

const windowTitle = computed(() => {
  if (activeDoc.value) {
    const dirty = activeDoc.value.is_dirty ? ' • ' : ''
    return `${activeDoc.value.title}${dirty}- ${t('app.title')}`
  }
  return t('app.title')
})

watch(
  () => activeDoc.value?.content,
  (newContent) => {
    if (newContent !== undefined && newContent !== editorContent.value) {
      editorContent.value = newContent
    }
  },
  { immediate: true },
)

watch(editorContent, (newContent) => {
  if (activeDocId.value) {
    docsStore.updateDocContent(activeDocId.value, newContent)
    editorStore.updateStats(newContent)
  }
})

function handleNew() {
  const newDoc: Document = {
    id: crypto.randomUUID(),
    path: '',
    title: t('app.untitled'),
    content: '',
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    is_dirty: false,
    is_encrypted: false,
    size_bytes: 0,
  }
  docsStore.openDocument(newDoc)
}

async function handleOpen() {
  const selected = await open({
    multiple: false,
    filters: [
      {
        name: 'Markdown',
        extensions: ['md', 'markdown', 'txt'],
      },
    ],
  })

  if (selected && typeof selected === 'string') {
    await openFile(selected)
  }
}

async function openFile(path: string) {
  const existing = docsStore.getDocByPath(path)
  if (existing) {
    docsStore.setActiveDoc(existing.id)
    return
  }

  try {
    const doc = await invoke<Document>('read_file', { path })
    docsStore.openDocument(doc)
  } catch (e) {
    console.error('Failed to open file:', e)
  }
}

async function handleSave() {
  if (!activeDoc.value) return

  if (!activeDoc.value.path) {
    await handleSaveAs()
    return
  }

  try {
    await invoke('write_file', {
      path: activeDoc.value.path,
      content: editorContent.value,
    })
    docsStore.markDocSaved(activeDocId.value!)
  } catch (e) {
    console.error('Failed to save file:', e)
  }
}

async function handleSaveAs() {
  if (!activeDoc.value) return

  const path = await save({
    defaultPath: activeDoc.value.title + '.md',
    filters: [
      {
        name: 'Markdown',
        extensions: ['md', 'markdown'],
      },
    ],
  })

  if (path) {
    try {
      await invoke('write_file', {
        path,
        content: editorContent.value,
      })

      const title = path.split(/[/\\]/).pop()?.replace(/\.[^.]+$/, '') || 'Untitled'
      docsStore.updateDocPath(activeDocId.value!, path, title)
      docsStore.markDocSaved(activeDocId.value!)
    } catch (e) {
      console.error('Failed to save file:', e)
    }
  }
}

function handleTabSelect(id: string) {
  docsStore.setActiveDoc(id)
}

function handleTabClose(id: string) {
  const doc = docsStore.docs.get(id)
  if (doc?.is_dirty) {
    if (!confirm('Save changes before closing?')) {
      // 用户取消，不关闭
      return
    }
    // 这里可以添加保存逻辑
  }
  docsStore.closeDocument(id)
}

function handleOpenFile(path: string) {
  openFile(path)
}

function handleSearchResult(path: string, line: number) {
  openFile(path).then(() => {
    nextTick(() => {
      editorRef.value?.scrollToLine(line)
    })
  })
}

function handleCursorChange(line: number, column: number) {
  cursorLine.value = line
  cursorColumn.value = column
  editorStore.setCursorPosition(line, column)
}

function handleEditorScroll(scrollTop: number) {
  // 可以实现同步滚动
}

function handleOutlineChange(items: OutlineItem[]) {
  editorStore.setOutline(items)
}

function handleOutlineNavigate(item: OutlineItem) {
  if (editorRef.value) {
    editorRef.value.scrollToLine(item.line)
  }
  if (previewRef.value) {
    previewRef.value.scrollToAnchor(item.anchor)
  }
}

function toggleSidebar() {
  themeStore.toggleSidebar()
}

function toggleOutline() {
  themeStore.toggleOutline()
}

function togglePreview() {
  themeStore.togglePreview()
}

function handleDrop(event: DragEvent) {
  event.preventDefault()
  const files = event.dataTransfer?.files
  if (!files) return

  for (let i = 0; i < files.length; i++) {
    const file = files[i]
    if (file.name.endsWith('.md') || file.name.endsWith('.markdown')) {
      // 处理 md 文件拖入
    }
  }
}

function handlePaste(event: ClipboardEvent) {
  if (!activeDoc.value?.path) return

  const baseDir = activeDoc.value.path.split(/[/\\]/).slice(0, -1).join('/')
  if (!baseDir) return

  handleImagePaste(event, baseDir).then((images) => {
    if (images.length > 0) {
      const markdown = images.map((img) => `![${img.fileName}](${img.relativePath})`).join('\n')
      // 将 markdown 插入到编辑器
      const view = editorRef.value?.getEditorView()
      if (view) {
        const pos = view.state.selection.main.head
        view.dispatch({
          changes: {
            from: pos,
            insert: '\n' + markdown + '\n',
          },
          selection: { anchor: pos + markdown.length + 2 },
        })
      }
    }
  })
}

onMounted(() => {
  themeStore.loadTheme()

  document.addEventListener('dragover', (e) => e.preventDefault())
  document.addEventListener('drop', handleDrop)
  document.addEventListener('paste', handlePaste)

  // 自动快照定时器
  setInterval(() => {
    if (activeDoc.value?.path && activeDoc.value?.is_dirty) {
      createSnapshot(activeDoc.value.path, editorContent.value).catch(console.error)
    }
  }, 10 * 60 * 1000) // 每 10 分钟

  // 处理键盘快捷键
  document.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault()
      handleSave()
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
      e.preventDefault()
      handleNew()
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 'o') {
      e.preventDefault()
      handleOpen()
    }
  })
})
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: var(--bg-color);
  color: var(--text-color);
  font-family: var(--font-family);
}

.main-content {
  display: flex;
  flex: 1;
  min-height: 0;
}

.sidebar-wrap {
  width: 280px;
  flex-shrink: 0;
  border-right: 1px solid var(--border-color);
}

.sidebar-hidden .sidebar-wrap {
  display: none;
}

.editor-area {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
}

.editor-preview-container {
  display: flex;
  flex: 1;
  min-height: 0;
  position: relative;
}

.editor-pane {
  flex: 1;
  min-width: 0;
  border-right: 1px solid var(--border-color);
}

.preview-pane {
  flex: 1;
  min-width: 0;
  overflow: hidden;
}

.outline-pane {
  width: 240px;
  flex-shrink: 0;
}

.empty-state-wrap {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-state {
  text-align: center;
  color: var(--text-muted);
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.empty-state h2 {
  margin: 0 0 8px 0;
  color: var(--text-color);
  font-size: 24px;
}

.empty-state p {
  margin: 0 0 24px 0;
  font-size: 14px;
}

.primary-btn {
  padding: 10px 24px;
  background: var(--accent-color);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: opacity 0.15s;
}

.primary-btn:hover {
  opacity: 0.9;
}
</style>
