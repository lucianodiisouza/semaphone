import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    projects: [
      {
        test: {
          name: "app",
          environment: "jsdom",
          include: ["src/**/*.test.ts"],
        },
      },
      {
        test: {
          name: "stream-deck",
          environment: "node",
          include: ["stream-deck/**/*.test.ts"],
        },
      },
    ],
  },
});
