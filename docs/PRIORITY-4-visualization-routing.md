# Priority Fix 4: Phase 4 Visualization — Multi-Page Routing

> **Issue:** `web/` contains two single-page apps (`index.html` 94KB, `world.html` 72KB) with no
> routing, no landing page, no dashboard, and no server-side HTML serving
> **Severity:** MEDIUM — SPEC.md §6 (Visualization) requires a multi-page SPA with routing
> **Reference:** SPEC.md §6, §7.1

---

## Current State

The `web/` directory contains:
- `index.html` (94KB) — standalone map viewer with hardcoded `localhost:8080` API URL
- `world.html` (72KB) — second viewer, possibly for different views
- `api-integration.js` (20KB) — shared API client code
- `js/` folder with JavaScript modules
- `css/` folder

**What's missing:**
- Landing page (`GET /`) — world selector with list of all worlds
- World overview page (`GET /worlds/:id`)
- Server-side routing (all routes served as HTML)
- Dashboard view with charts
- PNG export functionality
- Create world form on landing page

---

## Required Implementation

### 1. Server-Side HTML Routes (Axum)

Add these routes to the HTTP server:

```rust
// Serve the SPA for all /worlds/* routes (client-side routing)
Router::new()
    .route("/", get(serve_landing_page))
    .route("/worlds/:id", get(serve_world_overview))
    .route("/worlds/:id/map", get(serve_world_map))
    .route("/worlds/:id/timeline", get(serve_world_timeline))
    .route("/worlds/:id/dashboard", get(serve_world_dashboard))
```

Each route returns the **same HTML shell** with a different `<div id="app">` or `<script>` data attribute
to indicate which view to render. The server serves the SPA, and the client reads the URL to
determine which view to show.

### 2. Landing Page (`GET /`)

Returns HTML with:
- **Header:** "World Factory" with server status badge
- **Generate New World button** → opens modal form
- **World list cards** showing:
  - World name, ID, status (generating/ready/failed)
  - Progress bar for worlds still generating
  - "View Map", "Timeline", "Dashboard" buttons

**Generate New World Modal fields:**
- Name (required)
- Width/Height sliders (default: 64x64, max: 128)
- Pre-history years (default: 100, range: 0-1000)
- Seed (optional, auto-generated)
- Species selection (Human, Elf, Dwarf, Orc, Halfling checkboxes)
- Resource richness (Poor/Normal/Rich/Abundant dropdown)
- Disaster frequency (Low/Medium/High dropdown)
- Submit → `POST /api/v1/worlds` → on success redirect to `/worlds/:id`

**API calls needed:**
- `GET /api/v1/worlds` — list all worlds with status
- `POST /api/v1/worlds` — create new world
- `GET /api/v1/worlds/:id` — poll for generation status

### 3. World Overview Page (`GET /worlds/:id`)

Returns HTML with:
- World metadata (name, seed, dimensions, creation date, current year)
- Status with progress bar (if still generating)
- Tabs/navigation: Overview | Map | Timeline | Dashboard
- "Delete World" button

### 4. Map View (`GET /worlds/:id/map`)

Returns HTML that mounts the canvas map:
- Existing `index.html` canvas logic can be reused
- Full viewport canvas
- Overlay toggles: Resources | Political | Elevation
- Zoom controls: Fit | 50% | 100% | 200%
- Pan via drag
- Click polygon → details sidebar
- Mini-map in corner
- **PNG Export button** — `canvas.toDataURL('image/png')` → download

### 5. Timeline View (`GET /worlds/:id/timeline`)

Returns HTML with:
- Vertical timeline of events
- Filter bar: All | War | Settlement | Discovery | Plague | etc.
- Search by figure or place name
- Click event → expand details (year, description, participants)
- Click figure → biography popup

**API call:** `GET /api/v1/worlds/:id/history` (paginated events)

### 6. Dashboard View (`GET /worlds/:id/dashboard`)

