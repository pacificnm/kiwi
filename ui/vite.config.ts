import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  resolve: {
    alias: {
      "@nest/components": path.resolve(__dirname, "./src/nest-components"),
    },
    dedupe: ["clsx", "tailwind-merge", "lucide-react", "react", "react-dom"],
  },
  optimizeDeps: {
    include: ["lucide-react", "clsx", "tailwind-merge"],
  },
});
