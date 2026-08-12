/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./overlay.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        accent: "#6366f1",
      },
      borderRadius: {
        pill: "9999px",
      },
    },
  },
  plugins: [],
};
