/**
 * 内容脚本 - 数据拦截器 (运行在MAIN world)
 * 同花顺通过 JSONP (<script> 标签) 加载数据，不走 XHR/fetch
 * 拦截策略:
 *   1. MutationObserver 监听 <script> 插入，捕获请求 URL
 *   2. 覆写 JSONP 回调函数，捕获响应数据
 *   3. 同时 hook fetch 作为补充（部分请求可能走 fetch）
 */
const fetchUrlSet = new Set();
(window as any).fetchUrlSet = fetchUrlSet;


const TARGET_JSONP_DOMAINS = ['d.10jqka.com.cn'];
const TARGET_FETCH_PATH = ['Funds/realFunds'];

function isTargetUrl(url: string): boolean {
  if (!url) return false;
  return TARGET_JSONP_DOMAINS.some((d) => url.includes(d));
}

function isFetchUrl(url: string): boolean {
  if(!url) return false;
  return TARGET_FETCH_PATH.some((d) => url.includes(d));
}

// ===================== JSONP 拦截 =====================

// 从 URL 中提取 JSONP 回调函数名
function extractCallbackName(url: string): string | null {
  // 常见模式: callback=xxx, cb=xxx, jsonp=xxx
  const match = url.match(/[?&](callback|cb|jsonp)=([^&]+)/);
  if (match) return match[2]!;
  return null;
}

// Hook: 监听 <script> 标签插入，拦截 JSONP 请求
const observer = new MutationObserver((mutations) => {
  for (const mutation of mutations) {
    for (const node of mutation.addedNodes) {
      if (node instanceof HTMLScriptElement && node.src && isTargetUrl(node.src)) {
        const url = node.src;
        const callbackName = extractCallbackName(url);

        if (callbackName) {
          // 有明确回调名：覆写全局回调
          hookJsonpCallback(callbackName, url);
        } else {
          // 无明确回调名（如 last.js）：通过覆写 document 写入拦截
          // 对于 .js 结尾的 JSONP，回调名通常在文件名或全局约定中
          // 采样：尝试在 script onload 时读取全局变量
          node.addEventListener('load', () => {
            // 检查常见的全局数据对象
            emitCapturedData(url);
          });
        }
      }
    }
  }
});

// Hook JSONP 回调函数
function hookJsonpCallback(callbackName: string, url: string) {
  // 保存原始回调（如果存在）
  const originalCallback = (window as any)[callbackName];

  (window as any)[callbackName] = function (data: any) {
    // 捕获数据
    window.postMessage(
      {
        type: 'ASTOCK_JSONP_DATA',
        url,
        response: JSON.stringify(data),
        status: 200,
      },
      '*',
    );

    // 调用原始回调
    if (typeof originalCallback === 'function') {
      originalCallback.call(window, data);
    }
  };
}

// script.onload 后尝试读取全局数据对象
async function emitCapturedData(url: string) {
  // 对于 .js JSONP 端点，直接用 fetch 获取响应体
  if (url.includes('.js')) {
    try {
      const resp = await fetch(url);
      if (resp.ok) {
        const body = await resp.text();
        window.postMessage(
          {
            type: 'ASTOCK_JSONP_DATA',
            url,
            response: body,
            status: 200,
          },
          '*',
        );
      }
    } catch {
      // fallback: 只发 URL
      window.postMessage(
        {
          type: 'ASTOCK_SCRIPT_LOAD',
          url,
          status: 200,
        },
        '*',
      );
    }
    return;
  }
  window.postMessage(
    {
      type: 'ASTOCK_SCRIPT_LOAD',
      url,
      status: 200,
    },
    '*',
  );
}

// 启动 MutationObserver
observer.observe(document.documentElement, {
  childList: true,
  subtree: true,
});

// ===================== 动态 <script> 拦截 =====================
// 同花顺可能通过 document.createElement('script') 动态创建
// 需要拦截 createElement 来捕获

const originalCreateElement = document.createElement.bind(document);
document.createElement = function (tagName: string, options?: ElementCreationOptions): HTMLElement {
  const element = originalCreateElement(tagName, options);

  if (tagName.toLowerCase() === 'script') {
    const script = element as HTMLScriptElement;
    const originalSrcSetter = Object.getOwnPropertyDescriptor(
      HTMLScriptElement.prototype, 'src',
    )?.set;

    if (originalSrcSetter) {
      Object.defineProperty(script, 'src', {
        set(value: string) {
          if (isTargetUrl(value)) {
            const callbackName = extractCallbackName(value);
            if (callbackName) {
              hookJsonpCallback(callbackName, value);
            }
            // 记录 URL，等 load 事件时捕获
            script.addEventListener('load', () => emitCapturedData(value));
          }
          originalSrcSetter.call(this, value);
        },
        get() {
          return Object.getOwnPropertyDescriptor(HTMLScriptElement.prototype, 'src')?.get?.call(this) || '';
        },
        configurable: true,
      });
    }
  }

  return element;
} as any;

// ===================== XHR 拦截 =====================
const _originalXhrOpen = XMLHttpRequest.prototype.open;
const _originalXhrSend = XMLHttpRequest.prototype.send;

XMLHttpRequest.prototype.open = function (method: string, url: string | URL, ...rest: any[]) {
  (this as any)._astockUrl = typeof url === 'string' ? url : url.href;
  return _originalXhrOpen.apply(this, [method, url, ...rest] as any);
};

XMLHttpRequest.prototype.send = function (...args: any[]) {
  const url = (this as any)._astockUrl;
  fetchUrlSet.add(url);
  if (url && isFetchUrl(url)) {
    this.addEventListener('load', () => {
      window.postMessage(
        {
          type: 'ASTOCK_FETCH_DATA',
          url,
          response: this.responseText,
          status: this.status,
        },
        '*',
      );
    });
  }
  return _originalXhrSend.apply(this, args);
};

// ===================== fetch 拦截 (补充) =====================
// 保存原始 fetch，用闭包保护不被覆盖

const _originalFetch = window.fetch.bind(window);

window.fetch = async function (input: RequestInfo | URL, init?: RequestInit) {
  const response = await _originalFetch(input, init);
  const url = typeof input === 'string' ? input : input instanceof URL ? input.href : input.url;
  fetchUrlSet.add(url);
  if (isFetchUrl(url)) {
    try {
      const cloned = response.clone();
      cloned.text().then((body) => {
        window.postMessage(
          {
            type: 'ASTOCK_FETCH_DATA',
            url,
            response: body,
            status: response.status,
          },
          '*',
        );
      });
    } catch {
      // ignore clone errors
    }
  }
  return response;
};

console.log('[astock] 拦截器已安装 (JSONP + fetch + XHR)');
