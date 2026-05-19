import {useState, useEffect} from 'react'
import ConnectionStatus from './components/ConnectionStatus'
import StockInfo from './components/StockInfo'
import PushLog from './components/PushLog'
import Settings from './components/Settings'
import AlertPanel from './components/AlertPanel'
import type {StatusResponse, PushLogEntry} from '@shared/types'

export default function App() {
    const [status, setStatus] = useState<StatusResponse & { connected: boolean }>({
        connected: false, status: 'unknown', version: '', uptime_secs: 0,
    })
    const [currentCode, setCurrentCode] = useState<string | null>(null)
    const [pushLog, setPushLog] = useState<PushLogEntry[]>([])

    // 刷新状态
    const refresh = () => {
        chrome.runtime.sendMessage({type: 'GET_STATUS'}, (res) => {
            if (res) setStatus(res)
        })
        chrome.runtime.sendMessage({type: 'GET_CURRENT_CODE'}, (res) => {
            debugger;
            if (res) setCurrentCode(res)
        })
        chrome.runtime.sendMessage({type: 'GET_PUSH_LOG'}, (res) => {
            if (res) setPushLog(res)
        })
    }

    useEffect(() => {
        refresh()
        const timer = setInterval(refresh, 5000)
        return () => clearInterval(timer)
    }, [])

    return (
        <div className="p-3 space-y-2 bg-gray-50 min-h-screen">
            <h1 className="text-base font-bold text-blue-600">astock Bridge</h1>
            <ConnectionStatus status={status}/>
            <StockInfo code={currentCode}/>
            <AlertPanel currentCode={currentCode}/>
            <PushLog entries={pushLog}/>
            <Settings onSaved={refresh}/>
        </div>
    )
}
