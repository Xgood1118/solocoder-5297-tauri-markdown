import { invoke } from '@tauri-apps/api/core'

export async function saveImage(
  baseDir: string,
  fileName: string,
  imageData: ArrayBuffer,
): Promise<string> {
  const bytes = Array.from(new Uint8Array(imageData))
  const relativePath = await invoke<string>('save_image', {
    baseDir,
    fileName,
    imageData: bytes,
  })
  return relativePath
}

export function handleImageDrop(
  event: DragEvent,
  baseDir: string,
): Promise<{ fileName: string; relativePath: string }[]> {
  return new Promise((resolve, reject) => {
    const files = event.dataTransfer?.files
    if (!files || files.length === 0) {
      resolve([])
      return
    }

    const imageFiles = Array.from(files).filter((f) => f.type.startsWith('image/'))
    if (imageFiles.length === 0) {
      resolve([])
      return
    }

    const promises = imageFiles.map((file) =>
      file.arrayBuffer().then((buffer) =>
        saveImage(baseDir, file.name, buffer).then((relativePath) => ({
          fileName: file.name,
          relativePath,
        })),
      ),
    )

    Promise.all(promises)
      .then((results) => resolve(results))
      .catch(reject)
  })
}

export function handleImagePaste(
  event: ClipboardEvent,
  baseDir: string,
): Promise<{ fileName: string; relativePath: string }[]> {
  return new Promise((resolve, reject) => {
    const items = event.clipboardData?.items
    if (!items || items.length === 0) {
      resolve([])
      return
    }

    const imageItems: Promise<{ fileName: string; relativePath: string }>[] = []

    for (let i = 0; i < items.length; i++) {
      const item = items[i]
      if (item.type.startsWith('image/')) {
        const file = item.getAsFile()
        if (file) {
          imageItems.push(
            file.arrayBuffer().then((buffer) =>
              saveImage(baseDir, file.name || 'image.png', buffer).then((relativePath) => ({
                fileName: file.name || 'image.png',
                relativePath,
              })),
            ),
          )
        }
      }
    }

    Promise.all(imageItems)
      .then((results) => resolve(results))
      .catch(reject)
  })
}
