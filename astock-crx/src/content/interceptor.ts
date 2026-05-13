/**
 * 内容脚本 - XHR/Fetch拦截器 (运行在MAIN world)
 * 在页面上下文中hook网络请求，捕获同花顺数据API响应
 */
import {isTargetUrl, TARGET_DOMAINS} from '@shared/constants';



// Hook XMLHttpRequest
const OriginalXHR = window.XMLHttpRequest;

class HookedXHR extends OriginalXHR {
  _hookUrl = '';
  _hookMethod = '';

  open(method: string, url: string | URL, ...args: any[]) {
    debugger;
    this._hookUrl = url.toString();
    this._hookMethod = method;
    // @ts-ignore - XHR.open overloads
    return super.open(method, url, ...args);
  }

  send(...args: any[]) {
    debugger;
    this.addEventListener('load', function (this: HookedXHR) {
      if (isTargetUrl(this._hookUrl)) {
        window.postMessage(
          {
            type: 'ASTOCK_XHR_DATA',
            url: this._hookUrl,
            response: this.responseText,
            status: this.status,
          },
          '*',
        );
      }
    });
    return super.send(...args);
  }
}

(window as any).XMLHttpRequest = HookedXHR;

// Hook fetch
const originalFetch = window.fetch;

window.fetch = async function (input: RequestInfo | URL, init?: RequestInit) {
  const response = await originalFetch.call(this, input, init);
  const url = typeof input === 'string' ? input : input instanceof URL ? input.href : input.url;
  debugger;
  if (isTargetUrl(url)) {
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

console.log('[astock] 拦截器已安装');
