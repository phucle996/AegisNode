import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

// Cấu hình Vite cho AegisNode Web UI
export default defineConfig({
  plugins: [
    react(),
    tailwindcss(), // Sử dụng @tailwindcss/vite plugin thay vì postcss
  ],
  resolve: {
    alias: {
      // Alias "@" trỏ về src/ để import ngắn gọn: import { cn } from "@/lib/utils"
      '@': `${import.meta.dirname}/src`,
    },
  },
  server: {
    port: 5173,
    // Proxy API requests tới AegisNode backend trong dev mode
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
  },
})
