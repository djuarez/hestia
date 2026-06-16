import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// In dev, proxy the API (and the log WebSocket) to a running hestia-server,
// so the UI can use same-origin `/v1/...` paths.
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,
    proxy: {
      "/v1": {
        target: "http://127.0.0.1:4300",
        changeOrigin: true,
        ws: true,
      },
    },
  },
});
