"""
生成astock Bridge图标 - K线蜡烛图风格
深蓝底 + 红色三阳线(上涨趋势) + 白色A字
"""
from PIL import Image, ImageDraw, ImageFont

def create_icon(size, filename):
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    margin = max(1, int(size * 0.04))
    radius = max(3, int(size * 0.22))

    # 背景: 深蓝色
    draw.rounded_rectangle(
        [margin, margin, size - margin, size - margin],
        radius=radius,
        fill=(18, 65, 140, 255),
    )

    # 高光: 顶部稍亮
    draw.rounded_rectangle(
        [margin, margin, size - margin, int(size * 0.55)],
        radius=radius,
        fill=(28, 85, 175, 60),
    )

    # --- K线蜡烛图 ---
    # 阳线(红色): 实体 + 上下影线
    up_body = (220, 50, 50, 255)    # 红色实体
    up_wick = (220, 50, 50, 180)    # 红色影线(半透明)

    # 三根阳线，从左到右越来越高
    candles = [
        # (x_center, body_top, body_bottom, wick_top, wick_bottom)
        (0.26, 0.58, 0.76, 0.48, 0.82),
        (0.48, 0.40, 0.60, 0.30, 0.66),
        (0.72, 0.22, 0.44, 0.15, 0.52),
    ]

    candle_w = max(2, int(size * 0.10))
    wick_w = max(1, int(size * 0.035))

    for cx_r, bt_r, bb_r, wt_r, wb_r in candles:
        cx = int(size * cx_r)
        body_top = int(size * bt_r)
        body_bot = int(size * bb_r)
        wick_top = int(size * wt_r)
        wick_bot = int(size * wb_r)

        # 影线
        hw = max(1, wick_w // 2)
        draw.rectangle([cx - hw, wick_top, cx + hw, wick_bot], fill=up_wick)

        # 实体
        hcw = max(1, candle_w // 2)
        draw.rectangle([cx - hcw, body_top, cx + hcw, body_bot], fill=up_body)

    # 底部 "A" 字
    if size >= 32:
        try:
            font_size = max(7, int(size * 0.17))
            font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", font_size)
            text_y = int(size * 0.89)
            draw.text((size // 2, text_y), "A", fill=(255, 255, 255, 200), font=font, anchor="mm")
        except:
            pass

    img.save(filename, 'PNG')
    print(f'{filename}: {size}x{size}')

create_icon(16, 'icons/icon16.png')
create_icon(48, 'icons/icon48.png')
create_icon(128, 'icons/icon128.png')
print('Done!')
