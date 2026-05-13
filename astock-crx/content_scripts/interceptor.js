/**
 * interceptor.js - 运行在同花顺页面的MAIN world中
 * Hook XHR和fetch，拦截同花顺数据API的响应
 */
(function () {
  'use strict';

  const TARGET_DOMAINS = [
    'd.10jqka.com.cn',
    'push2his.eastmoney.com',
    'push2.eastmoney.com',
    'qt.gtimg.cn',
  ];

  function isTargetUrl(url) {
    if (!url) return false;
    return TARGET_DOMAINS.some(d => url.includes(d));
  }

  // Hook XMLHttpRequest
  const OriginalXHR = window.XMLHttpRequest;

  class HookedXHR extends OriginalXHR {
    constructor() {
      super();
      this._hookUrl = '';
      this._hookMethod = '';
    }

    open(method, url, ...args) {
      this._hookUrl = url;
      this._hookMethod = method;
      return super.open(method, url, ...args);
    }

    send(...args) {
      this.addEventListener('load', function () {
        if (isTargetUrl(this._hookUrl)) {
          window.postMessage({
            type: 'ASTOCK_XHR_DATA',
            url: this._hookUrl,
            response: this.responseText,
            status: this.status,
          }, '*');
        }
      });
      return super.send(...args);
    }
  }

  window.XMLHttpRequest = HookedXHR;

  // Hook fetch
  const originalFetch = window.fetch;

  window.fetch = async function (input, init) {
    const response = await originalFetch.call(this, input, init);
    const url = typeof input === 'string' ? input : (input.url || '');

    if (isTargetUrl(url)) {
      try {
        const cloned = response.clone();
        cloned.text().then(body => {
          window.postMessage({
            type: 'ASTOCK_FETCH_DATA',
            url: url,
            response: body,
            status: response.status,
          }, '*');
        });
      } catch (e) {
        // ignore clone errors
      }
    }

    return response;
  };

  console.log('[astock] 拦截器已安装');
})();
