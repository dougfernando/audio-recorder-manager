import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [svelte()],

  // Tauri expects a fixed port
  server: {
    port: 5173,
    strictPort: true,
  },

  // For production build
  build: {
    // Tauri uses Chromium on Windows
    target: 'chrome105',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },

  // Prevent vite from obscuring rust errors
  clearScreen: false,
})
