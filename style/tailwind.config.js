module.exports = {
  content: [
    "./crates/app/src/**/*.{rs,html}",
    "./crates/embed/src/**/*.{rs,html}"
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: "var(--brand)",
          dim: "var(--brand-hover)"
        },
        background: "var(--bg)",
        surface: "var(--panel-bg)",
        "surface-container-lowest": "var(--surface-container-lowest)",
        "surface-container-low": "var(--surface-container-low)",
        "surface-container": "var(--surface-container)",
        "surface-container-high": "var(--surface-container-high)",
        "surface-container-highest": "var(--surface-container-highest)",
        outline: "var(--border)",
        "outline-variant": "var(--outline-variant)",
        "on-surface": "var(--ink)",
        "on-surface-variant": "var(--muted)",
        "on-primary": "var(--on-primary)",
        "secondary-container": "var(--secondary-container)",
        border: "var(--border)",
        muted: "var(--muted)",
        foreground: "var(--ink)",
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
        body: ["Inter", "system-ui", "sans-serif"],
        headline: ["Space Grotesk", "Inter", "system-ui", "sans-serif"],
        label: ["Inter", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "Fira Code", "monospace"]
      }
    }
  },
  plugins: []
};
