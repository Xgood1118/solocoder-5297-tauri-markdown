<template>
  <div class="outline-panel">
    <div class="outline-header">
      <span class="outline-title">{{ t('outline.title') }}</span>
    </div>
    <div class="outline-content" v-if="items.length > 0">
      <div
        v-for="(item, index) in items"
        :key="index"
        :class="['outline-item', `level-${item.level}`]"
        @click="handleClick(item)"
      >
        <span class="outline-text" :title="item.text">
          {{ item.text }}
        </span>
      </div>
    </div>
    <div v-else class="outline-empty">
      {{ t('outline.empty') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { OutlineItem } from '@/types'

defineProps<{
  items: OutlineItem[]
}>()

const emit = defineEmits<{
  (e: 'navigate', item: OutlineItem): void
}>()

const { t } = useI18n()

function handleClick(item: OutlineItem) {
  emit('navigate', item)
}
</script>

<style scoped>
.outline-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border-color);
}

.outline-header {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.outline-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.outline-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.outline-item {
  padding: 6px 16px;
  cursor: pointer;
  transition: background 0.15s;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 13px;
}

.outline-item:hover {
  background: var(--hover-color);
}

.outline-item.level-1 {
  padding-left: 12px;
  font-weight: 600;
}

.outline-item.level-2 {
  padding-left: 28px;
}

.outline-item.level-3 {
  padding-left: 44px;
  font-size: 12px;
  color: var(--text-secondary);
}

.outline-text {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.outline-empty {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
  font-size: 13px;
}
</style>
