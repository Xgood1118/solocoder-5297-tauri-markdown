<template>
  <div class="status-bar">
    <div class="status-left">
      <span v-if="cursorLine" class="status-item">
        Ln {{ cursorLine }}, Col {{ cursorColumn }}
      </span>
      <span v-if="wordCount !== undefined" class="status-item">
        {{ wordCount }} words
      </span>
      <span v-if="charCount !== undefined" class="status-item">
        {{ charCount }} chars
      </span>
    </div>
    <div class="status-right">
      <span class="status-item" v-if="readTime">{{ readTime }}</span>
      <span class="status-item">Markdown</span>
      <span v-if="isDirty" class="status-item dirty">● Unsaved</span>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  cursorLine?: number
  cursorColumn?: number
  wordCount?: number
  charCount?: number
  readTime?: string
  isDirty?: boolean
}>()
</script>

<style scoped>
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 24px;
  padding: 0 12px;
  background: var(--accent-color);
  color: white;
  font-size: 12px;
  flex-shrink: 0;
}

.status-left,
.status-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.status-item.dirty {
  font-weight: 500;
}
</style>
