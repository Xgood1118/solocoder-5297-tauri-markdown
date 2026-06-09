import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { SyncConfig, SyncProvider } from '@/types'
import { invoke } from '@tauri-apps/api/core'

type SyncStatus = 'idle' | 'syncing' | 'success' | 'failed'

export const useSyncStore = defineStore('sync', () => {
  const config = ref<SyncConfig>({
    provider: 'disabled',
    endpoint: '',
    username: '',
    password: '',
    bucket: undefined,
    region: undefined,
    remote_path: '/',
    auto_sync: false,
    sync_interval_secs: 300,
  })

  const status = ref<SyncStatus>('idle')
  const lastSync = ref<string | null>(null)
  const error = ref<string | null>(null)

  const isEnabled = computed(() => config.value.provider !== 'disabled')
  const isSyncing = computed(() => status.value === 'syncing')

  async function loadConfig() {
    try {
      const savedConfig = await invoke<SyncConfig>('get_sync_config')
      config.value = savedConfig
    } catch (e) {
      console.error('Failed to load sync config:', e)
    }
  }

  async function saveConfig() {
    try {
      await invoke('set_sync_config', { config: config.value })
    } catch (e) {
      console.error('Failed to save sync config:', e)
      throw e
    }
  }

  async function testConnection(): Promise<boolean> {
    try {
      const result = await invoke<boolean>('test_sync_connection', {
        config: config.value,
      })
      return result
    } catch (e) {
      error.value = String(e)
      return false
    }
  }

  async function syncAll(localDir: string) {
    if (status.value === 'syncing') return

    status.value = 'syncing'
    error.value = null

    try {
      const log = await invoke<string[]>('sync_all', { localDir })
      status.value = 'success'
      lastSync.value = new Date().toISOString()
      return log
    } catch (e) {
      status.value = 'failed'
      error.value = String(e)
      throw e
    }
  }

  function setProvider(provider: SyncProvider) {
    config.value.provider = provider
  }

  function setEndpoint(endpoint: string) {
    config.value.endpoint = endpoint
  }

  function setCredentials(username: string, password: string) {
    config.value.username = username
    config.value.password = password
  }

  function setRemotePath(path: string) {
    config.value.remote_path = path
  }

  function setAutoSync(enabled: boolean, interval?: number) {
    config.value.auto_sync = enabled
    if (interval) {
      config.value.sync_interval_secs = interval
    }
  }

  return {
    config,
    status,
    lastSync,
    error,
    isEnabled,
    isSyncing,
    loadConfig,
    saveConfig,
    testConnection,
    syncAll,
    setProvider,
    setEndpoint,
    setCredentials,
    setRemotePath,
    setAutoSync,
  }
})
