import { defineConfig } from 'vite'
import path from 'path'
import react from '@vitejs/plugin-react'
import { crx } from '@crxjs/vite-plugin'
import manifest from './manifest.json'

export default defineConfig({
  plugins: [
    react(),
    crx({ manifest }),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src'),
      '@shared': path.resolve(__dirname, 'src/shared'),
      '@content': path.resolve(__dirname, 'src/content'),
      '@background': path.resolve(__dirname, 'src/background'),
    },
    dedupe: ['react', 'react-dom'],
  },
  build: {
    minify: false,
  },
  server: {
    port: 8082,
    cors: true,
    headers: { 'Access-Control-Allow-Origin': '*' },
  },
})
