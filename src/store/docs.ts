import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Document } from '@/types'
import { invoke } from '@tauri-apps/api/core'

export const useDocsStore = defineStore('docs', () => {
  const docs = ref<Map<string, Document>>(new Map())
  const activeDocId = ref<string | null>(null)
  const recentFiles = ref<string[]>([])

  const activeDoc = computed(() => {
    if (!activeDocId.value) return null
    return docs.value.get(activeDocId.value) || null
  })

  const docList = computed(() => {
    return Array.from(docs.value.values())
  })

  function openDocument(doc: Document) {
    docs.value.set(doc.id, doc)
    activeDocId.value = doc.id
    addRecentFile(doc.path)
  }

  function closeDocument(docId: string) {
    const doc = docs.value.get(docId)
    if (doc && doc.is_dirty) {
      return false
    }
    docs.value.delete(docId)
    if (activeDocId.value === docId) {
      const remaining = Array.from(docs.value.keys())
      activeDocId.value = remaining.length > 0 ? remaining[remaining.length - 1] : null
    }
    return true
  }

  function setActiveDoc(docId: string) {
    if (docs.value.has(docId)) {
      activeDocId.value = docId
    }
  }

  function updateDocContent(docId: string, content: string) {
    const doc = docs.value.get(docId)
    if (doc) {
      doc.content = content
      doc.is_dirty = true
      doc.size_bytes = new Blob([content]).size
    }
  }

  function markDocSaved(docId: string) {
    const doc = docs.value.get(docId)
    if (doc) {
      doc.is_dirty = false
      doc.updated_at = new Date().toISOString()
    }
  }

  function updateDocPath(docId: string, path: string, title: string) {
    const doc = docs.value.get(docId)
    if (doc) {
      doc.path = path
      doc.title = title
    }
  }

  function addRecentFile(path: string) {
    const index = recentFiles.value.indexOf(path)
    if (index > -1) {
      recentFiles.value.splice(index, 1)
    }
    recentFiles.value.unshift(path)
    if (recentFiles.value.length > 20) {
      recentFiles.value.pop()
    }
  }

  function hasUnsavedChanges(): boolean {
    for (const doc of docs.value.values()) {
      if (doc.is_dirty) return true
    }
    return false
  }

  function getDocByPath(path: string): Document | undefined {
    for (const doc of docs.value.values()) {
      if (doc.path === path) return doc
    }
    return undefined
  }

  return {
    docs,
    activeDocId,
    activeDoc,
    docList,
    recentFiles,
    openDocument,
    closeDocument,
    setActiveDoc,
    updateDocContent,
    markDocSaved,
    updateDocPath,
    addRecentFile,
    hasUnsavedChanges,
    getDocByPath,
  }
})
