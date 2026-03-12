module.exports = {
  content: [
    "./crates/app/src/**/*.{rs,html}",
    "./crates/embed/src/**/*.{rs,html}"
  ],
  theme: {
    extend: {
      colors: {
        ink: "#101828",
        mist: "#f2f4f7",
        accent: "#16a34a"
      },
      fontFamily: {
        display: ["Sora", "sans-serif"],
        mono: ["JetBrains Mono", "monospace"]
      }
    }
  },
  plugins: []
};
