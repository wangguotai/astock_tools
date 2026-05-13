import { useState, useEffect } from 'react'
import { getServerUrl, setServerUrl } from '@shared/storage'
import { DEFAULT_SERVER_URL } from '@shared/constants'

export default function OptionsApp() {
  const [url, setUrl] = useState(DEFAULT_SERVER_URL)
  const [saved, setSaved] = useState(false)

  useEffect(() => {
    getServerUrl().then((u) => setUrl(u))
  }, [])

  const handleSave = async () => {
    await setServerUrl(url.trim())
    setSaved(true)
    setTimeout(() => setSaved(false), 2000)
  }

  return (
    <div className="max-w-lg mx-auto p-6">
      <h1 className="text-xl font-bold text-blue-600 mb-4">astock Bridge 设置</h1>

      <div className="mb-4">
        <label className="block text-sm font-medium mb-1">astock 接收端地址</label>
        <input
          type="text"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          className="w-full px-3 py-2 border rounded text-sm"
          placeholder="http://127.0.0.1:17320"
        />
        <p className="text-xs text-gray-400 mt-1">
          确保astock接收端已启动: <code className="bg-gray-100 px-1 rounded">astock receiver --port 17320</code>
        </p>
      </div>

      <button
        onClick={handleSave}
        className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
      >
        保存
      </button>
      {saved && <span className="text-green-600 ml-3 text-sm">已保存</span>}
    </div>
  )
}
