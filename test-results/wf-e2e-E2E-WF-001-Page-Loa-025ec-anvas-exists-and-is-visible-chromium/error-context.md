# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: wf-e2e.spec.ts >> E2E-WF-001: Page Load & Initialization >> E2E-WF-001.2: Map canvas exists and is visible
- Location: e2e/wf-e2e.spec.ts:50:7

# Error details

```
Test timeout of 30000ms exceeded.
```

# Page snapshot

```yaml
- generic [ref=e2]:
  - banner [ref=e3]:
    - generic [ref=e4]:
      - img [ref=e5]
      - generic [ref=e8]: World Factory
    - generic [ref=e9]:
      - button "Map" [ref=e10] [cursor=pointer]
      - button "Timeline" [ref=e11] [cursor=pointer]
    - generic [ref=e12]:
      - button "Reset View" [ref=e13] [cursor=pointer]
      - button "Generate World" [ref=e14] [cursor=pointer]
  - main [ref=e15]:
    - generic [ref=e16]:
      - heading "Biomes" [level=4] [ref=e21]
      - generic [ref=e22]:
        - button "Resources" [ref=e23] [cursor=pointer]:
          - img [ref=e24]
          - text: Resources
        - button "Elevation" [ref=e26] [cursor=pointer]:
          - img [ref=e27]
          - text: Elevation
        - button "Political" [ref=e29] [cursor=pointer]:
          - img [ref=e30]
          - text: Political
        - button "Wonders" [ref=e32] [cursor=pointer]:
          - img [ref=e33]
          - text: Wonders
```