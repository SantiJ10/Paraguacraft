/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        // Sistema de marca Paraguacraft
        pc: {
          green: "#2ECC71",
          "green-dark": "#1A7A40",
          "green-hover": "#27AE60",
          ai: "#9B59B6",
          "ai-dark": "#7D3C98",
        },
        // Superficies vía CSS vars (rgb channels) para soportar /opacity
        surface: {
          0: "rgb(var(--surface-0) / <alpha-value>)",
          1: "rgb(var(--surface-1) / <alpha-value>)",
          2: "rgb(var(--surface-2) / <alpha-value>)",
          3: "rgb(var(--surface-3) / <alpha-value>)",
          4: "rgb(var(--surface-4) / <alpha-value>)",
          5: "rgb(var(--surface-5) / <alpha-value>)",
          6: "rgb(var(--surface-6) / <alpha-value>)",
        },
      },
      fontFamily: {
        sans: [
          "Inter",
          "Segoe UI",
          "Tahoma",
          "Geneva",
          "Verdana",
          "sans-serif",
        ],
        emoji: [
          "Segoe UI Emoji",
          "Apple Color Emoji",
          "Noto Color Emoji",
          "sans-serif",
        ],
      },
      boxShadow: {
        glow: "0 10px 25px -5px rgba(46, 204, 113, 0.25)",
        "glow-ai": "0 10px 25px -5px rgba(155, 89, 182, 0.25)",
      },
      keyframes: {
        wizFwd: {
          "0%": { opacity: "0", transform: "translateX(20px)" },
          "100%": { opacity: "1", transform: "translateX(0)" },
        },
        wizBack: {
          "0%": { opacity: "0", transform: "translateX(-20px)" },
          "100%": { opacity: "1", transform: "translateX(0)" },
        },
        fadeIn: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        pulseDot: {
          "0%, 100%": { opacity: "1" },
          "50%": { opacity: "0.3" },
        },
      },
      animation: {
        "wiz-fwd": "wizFwd 0.28s ease-out both",
        "wiz-back": "wizBack 0.28s ease-out both",
        "fade-in": "fadeIn 0.3s ease-out both",
        "pulse-dot": "pulseDot 1.2s ease-in-out infinite",
      },
    },
  },
  plugins: [],
};
