module.exports = {
  content: [
    "./crates/app/src/**/*.{rs,html}",
    "./crates/embed/src/**/*.{rs,html}"
  ],
  theme: {
    extend: {
      colors: {
        background: "#09090b",
        surface: "#18181b",
        border: "#27272a",
        muted: "#a1a1aa",
        foreground: "#f4f4f5",
        brand: {
          DEFAULT: "#f4f4f5",
          hover: "#d4d4d8"
        },
        danger: {
          DEFAULT: "#ef4444",
          hover: "#dc2626",
          bg: "#7f1d1d"
        },
        terminal: {
          bg: "#050505",
          panel: "#111111",
          ink: "#4ade80",
          dim: "#166534",
          muted: "#a3a3a3",
          danger: "#ef4444",
          border: "#14532d"
        }
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
        mono: ["Fira Code", "JetBrains Mono", "monospace"]
      }
    }
  },
  plugins: []
};
