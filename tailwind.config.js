/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./overlay.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        accent: {
          DEFAULT: "#6c5ce0",
          soft: "#8b7ff0",
        },
      },
      borderRadius: {
        pill: "999px",
      },
    },
  },
  plugins: [],
};
