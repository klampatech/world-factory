WOR-667: Add figure detail route handler
========================================

Status: IMPLEMENTED

## Summary

Added a new API endpoint GET /api/v1/figures/:id to retrieve detailed information
about historical figures including their full biography, achievements, and lifespan.

## Changes Made

### 1. api/v1/worlds.rs - Figure detail types and registry

Added:
- `FigureDetail` struct with fields: id, name, figure_type, species, birth_year,
  death_year, significance, description, achievements, related_world_id
- `FIGURE_REGISTRY` - in-memory HashMap storing all registered figures
- `register_figure(figure: FigureDetail)` - register a figure in the registry
- `get_figure(id: &str) -> Option<FigureDetail>` - retrieve figure by ID
- `list_figures() -> Vec<FigureDetail>` - list all registered figures

### 2. api/v1/mod.rs - Re-exports

Exported the new figure functions:
- FigureDetail, register_figure, get_figure, list_figures

### 3. main.rs - HTTP handler and route

Added:
- `get_figure_handler` async function handling GET /api/v1/figures/:id
- Route registration: .route("/api/v1/figures/:id", get(get_figure_handler))

## API Response Format

GET /api/v1/figures/:id

Success (200):
{
  "data": {
    "id": "...",
    "name": "Aldric the Wise",
    "figure_type": "Warrior",
    "species": "Homo Sapiens",
    "birth_year": 423,
    "death_year": 489,
    "significance": 0.95,
    "description": "First king of the northern tribes...",
    "achievements": ["United the seven tribes", "Founded Ironforge"],
    "related_world_id": "world-123"
  }
}

Not Found (404):
{
  "data": null,
  "error": "Figure not found"
}

## Notes

- The route follows the pattern of existing world-scoped routes
- Returns 404 if figure not found
- Registry is in-memory; figures need to be registered during world generation
- Frontend integration (calling this endpoint from showFigureBiography) is a 
  separate enhancement that can be implemented to replace the current local-only
  figure lookup behavior

## Test Scenarios

1. GET /api/v1/figures/nonexistent-id → 404
2. Register a figure, then GET /api/v1/figures/:id → 200 with figure data
3. List figures shows all registered figures

## Verification

Build command (requires Rust toolchain):
  cd web && cargo build

Run tests:
  cd web && cargo test