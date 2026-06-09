import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import type { ThemeName, FontFamily } from '@/types'

export const useThemeStore = defineStore('theme', () => {
  const theme = ref<ThemeName>('light')
  const fontFamily = ref<FontFamily>('system')
  const fontSize = ref(16)
  const showSidebar = ref(true)
  const showOutline = ref(true)
  const showPreview = ref(true)
  const sidebarWidth = ref(280)
  const outlineWidth = ref(240)

  const fontFamilies: Record<FontFamily, string> = {
    system: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    serif: 'Georgia, "Times New Roman", serif',
    mono: '"SF Mono", Menlo, Monaco, Consolas, "Courier New", monospace',
    sans: '"Helvetica Neue", Arial, sans-serif',
  }

  const themes = {
    light: {
      bg: '#ffffff',
      bgSecondary: '#f5f5f5',
      bgTertiary: '#fafafa',
      text: '#333333',
      textSecondary: '#666666',
      textMuted: '#999999',
      border: '#e0e0e0',
      accent: '#0066cc',
      hover: '#f0f0f0',
      active: '#e6f0ff',
      code: '#f5f5f5',
      link: '#0066cc',
      selection: '#cfe2ff',
    },
    dark: {
      bg: '#1e1e1e',
      bgSecondary: '#252526',
      bgTertiary: '#2d2d2d',
      text: '#d4d4d4',
      textSecondary: '#858585',
      textMuted: '#666666',
      border: '#3c3c3c',
      accent: '#0078d4',
      hover: '#2a2d2e',
      active: '#094771',
      code: '#1a1a1a',
      link: '#3794ff',
      selection: '#264f78',
    },
    sepia: {
      bg: '#fdf6e3',
      bgSecondary: '#f5ead0',
      bgTertiary: '#faf0d9',
      text: '#5c4b37',
      textSecondary: '#7a6952',
      textMuted: '#9a8870',
      border: '#d9c9a3',
      accent: '#b58900',
      hover: '#f0e5cc',
      active: '#e8dcb0',
      code: '#f0e6c8',
      link: '#b58900',
      selection: '#eee0b0',
    },
    solarized: {
      bg: '#002b36',
      bgSecondary: '#073642',
      bgTertiary: '#003340',
      text: '#93a1a1',
      textSecondary: '#657b83',
      textMuted: '#586e75',
      border: '#073642',
      accent: '#268bd2',
      hover: '#073642',
      active: '#0a5060',
      code: '#073642',
      link: '#268bd2',
      selection: '#073642',
    },
  }

  const currentTheme = computed(() => themes[theme.value])
  const currentFontFamily = computed(() => fontFamilies[fontFamily.value])

  function applyTheme() {
    const root = document.documentElement
    const t = currentTheme.value

    root.style.setProperty('--bg-color', t.bg)
    root.style.setProperty('--bg-secondary', t.bgSecondary)
    root.style.setProperty('--bg-tertiary', t.bgTertiary)
    root.style.setProperty('--text-color', t.text)
    root.style.setProperty('--text-secondary', t.textSecondary)
    root.style.setProperty('--text-muted', t.textMuted)
    root.style.setProperty('--border-color', t.border)
    root.style.setProperty('--accent-color', t.accent)
    root.style.setProperty('--hover-color', t.hover)
    root.style.setProperty('--active-color', t.active)
    root.style.setProperty('--code-bg', t.code)
    root.style.setProperty('--link-color', t.link)
    root.style.setProperty('--selection-bg', t.selection)

    root.style.setProperty('--font-family', currentFontFamily.value)
    root.style.setProperty('--font-size', `${fontSize.value}px`)

    root.setAttribute('data-theme', theme.value)
  }

  function setTheme(newTheme: ThemeName) {
    theme.value = newTheme
    applyTheme()
    saveTheme()
  }

  function setFontFamily(newFont: FontFamily) {
    fontFamily.value = newFont
    applyTheme()
    saveTheme()
  }

  function setFontSize(size: number) {
    fontSize.value = Math.max(10, Math.min(32, size))
    applyTheme()
    saveTheme()
  }

  function toggleSidebar() {
    showSidebar.value = !showSidebar.value
  }

  function toggleOutline() {
    showOutline.value = !showOutline.value
  }

  function togglePreview() {
    showPreview.value = !showPreview.value
  }

  function saveTheme() {
    localStorage.setItem('theme', theme.value)
    localStorage.setItem('fontFamily', fontFamily.value)
    localStorage.setItem('fontSize', String(fontSize.value))
  }

  function loadTheme() {
    const savedTheme = localStorage.getItem('theme') as ThemeName | null
    const savedFont = localStorage.getItem('fontFamily') as FontFamily | null
    const savedSize = localStorage.getItem('fontSize')

    if (savedTheme && themes[savedTheme]) {
      theme.value = savedTheme
    }
    if (savedFont && fontFamilies[savedFont]) {
      fontFamily.value = savedFont
    }
    if (savedSize) {
      fontSize.value = parseInt(savedSize, 10)
    }

    applyTheme()
  }

  return {
    theme,
    fontFamily,
    fontSize,
    showSidebar,
    showOutline,
    showPreview,
    sidebarWidth,
    outlineWidth,
    currentTheme,
    currentFontFamily,
    setTheme,
    setFontFamily,
    setFontSize,
    toggleSidebar,
    toggleOutline,
    togglePreview,
    loadTheme,
    applyTheme,
  }
})
