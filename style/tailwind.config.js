module.exports = {
  content: [
    "./crates/app/src/**/*.{rs,html}",
    "./crates/embed/src/**/*.{rs,html}"
  ],
  theme: {
    extend: {
      colors: {
        ink: "#f5f5f5",
        void: "#090909",
        panel: "#111111",
        line: "#2a2a2a"
      },
      fontFamily: {
        display: ["Space Grotesk", "sans-serif"],
        mono: ["IBM Plex Mono", "monospace"]
      }
    }
  },
  plugins: []
};
