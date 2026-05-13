#!/bin/bash
# 创建Chrome插件图标 (简单SVG→PNG占位)
# 实际使用时替换为正式设计的图标

cd "$(dirname "$0")/icons"

# 创建SVG图标
cat > icon.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128">
  <rect width="128" height="128" rx="20" fill="#1a73e8"/>
  <text x="64" y="78" text-anchor="middle" font-size="60" font-family="Arial" fill="white" font-weight="bold">A</text>
  <text x="64" y="105" text-anchor="middle" font-size="24" font-family="Arial" fill="#90CAF9">stock</text>
</svg>
EOF

echo "图标SVG已创建: icon.svg"
echo "请使用在线工具将SVG转为16/48/128 PNG图标"
echo "推荐: https://cloudconvert.com/svg-to-png"
