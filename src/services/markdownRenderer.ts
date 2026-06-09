import MarkdownIt from 'markdown-it'
import markdownItAnchor from 'markdown-it-anchor'
import markdownItTaskLists from 'markdown-it-task-lists'
import hljs from 'highlight.js'
import katex from 'katex'
import type { OutlineItem } from '@/types'

class MarkdownRenderer {
  private md: MarkdownIt
  private outlineCache: OutlineItem[] = []

  constructor() {
    this.md = new MarkdownIt({
      html: true,
      linkify: true,
      typographer: true,
      breaks: true,
    })

    this.md.use(markdownItAnchor, {
      level: [1, 2, 3],
      permalink: false,
      slugify: (s: string) =>
        s
          .toLowerCase()
          .trim()
          .replace(/[\s]+/g, '-')
          .replace(/[^\w\-]/g, ''),
    })

    this.md.use(markdownItTaskLists, {
      enabled: true,
      label: true,
    })

    this.setupHighlight()
    this.setupKatex()
    this.setupMermaid()
  }

  private setupHighlight() {
    const defaultRender = this.md.renderer.rules.fence || function (tokens, idx, options, env, self) {
      return self.renderToken(tokens, idx, options)
    }

    this.md.renderer.rules.fence = (tokens, idx, options, env, self) => {
      const token = tokens[idx]
      const info = token.info ? token.info.trim() : ''
      const lang = info.split(/\s+/)[0]
      const code = token.content

      if (lang === 'mermaid') {
        return `<div class="mermaid-diagram" data-mermaid="${encodeURIComponent(code)}"></div>`
      }

      if (lang && hljs.getLanguage(lang)) {
        try {
          const highlighted = hljs.highlight(code, { language: lang }).value
          return `<pre class="hljs"><code class="language-${lang}">${highlighted}</code></pre>`
        } catch (_) {
          // fall through
        }
      }

      const highlighted = hljs.highlightAuto(code).value
      return `<pre class="hljs"><code>${highlighted}</code></pre>`
    }
  }

  private setupKatex() {
    const inlineRule = (state: any, silent: boolean) => {
      const start = state.pos
      const src = state.src

      if (src.charCodeAt(start) !== 0x24) return false
      if (src.charCodeAt(start + 1) === 0x24) return false

      let end = -1
      let i = start + 1
      while (i < src.length) {
        if (src.charCodeAt(i) === 0x24 && src.charCodeAt(i - 1) !== 0x5c) {
          end = i
          break
        }
        if (src.charCodeAt(i) === 0x0a) break
        i++
      }

      if (end === -1 || end === start + 1) return false
      if (silent) return true

      const math = src.slice(start + 1, end)

      try {
        const rendered = katex.renderToString(math, {
          throwOnError: false,
          displayMode: false,
        })
        const token = state.push('math_inline', 'span', 0)
        token.content = rendered
        token.markup = '$'
        token.info = 'math'
        state.pos = end + 1
        return true
      } catch (e) {
        return false
      }
    }

    const blockRule = (state: any, start: number, end: number, silent: boolean) => {
      const src = state.src
      let pos = state.bMarks[start] + state.tShift[start]
      const max = state.eMarks[start]

      if (pos + 2 > max) return false
      if (src.charCodeAt(pos) !== 0x24 || src.charCodeAt(pos + 1) !== 0x24) return false

      let firstLine = start
      let lastLine = start
      let found = false
      let mathContent = ''

      pos += 2
      if (pos < max) {
        mathContent += src.slice(pos, max)
      }

      for (let i = start + 1; i < end; i++) {
        pos = state.bMarks[i] + state.tShift[i]
        const lineEnd = state.eMarks[i]

        let hasClosing = false
        let closingPos = -1

        for (let j = pos; j < lineEnd - 1; j++) {
          if (src.charCodeAt(j) === 0x24 && src.charCodeAt(j + 1) === 0x24) {
            if (j > pos) {
              mathContent += '\n' + src.slice(pos, j)
            }
            hasClosing = true
            closingPos = j
            lastLine = i
            found = true
            break
          }
        }

        if (!hasClosing) {
          mathContent += '\n' + src.slice(pos, lineEnd)
        } else {
          break
        }
      }

      if (!found) return false
      if (silent) return true

      try {
        const rendered = katex.renderToString(mathContent.trim(), {
          throwOnError: false,
          displayMode: true,
        })

        const token = state.push('math_block', 'div', 0)
        token.block = true
        token.content = rendered
        token.map = [firstLine, lastLine + 1]
        token.markup = '$$'
        token.info = 'math'

        state.line = lastLine + 1
        return true
      } catch (e) {
        return false
      }
    }

    this.md.inline.ruler.after('escape', 'math_inline', inlineRule)
    this.md.block.ruler.after('fence', 'math_block', blockRule, {
      alt: ['paragraph', 'reference', 'blockquote', 'list'],
    })

    this.md.renderer.rules.math_inline = (tokens: any[], idx: number) => {
      return `<span class="math-inline">${tokens[idx].content}</span>`
    }

    this.md.renderer.rules.math_block = (tokens: any[], idx: number) => {
      return `<div class="math-block">${tokens[idx].content}</div>`
    }
  }

  private setupMermaid() {
    // Mermaid is handled in the fence rule and rendered on the client side
    // after the DOM is updated
  }

  render(content: string): string {
    this.outlineCache = []
    const html = this.md.render(content)
    this.extractOutline(content)
    return html
  }

  private extractOutline(content: string) {
    const lines = content.split('\n')
    const outline: OutlineItem[] = []
    const usedAnchors = new Set<string>()

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]
      const match = line.match(/^(#{1,3})\s+(.+)$/)

      if (match) {
        const level = match[1].length
        const text = match[2].trim()
        const anchor = this.slugify(text)

        let uniqueAnchor = anchor
        let counter = 1
        while (usedAnchors.has(uniqueAnchor)) {
          uniqueAnchor = `${anchor}-${counter}`
          counter++
        }
        usedAnchors.add(uniqueAnchor)

        outline.push({
          level,
          text,
          anchor: uniqueAnchor,
          line: i + 1,
        })
      }
    }

    this.outlineCache = outline
  }

  private slugify(s: string): string {
    return s
      .toLowerCase()
      .trim()
      .replace(/[\s]+/g, '-')
      .replace(/[^\w\-一-龥ぁ-ゔァ-ヴー々〆〤]/g, '')
  }

  getOutline(): OutlineItem[] {
    return this.outlineCache
  }
}

export const markdownRenderer = new MarkdownRenderer()
