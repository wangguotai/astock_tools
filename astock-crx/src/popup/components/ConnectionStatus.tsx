import type { StatusResponse } from '@shared/types'

interface Props {
  status: StatusResponse & { connected: boolean }
}

export default function ConnectionStatus({ status }: Props) {
  return (
    <div className="bg-white rounded-lg p-2.5 flex items-center gap-2">
      <span
        className={`w-2.5 h-2.5 rounded-full ${
          status.connected ? 'bg-green-500' : 'bg-red-500'
        }`}
      />
      <span className="text-sm">
        {status.connected
          ? `已连接 (v${status.version}, 运行${status.uptime_secs}s)`
          : '未连接 - 请启动 astock receiver'}
      </span>
    </div>
  )
}
