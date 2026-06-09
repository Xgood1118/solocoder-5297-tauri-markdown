<template>
  <div ref="editorContainer" class="editor-container"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, shallowRef } from 'vue'
import { EditorState } from '@codemirror/state'
import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter, drawSelection, dropCursor, rectangularSelection, crosshairCursor, highlightSpecialChars } from '@codemirror/view'
import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands'
import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { languages } from '@codemirror/language-data'
import { bracketMatching } from '@codemirror/language'
import { syntaxHighlighting, HighlightStyle } from '@codemirror/language'
import { tags as t } from '@lezer/highlight'
import { searchKeymap, highlightSelectionMatches } from '@codemirror/search'
import { autocompletion, completionKeymap } from '@codemirror/autocomplete'
import { lintKeymap } from '@codemirror/lint'
import { foldGutter, foldKeymap } from '@codemirror/language'

const props = defineProps<{
  modelValue: string
  theme: 'light' | 'dark' | 'sepia' | 'solarized'
  readOnly?: boolean
  fontSize?: number
  fontFamily?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
  (e: 'cursorChange', line: number, column: number): void
  (e: 'scroll', scrollTop: number): void
  (e: 'focus'): void
  (e: 'blur'): void
}>()

const editorContainer = ref<HTMLElement | null>(null)
const editorView = shallowRef<EditorView | null>(null)

const themeStyles = {
  light: {
    background: '#ffffff',
    foreground: '#333333',
    selection: '#cfe2ff',
    cursor: '#000000',
    lineNumbers: '#999999',
    activeLine: '#f5f5f5',
    gutter: '#fafafa',
    gutterBorder: '#e0e0e0',
    keyword: '#0000ff',
    string: '#a31515',
    comment: '#008000',
    heading: '#000000',
    link: '#0066cc',
    code: '#dcdcdc',
  },
  dark: {
    background: '#1e1e1e',
    foreground: '#d4d4d4',
    selection: '#264f78',
    cursor: '#aeafad',
    lineNumbers: '#858585',
    activeLine: '#2a2d2e',
    gutter: '#1e1e1e',
    gutterBorder: '#3c3c3c',
    keyword: '#569cd6',
    string: '#ce9178',
    comment: '#6a9955',
    heading: '#e8e8e8',
    link: '#3794ff',
    code: '#1a1a1a',
  },
  sepia: {
    background: '#fdf6e3',
    foreground: '#5c4b37',
    selection: '#eee0b0',
    cursor: '#5c4b37',
    lineNumbers: '#9a8870',
    activeLine: '#f0e5cc',
    gutter: '#f5ead0',
    gutterBorder: '#d9c9a3',
    keyword: '#b58900',
    string: '#cb4b16',
    comment: '#859900',
    heading: '#5c4b37',
    link: '#b58900',
    code: '#f0e6c8',
  },
  solarized: {
    background: '#002b36',
    foreground: '#93a1a1',
    selection: '#073642',
    cursor: '#93a1a1',
    lineNumbers: '#586e75',
    activeLine: '#073642',
    gutter: '#002b36',
    gutterBorder: '#073642',
    keyword: '#268bd2',
    string: '#2aa198',
    comment: '#586e75',
    heading: '#93a1a1',
    link: '#268bd2',
    code: '#073642',
  },
}

function createHighlightStyle(themeKey: string) {
  const t_ = themeStyles[themeKey as keyof typeof themeStyles] || themeStyles.light
  return HighlightStyle.define([
    { tag: t.keyword, color: t_.keyword },
    { tag: t.string, color: t_.string },
    { tag: t.comment, color: t_.comment },
    { tag: t.heading, color: t_.heading, fontWeight: 'bold' },
    { tag: t.link, color: t_.link, textDecoration: 'underline' },
    { tag: t.emphasis, fontStyle: 'italic' },
    { tag: t.strong, fontWeight: 'bold' },
    { tag: t.monospace, fontFamily: 'monospace' },
    { tag: t.url, color: t_.link },
    { tag: t.meta, color: t_.comment },
    { tag: t.invalid, color: '#ff0000' },
  ])
}

