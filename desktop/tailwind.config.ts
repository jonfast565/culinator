import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{vue,ts}"],
  theme: {
    extend: {
      colors: {
        canvas: "#f3f1eb",
        ink: "#1f2925",
        herb: "#38634f",
        cream: "#fffdf8",
      },
    },
  },
  plugins: [],
} satisfies Config;
