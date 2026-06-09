import { ref } from 'vue'
import type { PluginInfo, PluginType } from '@/types'
import { invoke } from '@tauri-apps/api/core'

class PluginManager {
  private plugins = ref<Map<string, PluginInfo>>(new Map())
  private loadedCodes = new Map<string, string>()
  private pluginSandboxes = new Map<string, any>()

  async loadPlugins(): Promise<PluginInfo[]> {
    try {
      const pluginList = await invoke<PluginInfo[]>('list_plugins')
      this.plugins.value = new Map(pluginList.map((p) => [p.id, p]))
      return pluginList
    } catch (e) {
      console.error('Failed to list plugins:', e)
      return []
    }
  }

  async loadPlugin(pluginId: string): Promise<boolean> {
    try {
      const code = await invoke<string>('load_plugin', { pluginId })
      this.loadedCodes.set(pluginId, code)
      return true
    } catch (e) {
      console.error(`Failed to load plugin ${pluginId}:`, e)
      return false
    }
  }

  async runCommand(pluginId: string, command: string, args: any): Promise<any> {
    const plugin = this.plugins.value.get(pluginId)
    if (!plugin) {
      throw new Error(`Plugin ${pluginId} not found`)
    }

    if (!this.loadedCodes.has(pluginId)) {
      await this.loadPlugin(pluginId)
    }

    const code = this.loadedCodes.get(pluginId)!
    return this.executeInSandbox(code, command, args)
  }

  private executeInSandbox(code: string, command: string, args: any): any {
    const sandbox = {
      console: {
        log: (...msg: any[]) => console.log('[Plugin]', ...msg),
        error: (...msg: any[]) => console.error('[Plugin]', ...msg),
        warn: (...msg: any[]) => console.warn('[Plugin]', ...msg),
      },
      args,
      result: null,
    }

    try {
      const wrappedCode = `
        (function(console, args) {
          ${code}
          if (typeof module !== 'undefined' && module.exports) {
            return module.exports;
          }
          if (typeof exports !== 'undefined') {
            return exports;
          }
          return {};
        })(sandbox.console, sandbox.args)
      `

      const factory = new Function('sandbox', wrappedCode)
      const pluginExports = factory(sandbox)

      if (typeof pluginExports[command] === 'function') {
        return pluginExports[command](args)
      }

      throw new Error(`Command ${command} not found in plugin`)
    } catch (e) {
      console.error(`Plugin execution error:`, e)
      throw e
    }
  }

  getPlugins(): PluginInfo[] {
    return Array.from(this.plugins.value.values())
  }

  isPluginLoaded(pluginId: string): boolean {
    return this.loadedCodes.has(pluginId)
  }

  unloadPlugin(pluginId: string): boolean {
    this.loadedCodes.delete(pluginId)
    this.pluginSandboxes.delete(pluginId)
    return true
  }

  getRenderers(): string[] {
    const renderers: string[] = []
    for (const plugin of this.plugins.value.values()) {
      if (plugin.plugin_type === 'renderer' as PluginType) {
        renderers.push(plugin.id)
      }
    }
    return renderers
  }

  getThemes(): string[] {
    const themes: string[] = []
    for (const plugin of this.plugins.value.values()) {
      if (plugin.plugin_type === 'theme' as PluginType) {
        themes.push(plugin.id)
      }
    }
    return themes
  }

  getCommands(): Map<string, string[]> {
    const commands = new Map<string, string[]>()
    for (const plugin of this.plugins.value.values()) {
      if (plugin.plugin_type === 'command' as PluginType) {
        commands.set(plugin.id, [])
      }
    }
    return commands
  }
}

export const pluginManager = new PluginManager()
