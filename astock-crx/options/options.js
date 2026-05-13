document.addEventListener('DOMContentLoaded', async () => {
  const result = await chrome.storage.local.get('serverUrl');
  document.getElementById('server-url').value = result.serverUrl || 'http://127.0.0.1:17320';

  document.getElementById('save-btn').addEventListener('click', async () => {
    const url = document.getElementById('server-url').value.trim();
    await chrome.storage.local.set({ serverUrl: url });
    const saved = document.getElementById('saved');
    saved.style.display = 'inline';
    setTimeout(() => { saved.style.display = 'none'; }, 2000);
  });
});
