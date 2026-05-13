import type { PushLogEntry } from '@shared/types'

interface Props {
  entries: PushLogEntry[]
}

export default function PushLog({ entries }: Props) {
  return (
    <div className="bg-white rounded-lg p-2.5">
      <h3 className="text-xs text-gray-500 mb-1">推送记录</h3>
      {entries.length === 0 ? (
        <div className="text-xs text-gray-400">暂无记录</div>
      ) : (
        <div className="max-h-40 overflow-y-auto space-y-0.5">
          {entries.map((entry, i) => (
            <div key={i} className="text-xs flex items-center gap-1.5">
              <span
                className={`inline-block w-1.5 h-1.5 rounded-full ${
                  entry.success ? 'bg-green-500' : 'bg-red-400'
                }`}
              />
              <span className={entry.success ? 'text-green-700' : 'text-red-600'}>
                {entry.type}
              </span>
              {entry.count && (
                <span className="text-gray-400">({entry.count}条)</span>
              )}
              <span className="text-gray-400">{entry.code?.replace(/^(sh|sz|bj)/, '')}</span>
              <span className="text-gray-300 ml-auto">
                {entry.time?.substring(11, 19)}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
