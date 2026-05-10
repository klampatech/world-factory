// TypeScript declarations to fix compilation errors in e2e tests
// These augment the existing Playwright types

declare module '@playwright/test' {
  interface TestInfo {
    // Custom storage object for sharing state between tests
    // This is a pattern used in some test suites but not in standard Playwright types
    storage: Record<string, unknown>;
  }
}

// Extend the Route interface to include the delay method
declare module 'playwright' {
  interface Route {
    // Delay before continuing with the route (in ms)
    // This is not in standard Playwright types but was used in some test patterns
    delay(ms: number): Promise<void>;
  }
}

// Also extend the core types
declare module '@playwright/test' {
  interface Route {
    delay(ms: number): Promise<void>;
  }
}

export {};