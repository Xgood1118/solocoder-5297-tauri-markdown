import { invoke } from '@tauri-apps/api/core'
import type { Snapshot } from '@/types'

export async function createSnapshot(filePath: string, content: string): Promise<Snapshot> {
  return invoke<Snapshot>('create_snapshot', { filePath, content })
}

export async function listSnapshots(filePath: string): Promise<Snapshot[]> {
  return invoke<Snapshot[]>('list_snapshots', { filePath })
}

export async function restoreSnapshot(
  snapshotId: string,
  filePath: string,
): Promise<string> {
  return invoke<string>('restore_snapshot', { snapshotId, filePath })
}

export async function deleteSnapshot(snapshotId: string, filePath: string): Promise<boolean> {
  return invoke<boolean>('delete_snapshot', { snapshotId, filePath })
}
