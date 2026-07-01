/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}"],
  darkMode: ["class", '[data-theme="dark"]'],
  theme: {
    extend: {
      colors: {
        pc: {
          green: "#2ECC71",
          purple: "#9B59B6",
          dark: "var(--pc-dark)",
          panel: "var(--pc-panel)",
          border: "var(--pc-border)",
        },
        brand: {
          200: "#a7f3c7",
          300: "#6ee7a0",
          400: "#2ECC71",
          500: "#27ae60",
        },
        bg: {
          DEFAULT: "var(--bg)",
          card: "var(--bg-card)",
          soft: "var(--bg-soft)",
          border: "var(--bg-border)",
          hover: "var(--bg-hover)",
        },
      },
      fontFamily: {
        sans: ["Inter", "Segoe UI", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "Consolas", "monospace"],
      },
      boxShadow: {
        "glow-sm": "0 0 20px rgba(46, 204, 113, 0.15)",
        "glow-md": "0 0 40px rgba(46, 204, 113, 0.2)",
      },
      animation: {
        "fade-up": "fadeUp 0.7s ease-out both",
        "pulse-glow": "pulseGlow 3s ease-in-out infinite",
        float: "float 6s ease-in-out infinite",
      },
      keyframes: {
        fadeUp: {
          "0%": { opacity: "0", transform: "translateY(18px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        pulseGlow: {
          "0%, 100%": { boxShadow: "0 0 24px rgba(46,204,113,0.15)" },
          "50%": { boxShadow: "0 0 48px rgba(46,204,113,0.35)" },
        },
        float: {
          "0%, 100%": { transform: "translateY(0)" },
          "50%": { transform: "translateY(-8px)" },
        },
      },
    },
  },
  plugins: [],
};
