<template>
  <div class="title-bar">
    <div class="title-bar-left">
      <div class="app-icon">📝</div>
      <span class="app-title">{{ title }}</span>
    </div>
    <div class="title-bar-center">
      <nav class="menu-bar">
        <div class="menu-item" @click="toggleMenu('file')">
          {{ t('menu.file') }}
          <div v-if="openMenu === 'file'" class="dropdown">
            <div class="dropdown-item" @click="emit('new')">
              <span>{{ t('menu.new') }}</span>
              <span class="shortcut">Ctrl+N</span>
            </div>
            <div class="dropdown-item" @click="emit('open')">
              <span>{{ t('menu.open') }}</span>
              <span class="shortcut">Ctrl+O</span>
            </div>
            <div class="dropdown-item" @click="emit('save')">
              <span>{{ t('menu.save') }}</span>
              <span class="shortcut">Ctrl+S</span>
            </div>
            <div class="dropdown-item" @click="emit('saveAs')">
              <span>{{ t('menu.save_as') }}</span>
              <span class="shortcut">Ctrl+Shift+S</span>
            </div>
            <div class="dropdown-divider"></div>
            <div class="dropdown-item" @click="emit('export')">
              {{ t('menu.export') }} ▸
            </div>
            <div class="dropdown-divider"></div>
            <div class="dropdown-item" @click="emit('exit')">
              {{ t('menu.exit') }}
            </div>
          </div>
        </div>
        <div class="menu-item" @click="toggleMenu('edit')">
          {{ t('edit') ? 'Edit' : t('menu.file') }}
          <div v-if="openMenu === 'edit'" class="dropdown">
            <div class="dropdown-item" @click="emit('undo')">
              <span>{{ t('edit.undo') }}</span>
              <span class="shortcut">Ctrl+Z</span>
            </div>
            <div class="dropdown-item" @click="emit('redo')">
              <span>{{ t('edit.redo') }}</span>
              <span class="shortcut">Ctrl+Y</span>
            </div>
            <div class="dropdown-divider"></div>
            <div class="dropdown-item" @click="emit('cut')">
              <span>{{ t('edit.cut') }}</span>
              <span class="shortcut">Ctrl+X</span>
            </div>
            <div class="dropdown-item" @click="emit('copy')">
              <span>{{ t('edit.copy') }}</span>
              <span class="shortcut">Ctrl+C</span>
            </div>
            <div class="dropdown-item" @click="emit('paste')">
              <span>{{ t('edit.paste') }}</span>
              <span class="shortcut">Ctrl+V</span>
            </div>
            <div class="dropdown-divider"></div>
            <div class="dropdown-item" @click="emit('find')">
              <span>{{ t('edit.find') }}</span>
              <span class="shortcut">Ctrl+F</span>
            </div>
            <div class="dropdown-item" @click="emit('replace')">
              <span>{{ t('edit.replace') }}</span>
              <span class="shortcut">Ctrl+H</span>
            </div>
          </div>
        </div>
        <div class="menu-item" @click="toggleMenu('view')">
          {{ t('menu.view') }}
          <div v-if="openMenu === 'view'" class="dropdown">
            <div class="dropdown-item" @click="emit('toggleSidebar')">
              {{ t('view.toggle_sidebar') }}
            </div>
            <div class="dropdown-item" @click="emit('toggleOutline')">
              {{ t('view.toggle_outline') }}
            </div>
            <div class="dropdown-item" @click="emit('togglePreview')">
              {{ t('view.toggle_preview') }}
            </div>
          </div>
        </div>
      </nav>
    </div>
    <div class="title-bar-right">
      <div class="window-controls">
        <button class="window-btn minimize" @click="emit('minimize')">—</button>
        <button class="window-btn maximize" @click="emit('maximize')">▢</button>
        <button class="window-btn close" @click="emit('close')">✕</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

defineProps<{
  title: string
}>()

const emit = defineEmits<{
  (e: 'new'): void
  (e: 'open'): void
  (e: 'save'): void
  (e: 'saveAs'): void
  (e: 'export'): void
  (e: 'exit'): void
  (e: 'undo'): void
  (e: 'redo'): void
  (e: 'cut'): void
  (e: 'copy'): void
  (e: 'paste'): void
  (e: 'find'): void
  (e: 'replace'): void
  (e: 'toggleSidebar'): void
  (e: 'toggleOutline'): void
  (e: 'togglePreview'): void
  (e: 'minimize'): void
  (e: 'maximize'): void
  (e: 'close'): void
}>()

const { t } = useI18n()
const openMenu = ref<string | null>(null)

function toggleMenu(menu: string) {
  openMenu.value = openMenu.value === menu ? null : menu
}
</script>

<style scoped>
.title-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 36px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  padding: 0 8px;
  flex-shrink: 0;
  user-select: none;
  -webkit-app-region: drag;
}

.title-bar-left,
.title-bar-right {
  display: flex;
  align-items: center;
  min-width: 100px;
}

.title-bar-left {
  gap: 8px;
}

.app-icon {
  font-size: 18px;
}

.app-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
}

.title-bar-center {
  flex: 1;
  display: flex;
  justify-content: center;
}

.menu-bar {
  display: flex;
  gap: 4px;
}

.menu-item {
  position: relative;
  padding: 4px 12px;
  font-size: 13px;
  cursor: pointer;
  border-radius: 4px;
  -webkit-app-region: no-drag;
}

.menu-item:hover {
  background: var(--hover-color);
}

.dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  min-width: 200px;
  background: var(--bg-color);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 4px 0;
  z-index: 1000;
  margin-top: 4px;
}

.dropdown-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  cursor: pointer;
  font-size: 13px;
  gap: 24px;
}

.dropdown-item:hover {
  background: var(--hover-color);
}

.shortcut {
  color: var(--text-muted);
  font-size: 12px;
}

.dropdown-divider {
  height: 1px;
  background: var(--border-color);
  margin: 4px 0;
}

.title-bar-right {
  justify-content: flex-end;
}

.window-controls {
  display: flex;
  gap: 4px;
  -webkit-app-region: no-drag;
}

.window-btn {
  width: 32px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 4px;
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s;
}

.window-btn:hover {
  background: var(--hover-color);
}

.window-btn.close:hover {
  background: #e81123;
  color: white;
}
</style>