function createThemeExtension(themeKey: string) {
  const t_ = themeStyles[themeKey as keyof typeof themeStyles] || themeStyles.light
  return EditorView.theme({
    '&': {
      backgroundColor: t_.background,
      color: t_.foreground,
    },
    '.cm-content': {
      caretColor: t_.cursor,
      padding: '10px 0',
    },
    '.cm-cursor, .cm-dropCursor': {
      borderLeftColor: t_.cursor,
    },
    '&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection':
      {
        backgroundColor: t_.selection,
      },
    '.cm-gutters': {
      backgroundColor: t_.gutter,
      color: t_.lineNumbers,
      border: 'none',
      borderRight: `1px solid ${t_.gutterBorder}`,
    },
    '.cm-activeLineGutter': {
      backgroundColor: t_.activeLine,
    },
    '.cm-activeLine': {
      backgroundColor: t_.activeLine,
    },
    '.cm-lineNumbers': {
      padding: '0 8px',
    },
    '.cm-foldPlaceholder': {
      backgroundColor: t_.code,
      border: `1px solid ${t_.gutterBorder}`,
      borderRadius: '2px',
    },
    '.cm-tooltip': {
      backgroundColor: t_.background,
      border: `1px solid ${t_.gutterBorder}`,
    },
    '.cm-tooltip-autocomplete': {
      '& > ul > li': {
        color: t_.foreground,
      },
      '& > ul > li[aria-selected]': {
        backgroundColor: t_.activeLine,
      },
    },
    '.cm-searchMatch': {
      backgroundColor: '#ffff0040',
    },
    '.cm-searchMatch.cm-searchMatch-selected': {
      backgroundColor: '#ffa50080',
    },
    '.cm-panels': {
      backgroundColor: t_.background,
      color: t_.foreground,
    },
    '.cm-panel': {
      backgroundColor: t_.background,
      color: t_.foreground,
    },
  })
}

function createEditorState(content: string, themeKey: string) {
  const extensions = [
    lineNumbers(),
    highlightActiveLineGutter(),
    highlightSpecialChars(),
    history(),
    foldGutter(),
    drawSelection(),
    dropCursor(),
    EditorState.allowMultipleSelections.of(true),
    highlightActiveLine(),
    highlightSelectionMatches(),
    rectangularSelection(),
    crosshairCursor(),
    bracketMatching(),
    markdown({ base: markdownLanguage, codeLanguages: languages }),
    syntaxHighlighting(createHighlightStyle(themeKey)),
    autocompletion(),
    keymap.of([
      ...defaultKeymap,
      ...historyKeymap,
      ...searchKeymap,
      ...completionKeymap,
      ...lintKeymap,
      indentWithTab,
    ]),
    EditorView.lineWrapping,
    createThemeExtension(themeKey),
    EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        const newValue = update.state.doc.toString()
        emit('update:modelValue', newValue)
      }
      if (update.selectionSet) {
        const selection = update.state.selection.main
        const line = update.state.doc.lineAt(selection.head)
        emit('cursorChange', line.number, selection.head - line.from + 1)
      }
    }),
    EditorView.domEventHandlers({
      scroll(event, view) {
        emit('scroll', view.scrollDOM.scrollTop)
      },
      focus() {
        emit('focus')
      },
      blur() {
        emit('blur')
      },
    }),
  ]

  if (props.readOnly) {
    extensions.push(EditorView.editable.of(false))
  }

  return EditorState.create({
    doc: content,
    extensions,
  })
}

onMounted(() => {
  if (!editorContainer.value) return

  const state = createEditorState(props.modelValue, props.theme)
  editorView.value = new EditorView({
    state,
    parent: editorContainer.value,
  })
})

watch(
  () => props.modelValue,
  (newValue) => {
    if (!editorView.value) return
    const currentValue = editorView.value.state.doc.toString()
    if (newValue !== currentValue) {
      editorView.value.dispatch({
        changes: {
          from: 0,
          to: editorView.value.state.doc.length,
          insert: newValue,
        },
      })
    }
  },
)

watch(
  () => props.theme,
  (newTheme) => {
    if (!editorView.value) return
    const state = createEditorState(
      editorView.value.state.doc.toString(),
      newTheme,
    )
    editorView.value.setState(state)
  },
)

watch(
  () => props.fontSize,
  (size) => {
    if (editorContainer.value && size) {
      editorContainer.value.style.fontSize = `${size}px`
    }
  },
  { immediate: true },
)

watch(
  () => props.fontFamily,
  (font) => {
    if (editorContainer.value && font) {
      editorContainer.value.style.fontFamily = font
    }
  },
  { immediate: true },
)

onBeforeUnmount(() => {
  if (editorView.value) {
    editorView.value.destroy()
    editorView.value = null
  }
})

function scrollToLine(line: number) {
  if (!editorView.value) return
  const pos = editorView.value.state.doc.line(line).from
  editorView.value.dispatch({
    selection: { anchor: pos },
    effects: EditorView.scrollIntoView(pos, { y: 'start', yMargin: 50 }),
  })
  editorView.value.focus()
}

function getEditorView() {
  return editorView.value
}

defineExpose({
  scrollToLine,
  getEditorView,
  focus: () => editorView.value?.focus(),
})
</script>

<style scoped>
.editor-container {
  height: 100%;
  width: 100%;
  overflow: hidden;
}

.editor-container :deep(.cm-editor) {
  height: 100%;
}

.editor-container :deep(.cm-scroller) {
  overflow: auto;
}
</style>
