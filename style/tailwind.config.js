module.exports = {
  content: [
    "./crates/app/src/**/*.{rs,html}",
    "./crates/embed/src/**/*.{rs,html}"
  ],
  theme: {
    extend: {
      colors: {
        bg:     "#0a0a0a",
        panel:  "#0e0e0e",
        ink:    "#00ff41",
        "ink-dim": "#00c832",
        muted:  "#1a6e2e",
        amber:  "#ffb000",
        danger: "#ff3333",
        border: "rgba(0,255,65,0.25)"
      },
      fontFamily: {
        display: ["JetBrains Mono", "monospace"],
        mono:    ["IBM Plex Mono", "monospace"]
      }
    }
  },
  plugins: []
};
