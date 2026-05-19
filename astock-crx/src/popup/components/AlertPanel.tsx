import { useState, useEffect } from 'react'
import type { AlertRule, AlertHistory, AlertSignalType } from '@shared/types'

interface Props {
  currentCode: string | null
}

const SIGNAL_TYPES: { type: AlertSignalType; label: string; desc: string }[] = [
  { type: 'MA_GOLDEN_CROSS', label: 'MA金叉', desc: 'MA5上穿MA10' },
  { type: 'MA_DEAD_CROSS', label: 'MA死叉', desc: 'MA5下穿MA10' },
  { type: 'RSI_OVERBOUGHT', label: 'RSI超买', desc: 'RSI > 70' },
  { type: 'RSI_OVERSOLD', label: 'RSI超卖', desc: 'RSI < 30' },
  { type: 'MACD_GOLDEN_CROSS', label: 'MACD金叉', desc: 'DIF上穿DEA' },
  { type: 'MACD_DEAD_CROSS', label: 'MACD死叉', desc: 'DIF下穿DEA' },
  { type: 'MONEYFLOW_IN', label: '主力流入', desc: '净流入超过阈值(万元)' },
  { type: 'MONEYFLOW_OUT', label: '主力流出', desc: '净流出超过阈值(万元)' },
]

export default function AlertPanel({ currentCode }: Props) {
  const [rules, setRules] = useState<AlertRule[]>([])
  const [history, setHistory] = useState<AlertHistory[]>([])
  const [newCode, setNewCode] = useState('')
  const [selectedType, setSelectedType] = useState<AlertSignalType>('MA_GOLDEN_CROSS')
  const [threshold, setThreshold] = useState('100') // 万元
  const [activeTab, setActiveTab] = useState<'rules' | 'history'>('rules')

  useEffect(() => {
    loadData()
  }, [])

  const loadData = async () => {
    const [r, h] = await Promise.all([
      new Promise<AlertRule[]>(res => chrome.runtime.sendMessage({ type: 'GET_ALERT_RULES' }, (r: AlertRule[]) => res(r || []))),
      new Promise<AlertHistory[]>(res => chrome.runtime.sendMessage({ type: 'GET_ALERT_HISTORY' }, (h: AlertHistory[]) => res(h || []))),
    ])
    setRules(Array.isArray(r) ? r : [])
    setHistory(Array.isArray(h) ? h : [])
  }

  const handleAddRule = async () => {
    const code = newCode.trim() || currentCode
    if (!code) return
    const type = selectedType
    const params: Record<string, number> = {}
    if (type === 'MONEYFLOW_IN' || type === 'MONEYFLOW_OUT') {
      params.threshold = parseFloat(threshold) || 100
    }
    await new Promise(res => chrome.runtime.sendMessage({ type: 'ADD_ALERT_RULE', code, signal_type: type, params }, res))
    setNewCode('')
    loadData()
  }

  const handleDeleteRule = async (id: number) => {
    await new Promise(res => chrome.runtime.sendMessage({ type: 'DELETE_ALERT_RULE', id }, res))
    loadData()
  }

  return (
    <div className="bg-white rounded-lg p-2.5">
      <div className="flex gap-2 mb-2 border-b">
        <button
          onClick={() => setActiveTab('rules')}
          className={`text-xs px-2 py-1 ${activeTab === 'rules' ? 'border-b-2 border-blue-600 text-blue-600' : 'text-gray-500'}`}
        >
          告警规则
        </button>
        <button
          onClick={() => setActiveTab('history')}
          className={`text-xs px-2 py-1 ${activeTab === 'history' ? 'border-b-2 border-blue-600 text-blue-600' : 'text-gray-500'}`}
        >
          告警历史
        </button>
      </div>

      {activeTab === 'rules' && (
        <>
          {/* 添加规则 */}
          <div className="space-y-1.5 mb-3">
            <div className="flex gap-1.5">
              <input
                type="text"
                value={newCode || currentCode || ''}
                onChange={e => setNewCode(e.target.value)}
                placeholder={currentCode || '股票代码'}
                className="flex-1 text-xs px-2 py-1 border border-gray-200 rounded"
              />
              <select
                value={selectedType}
                onChange={e => setSelectedType(e.target.value as AlertSignalType)}
                className="text-xs px-1 py-1 border border-gray-200 rounded"
              >
                {SIGNAL_TYPES.map(t => (
                  <option key={t.type} value={t.type}>{t.label}</option>
                ))}
              </select>
            </div>
            {(selectedType === 'MONEYFLOW_IN' || selectedType === 'MONEYFLOW_OUT') && (
              <div className="flex items-center gap-1">
                <span className="text-xs text-gray-500">阈值(万元)</span>
                <input
                  type="number"
                  value={threshold}
                  onChange={e => setThreshold(e.target.value)}
                  className="flex-1 text-xs px-2 py-1 border border-gray-200 rounded"
                />
              </div>
            )}
            <button
              onClick={handleAddRule}
              className="w-full text-xs py-1.5 bg-green-600 text-white rounded hover:bg-green-700"
            >
              添加规则
            </button>
          </div>

          {/* 规则列表 */}
          {rules.length === 0 ? (
            <p className="text-xs text-gray-400 text-center py-2">暂无告警规则</p>
          ) : (
            <div className="space-y-1.5 max-h-48 overflow-y-auto">
              {rules.map(rule => {
                const signalInfo = SIGNAL_TYPES.find(t => t.type === rule.signal_type)
                return (
                  <div key={rule.id} className="flex items-center justify-between bg-gray-50 rounded p-1.5 text-xs">
                    <div>
                      <span className="font-medium">{rule.code}</span>
                      <span className="ml-1 text-gray-600">{signalInfo?.label || rule.signal_type}</span>
                      {rule.params.threshold && (
                        <span className="ml-1 text-gray-400">({rule.params.threshold}万)</span>
                      )}
                    </div>
                    <button
                      onClick={() => handleDeleteRule(rule.id)}
                      className="text-red-500 hover:text-red-700 ml-2"
                    >
                      ×
                    </button>
                  </div>
                )
              })}
            </div>
          )}
        </>
      )}

      {activeTab === 'history' && (
        <>
          {history.length === 0 ? (
            <p className="text-xs text-gray-400 text-center py-2">暂无告警历史</p>
          ) : (
            <div className="space-y-1 max-h-48 overflow-y-auto">
              {history.map(h => (
                <div key={h.id} className="bg-gray-50 rounded p-1.5 text-xs">
                  <span className="font-medium">{h.code}</span>
                  <span className="ml-1 text-gray-600">{h.signal_type}</span>
                  <p className="text-gray-700 mt-0.5">{h.message}</p>
                  <span className="text-gray-400 text-[10px]">{h.triggered_at}</span>
                </div>
              ))}
            </div>
          )}
        </>
      )}
    </div>
  )
}