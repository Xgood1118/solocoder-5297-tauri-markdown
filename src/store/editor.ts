import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { OutlineItem } from '@/types'

export const useEditorStore = defineStore('editor', () => {
  const outline = ref<OutlineItem[]>([])
  const cursorLine = ref(1)
  const cursorColumn = ref(1)
  const wordCount = ref(0)
  const charCount = ref(0)
  const readTime = ref(0)

  function setOutline(items: OutlineItem[]) {
    outline.value = items
  }

  function setCursorPosition(line: number, column: number) {
    cursorLine.value = line
    cursorColumn.value = column
  }

  function updateStats(content: string) {
    charCount.value = content.length
    wordCount.value = content
      .trim()
      .split(/\s+/)
      .filter((w) => w.length > 0).length

    const chineseChars = (content.match(/[\u4e00-\u9fa5]/g) || []).length
    const englishWords = content
      .trim()
      .split(/[^a-zA-Z]+/)
      .filter((w) => w.length > 0).length

    const totalWords = chineseChars + englishWords
    readTime.value = Math.ceil(totalWords / 300)
  }

  const readingTimeText = computed(() => {
    if (readTime.value < 1) return '< 1 min'
    return `${readTime.value} min read`
  })

  return {
    outline,
    cursorLine,
    cursorColumn,
    wordCount,
    charCount,
    readTime,
    readingTimeText,
    setOutline,
    setCursorPosition,
    updateStats,
  }
})
