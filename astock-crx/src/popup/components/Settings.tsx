import { useState, useEffect } from 'react'
import { getServerUrl, setServerUrl } from '@shared/storage'
import { DEFAULT_SERVER_URL } from '@shared/constants'

interface Props {
  onSaved: () => void
}

export default function Settings({ onSaved }: Props) {
  const [url, setUrl] = useState(DEFAULT_SERVER_URL)
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    getServerUrl().then((u) => setUrl(u))
  }, [])

  const handleSave = async () => {
    await setServerUrl(url.trim())
    setSaved(true)
    onSaved()
    setTimeout(() => setSaved(false), 2000)
  }

  const handleForcePush = () => {
    chrome.runtime.sendMessage({ type: 'FORCE_PUSH' })
  }

  return (
    <div className="bg-white rounded-lg p-2.5">
      <h3 className="text-xs text-gray-500 mb-1.5">设置</h3>
      <div className="flex items-center gap-2 mb-2">
        <label className="text-xs whitespace-nowrap text-gray-600">服务器</label>
        <input
          type="text"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          className="flex-1 text-xs px-2 py-1 border border-gray-200 rounded"
        />
      </div>
      <div className="flex gap-2">
        <button
          onClick={handleSave}
          className="flex-1 text-xs py-1.5 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
        >
          {saved ? '已保存' : '保存'}
        </button>
        <button
          onClick={handleForcePush}
          className="flex-1 text-xs py-1.5 bg-gray-100 text-gray-700 rounded hover:bg-gray-200 transition-colors"
        >
          强制推送
        </button>
      </div>
    </div>
  )
}
