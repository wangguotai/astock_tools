interface Props {
  code: string | null
}

export default function StockInfo({ code }: Props) {
  return (
    <div className="bg-white rounded-lg p-2.5">
      <h3 className="text-xs text-gray-500 mb-1">当前页面</h3>
      <div className="text-sm font-medium">
        {code ? `股票代码: ${code}` : '未检测到股票页面'}
      </div>
      {code && (
        <div className="text-xs text-gray-400 mt-0.5">
          请打开 stockpage.10jqka.com.cn/{code}
        </div>
      )}
    </div>
  )
}
