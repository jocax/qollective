import { fileURLToPath } from "node:url";
import { defineVitestConfig } from "@nuxt/test-utils/config";

export default defineVitestConfig({
	test: {
		globals: true,
		environment: "nuxt",
		environmentOptions: {
			nuxt: {
				rootDir: fileURLToPath(new URL(".", import.meta.url))
			}
		},
		setupFiles: ["./vitest.setup.ts"],

		// Optimize test execution with forks pool
		pool: "forks",
		poolOptions: {
			forks: {
				singleFork: false
			}
		},

		coverage: {
			provider: "v8",
			reporter: ["text", "json", "html"],
			exclude: [
				"node_modules/",
				"dist/",
				".nuxt/",
				"src-tauri/",
				"**/*.spec.ts",
				"**/__tests__/**"
			],
			thresholds: {
				statements: 80,
				branches: 80,
				functions: 80,
				lines: 80
			}
		}
	}
});
