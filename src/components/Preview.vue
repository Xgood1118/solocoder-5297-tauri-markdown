<template>
  <div class="preview-container" ref="previewContainer">
    <div class="markdown-body" v-html="renderedHtml"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'
import { markdownRenderer } from '@/services/markdownRenderer'
import mermaid from 'mermaid'
import type { OutlineItem } from '@/types'

const props = defineProps<{
  content: string
  theme: 'light' | 'dark' | 'sepia' | 'solarized'
  fontSize?: number
  fontFamily?: string
}>()

const emit = defineEmits<{
  (e: 'outlineChange', outline: OutlineItem[]): void
  (e: 'scroll', scrollTop: number): void
}>()

const previewContainer = ref<HTMLElement | null>(null)
let renderDebounceTimer: number | null = null
let outlineThrottleTimer: number | null = null
let lastOutlineTime = 0

const renderedHtml = ref('')

function renderMarkdown() {
  if (renderDebounceTimer) {
    clearTimeout(renderDebounceTimer)
  }

  renderDebounceTimer = window.setTimeout(() => {
    renderedHtml.value = markdownRenderer.render(props.content)

    const now = Date.now()
    if (now - lastOutlineTime > 500) {
      lastOutlineTime = now
      emit('outlineChange', markdownRenderer.getOutline())
    } else {
      if (outlineThrottleTimer) {
        clearTimeout(outlineThrottleTimer)
      }
      outlineThrottleTimer = window.setTimeout(() => {
        lastOutlineTime = Date.now()
        emit('outlineChange', markdownRenderer.getOutline())
      }, 500 - (now - lastOutlineTime))
    }

    nextTick(() => {
      renderMermaid()
    })
  }, 300)
}

function renderMermaid() {
  if (!previewContainer.value) return

  const diagrams = previewContainer.value.querySelectorAll('.mermaid-diagram')
  diagrams.forEach((el, index) => {
    const diagramEl = el as HTMLElement
    const code = decodeURIComponent(diagramEl.dataset.mermaid || '')

    if (!code) return

    const id = `mermaid-${index}-${Date.now()}`

    try {
      mermaid.render(id, code).then((result) => {
        diagramEl.innerHTML = result.svg
      }).catch((err) => {
        diagramEl.innerHTML = `<div class="mermaid-error">Mermaid error: ${err.message}</div>`
      })
    } catch (e) {
      diagramEl.innerHTML = `<div class="mermaid-error">Mermaid error: ${e}</div>`
    }
  })
}

watch(
  () => props.content,
  () => {
    renderMarkdown()
  },
  { immediate: true },
)

watch(
  () => props.theme,
  () => {
    mermaid.initialize({
      theme: props.theme === 'dark' || props.theme === 'solarized' ? 'dark' : 'default',
    })
    renderMarkdown()
  },
)

watch(
  () => props.fontSize,
  (size) => {
    if (previewContainer.value && size) {
      previewContainer.value.style.fontSize = `${size}px`
    }
  },
  { immediate: true },
)

watch(
  () => props.fontFamily,
  (font) => {
    if (previewContainer.value && font) {
      previewContainer.value.style.fontFamily = font
    }
  },
  { immediate: true },
)

onMounted(() => {
  mermaid.initialize({
    startOnLoad: false,
    theme: props.theme === 'dark' || props.theme === 'solarized' ? 'dark' : 'default',
    securityLevel: 'strict',
  })

  if (previewContainer.value) {
    previewContainer.value.addEventListener('scroll', handleScroll)
  }
})

onBeforeUnmount(() => {
  if (renderDebounceTimer) clearTimeout(renderDebounceTimer)
  if (outlineThrottleTimer) clearTimeout(outlineThrottleTimer)
  if (previewContainer.value) {
    previewContainer.value.removeEventListener('scroll', handleScroll)
  }
})

function handleScroll(e: Event) {
  const target = e.target as HTMLElement
  emit('scroll', target.scrollTop)
}

function scrollToAnchor(anchor: string) {
  if (!previewContainer.value) return
  const el = previewContainer.value.querySelector(`#${anchor}`)
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }
}

defineExpose({
  scrollToAnchor,
})
</script>

<style scoped>
.preview-container {
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 20px 40px;
  box-sizing: border-box;
  line-height: 1.8;
}

.markdown-body {
  max-width: 800px;
  margin: 0 auto;
}

.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3),
.markdown-body :deep(h4),
.markdown-body :deep(h5),
.markdown-body :deep(h6) {
  margin-top: 24px;
  margin-bottom: 16px;
  font-weight: 600;
  line-height: 1.25;
}

.markdown-body :deep(h1) {
  font-size: 2em;
  padding-bottom: 0.3em;
  border-bottom: 1px solid var(--border-color);
}

.markdown-body :deep(h2) {
  font-size: 1.5em;
  padding-bottom: 0.3em;
  border-bottom: 1px solid var(--border-color);
}

.markdown-body :deep(h3) {
  font-size: 1.25em;
}

.markdown-body :deep(p) {
  margin-top: 0;
  margin-bottom: 16px;
}

.markdown-body :deep(blockquote) {
  margin: 0;
  padding: 0 1em;
  border-left: 4px solid var(--border-color);
  color: var(--text-secondary);
}

.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  margin-top: 0;
  margin-bottom: 16px;
  padding-left: 2em;
}

.markdown-body :deep(li) {
  margin-top: 4px;
}

.markdown-body :deep(code) {
  padding: 2px 6px;
  border-radius: 3px;
  font-family: var(--font-family, monospace);
  font-size: 0.9em;
  background: var(--code-bg);
}

.markdown-body :deep(pre) {
  padding: 16px;
  border-radius: 6px;
  overflow-x: auto;
  background: var(--code-bg);
  margin-bottom: 16px;
}

.markdown-body :deep(pre code) {
  padding: 0;
  background: transparent;
}

.markdown-body :deep(a) {
  color: var(--link-color);
  text-decoration: none;
}

.markdown-body :deep(a:hover) {
  text-decoration: underline;
}

.markdown-body :deep(table) {
  border-collapse: collapse;
  margin-bottom: 16px;
  width: 100%;
}

.markdown-body :deep(th),
.markdown-body :deep(td) {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  text-align: left;
}

.markdown-body :deep(th) {
  background: var(--bg-secondary);
  font-weight: 600;
}

.markdown-body :deep(hr) {
  border: none;
  border-top: 1px solid var(--border-color);
  margin: 24px 0;
}

.markdown-body :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: 4px;
}

.markdown-body :deep(.math-inline) {
  display: inline-block;
  margin: 0 2px;
}

.markdown-body :deep(.math-block) {
  display: flex;
  justify-content: center;
  margin: 16px 0;
  overflow-x: auto;
}

.markdown-body :deep(.mermaid-diagram) {
  display: flex;
  justify-content: center;
  margin: 16px 0;
}

.markdown-body :deep(.mermaid-error) {
  color: #ff0000;
  padding: 8px;
  border: 1px solid #ff0000;
  border-radius: 4px;
}

.markdown-body :deep(.task-list-item) {
  list-style: none;
  margin-left: -1em;
}

.markdown-body :deep(.task-list-item-checkbox) {
  margin-right: 8px;
}
</style>
