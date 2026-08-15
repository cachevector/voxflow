/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./overlay.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        accent: "#6366f1",
        // Sampled from the VoxFlow logo (src/assets/logo.png).
        brand: {
          cyan: "#00b2c5",
          navy: "#192738",
        },
      },
      borderRadius: {
        pill: "9999px",
      },
    },
  },
  plugins: [],
};
