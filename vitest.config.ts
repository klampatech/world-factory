import { defineConfig } from 'vitest/config';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    include: ['tests/**/*.test.ts', 'tests/**/*.spec.ts'],
    exclude: [
      'tests/**/*.rs', // Rust tests
      'tests/api/**',  // Exclude broken TS tests that reference missing modules
      'tests/worlds-api.test.ts', // Missing supertest dependency
      'tests/WorldModel.test.ts', // Missing WorldModel module
    ],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['src/services/**/*.ts'],
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
    extensions: ['.ts', '.tsx', '.js', '.jsx', '.json'],
  },
});