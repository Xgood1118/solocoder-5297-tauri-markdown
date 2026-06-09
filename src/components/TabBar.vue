<template>
  <div class="tab-bar">
    <div class="tabs-container">
      <div
        v-for="doc in docs"
        :key="doc.id"
        :class="['tab', { active: doc.id === activeDocId, dirty: doc.is_dirty }]"
        @click="handleTabClick(doc.id)"
        @mousedown.middle="handleMiddleClick($event, doc.id)"
        @contextmenu.prevent="handleContextMenu($event, doc.id)"
      >
        <span class="tab-title" :title="doc.path">
          {{ doc.title || 'Untitled' }}
          <span v-if="doc.is_dirty" class="dirty-dot"></span>
        </span>
        <button
          class="tab-close"
          @click.stop="handleClose(doc.id)"
          :title="t('tab.close')"
        >
          ×
        </button>
      </div>
    </div>
    <div class="tab-actions">
      <button class="action-btn" @click="$emit('newTab')" title="New tab">
        +
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { Document } from '@/types'

const props = defineProps<{
  docs: Document[]
  activeDocId: string | null
}>()

const emit = defineEmits<{
  (e: 'select', id: string): void
  (e: 'close', id: string): void
  (e: 'newTab'): void
}>()

const { t } = useI18n()

function handleTabClick(id: string) {
  emit('select', id)
}

function handleClose(id: string) {
  emit('close', id)
}

function handleMiddleClick(event: MouseEvent, id: string) {
  event.preventDefault()
  emit('close', id)
}

function handleContextMenu(event: MouseEvent, id: string) {
  // 可以在这里实现右键菜单
}
</script>

<style scoped>
.tab-bar {
  display: flex;
  align-items: stretch;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  height: 38px;
  overflow-x: auto;
  overflow-y: hidden;
  flex-shrink: 0;
}

.tabs-container {
  display: flex;
  flex: 1;
  overflow-x: auto;
  overflow-y: hidden;
}

.tab {
  display: flex;
  align-items: center;
  padding: 0 12px;
  min-width: 120px;
  max-width: 200px;
  height: 100%;
  cursor: pointer;
  border-right: 1px solid var(--border-color);
  background: var(--bg-secondary);
  transition: background 0.15s;
  user-select: none;
  position: relative;
  flex-shrink: 0;
}

.tab:hover {
  background: var(--hover-color);
}

.tab.active {
  background: var(--bg-color);
  color: var(--accent-color);
}

.tab.active::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: var(--accent-color);
}

.tab-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 6px;
}

.dirty-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent-color);
  flex-shrink: 0;
}

.tab-close {
  width: 18px;
  height: 18px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  margin-left: 8px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s, background 0.15s;
}

.tab:hover .tab-close {
  opacity: 1;
}

.tab-close:hover {
  background: var(--hover-color);
  color: var(--text-color);
}

.tab-actions {
  display: flex;
  align-items: center;
  padding: 0 8px;
  border-left: 1px solid var(--border-color);
  flex-shrink: 0;
}

.action-btn {
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 18px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: background 0.15s, color 0.15s;
}

.action-btn:hover {
  background: var(--hover-color);
  color: var(--text-color);
}
</style>
