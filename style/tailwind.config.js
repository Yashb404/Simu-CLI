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
        },
        homepage: {
          primary: "#4ae176",
          "primary-dim": "#38d36a",
          "primary-fixed": "#6bff8f",
          "primary-fixed-dim": "#5bf083",
          "primary-container": "#005321",
          background: "#0e0e10",
          surface: "#0e0e10",
          "surface-bright": "#2b2c32",
          "surface-container-lowest": "#000000",
          "surface-container-low": "#131316",
          "surface-container": "#19191d",
          "surface-container-high": "#1f1f24",
          "surface-container-highest": "#25252b",
          "surface-dim": "#0e0e10",
          "surface-tint": "#4ae176",
          outline: "#75757c",
          "outline-variant": "#47474e",
          "on-background": "#e7e4ec",
          "on-surface": "#e7e4ec",
          "on-surface-variant": "#acaab1",
          "on-primary": "#004b1e",
          "on-primary-container": "#56eb7f",
          "on-primary-fixed": "#004a1d",
          "on-primary-fixed-variant": "#006a2d",
          "secondary": "#9f9da1",
          "secondary-dim": "#9f9da1",
          "secondary-container": "#3b3b3e",
          "secondary-fixed": "#e4e1e5",
          "secondary-fixed-dim": "#d6d3d7",
          "on-secondary": "#202023",
          "on-secondary-fixed": "#3f3f42",
          "on-secondary-fixed-variant": "#5c5b5e",
          "on-secondary-container": "#c1bec2",
          tertiary: "#e9ffed",
          "tertiary-dim": "#cdedd6",
          "tertiary-container": "#d3f3dc",
          "tertiary-fixed": "#dbfce4",
          "tertiary-fixed-dim": "#cdedd6",
          "on-tertiary": "#496553",
          "on-tertiary-container": "#415d4b",
          "on-tertiary-fixed": "#34503f",
          "on-tertiary-fixed-variant": "#506d5a",
          error: "#ed7f64",
          "error-dim": "#ba573f",
          "error-container": "#7e2b17",
          "on-error": "#450900",
          "on-error-container": "#ff9b82",
          "inverse-primary": "#006e2f",
          "inverse-surface": "#fcf8fb",
          "inverse-on-surface": "#565457"
        }
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
        body: ["Inter", "system-ui", "sans-serif"],
        headline: ["Inter", "system-ui", "sans-serif"],
        label: ["Space Grotesk", "Inter", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "Fira Code", "monospace"]
      }
    }
  },
  plugins: []
};