Returns HTML with:
- Current year display (large, prominent)
- Active disasters count with icons
- Population totals by species (pie chart via Chart.js or similar)
- Resource summary (bar chart)
- Notable figures spotlight (top 5 by impact score)
- Recent events list (last 10)

**API calls:**
- `GET /api/v1/worlds/:id` — current year, active disasters
- `GET /api/v1/worlds/:id/figures` — notable figures
- `GET /api/v1/worlds/:id/events` — recent events

### 7. Global Navigation

Shared header across all pages:
```
┌─────────────────────────────────────────────────────────────┐
│  🌍 WF  │ World Selector │ Map: [World] │ Timeline │ Dash   │
└─────────────────────────────────────────────────────────────┘
```

- "World Selector" → `GET /`
- "Map/Timeline/Dash" links for current world
- World name in header → `GET /worlds/:id`

---

## Implementation Approach

**Two options — pick one:**

### Option A: Server-Side Page Templates (Recommended)

Create HTML template files in `web/templates/` and serve them via Axum routes.
Axum returns the HTML directly; JavaScript on the client reads URL params to
determine which components to mount.

```
web/
├── templates/
│   ├── landing.html      # Landing page template
│   ├── world.html        # World overview + tabs
│   ├── map.html          # Map view
│   ├── timeline.html     # Timeline view
│   └── dashboard.html    # Dashboard view
└── js/
    ├── app.js            # Main SPA entry
    ├── components/       # Reusable UI components
    └── api.js            # API client
```

### Option B: Single Page App with Hash Routing

Keep `index.html` as the shell and use hash-based routing (`#worlds/:id/map`).
The server only needs to serve one HTML file for all routes.

```
GET /                     → index.html
GET /worlds/:id/*         → index.html (SPA handles routing)
```

---

## Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|--------------|
| 1 | `GET /` returns landing page HTML with world list | `curl http://localhost:8080/` |
| 2 | `GET /worlds/:id` returns world overview HTML | `curl http://localhost:8080/worlds/<id>` |
| 3 | `GET /worlds/:id/map` returns map view HTML | `curl http://localhost:8080/worlds/<id>/map` |
| 4 | `GET /worlds/:id/timeline` returns timeline HTML | `curl http://localhost:8080/worlds/<id>/timeline` |
| 5 | `GET /worlds/:id/dashboard` returns dashboard HTML | `curl http://localhost:8080/worlds/<id>/dashboard` |
| 6 | Landing page lists all worlds from `GET /api/v1/worlds` | Browser: worlds appear as cards |
| 7 | "Generate New World" form creates a world via `POST /api/v1/worlds` | Submit form → new world appears |
| 8 | Polling works for in-progress worlds (status + progress) | Create large world, watch progress bar |
| 9 | Map canvas renders correctly with zoom/pan | Visual inspection |
| 10 | PNG export button downloads map as image | Click export → file downloads |
| 11 | Timeline shows events with filtering | Apply filters, verify events change |
| 12 | Dashboard shows population pie chart, resource bar chart | Visual inspection |
| 13 | No hardcoded `localhost:8080` in served HTML | All API URLs relative or configurable |
| 14 | `cargo test --lib` still passes | No regression |

---

## Key Files

- `src/main.rs` — add new Axum routes for HTML serving
- `web/index.html` — existing canvas viewer (reuse canvas rendering logic)
- `web/api-integration.js` — existing API client
- `web/js/` — existing JavaScript modules

---

## Notes

- The existing canvas rendering in `index.html` already works — extract and reuse it
- Use `Chart.js` (CDN) for dashboard charts — no need to build your own
- API base URL should be relative (`/api`) not hardcoded, so it works in any environment
- The server is Axum-based — see existing route definitions in `src/api/v1/` for patterns
- Consider serving static assets (JS, CSS, images) via Axum's static file middleware or a simple file server