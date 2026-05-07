# WOR-81: World Selector Landing Page (Section 6.0)

## Status: ✅ Complete

**Created:** `web/selector.html` — World Selector Landing Page

---

## What Was Built

### Core Features

| Feature | Description |
|---------|-------------|
| **Hero Section** | Title "Choose Your World" with description |
| **Stats Bar** | Total/Ready/Generating world counts |
| **Filter Tabs** | All, Ready, Generating, Failed |
| **World Grid** | Cards with name, date, status badge, progress bar |
| **Create Modal** | Full form with all parameters per spec |

### Create World Form Fields (per Section 6.0)

| Field | Type | Details |
|-------|------|---------|
| World Name | Text input | Required, max 50 chars |
| Width | Range slider | 16-128, step 8, default 64 |
| Height | Range slider | 16-128, step 8, default 64 |
| Pre-history Years | Range slider | 0-10000, step 100, default 1000 |
| Seed | Number input | Optional, for reproducibility |
| Resource Richness | Dropdown | Poor/Normal/Rich/Abundant, default Normal |
| Disaster Frequency | Dropdown | Low/Medium/High, default Medium |
| Species | Multi-select | 8 species with checkboxes and tags |

### Technical Implementation

- **API Integration:**
  - `GET /api/worlds` — fetches worlds list
  - `POST /api/worlds` — creates new world with all parameters
- **Graceful fallback:** Demo data (3 sample worlds) when API unavailable
- **Auto-refresh:** Polls every 10 seconds for generating world updates
- **Navigation:** Redirects to `web/index.html?world={id}` when ready world clicked
- **Responsive design:** Mobile-friendly with media queries
- **Keyboard support:** Escape closes modal, Enter submits form

### API Contract

```typescript
// POST /api/worlds
Request: {
  name: string,           // required
  width?: number,         // default 64
  height?: number,        // default 64
  preHistoryYears?: number, // default 1000
  seed?: number,          // optional
  species?: string[],     // optional
  resourceRichness?: "poor" | "normal" | "rich" | "abundant",
  disasterFrequency?: "low" | "medium" | "high"
}

Response: {
  success: true,
  data: {
    id: string,
    name: string,
    status: "ready" | "generating",
    progress: number,
    createdAt: string
  }
}
```

### Files Changed

```
web/
├── index.html      # Main world viewer (existing)
└── selector.html  # World selector landing page (ENHANCED)
```

---

## Testing Checklist

- [x] Page loads at `/web/selector.html`
- [x] Hero section displays correctly
- [x] Stats bar shows correct counts
- [x] Filter tabs work correctly
- [x] World cards display properly
- [x] Status badges show correct colors (green=ready, yellow=generating, red=failed)
- [x] Progress bars animate for generating worlds
- [x] Create modal opens with full form
- [x] Width/Height sliders work (16-128 range)
- [x] Pre-history slider works (0-10000 range)
- [x] Resource richness dropdown works
- [x] Disaster frequency dropdown works
- [x] Species multi-select with checkboxes works
- [x] Species tags removable
- [x] Create world sends all parameters to API
- [x] Modal resets on close
- [x] Navigation to viewer after world creation
- [x] API fallback with demo data when server unavailable
- [x] Auto-refresh for generating worlds

---

## Browser Testing

Access the page at: `http://localhost:4321/web/selector.html`

The page will:
1. Try to fetch worlds from `GET /api/worlds`
2. Fall back to demo data if API unavailable
3. Allow creating new worlds via modal with full spec parameters
4. Navigate to viewer when ready world is clicked