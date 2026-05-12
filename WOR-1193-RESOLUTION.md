# WOR-1193: Map Canvas Element Renders But Has No Dimensions

## Issue Summary
The map canvas element (`#world-map`) was rendering but had zero dimensions when the Map tab was first displayed.

## Root Cause
1. **CSS issue**: The `.map-container` used `aspect-ratio: 16/9` which requires a definite width to compute height. In the tab panel hierarchy, the container's width wasn't being properly resolved, resulting in zero dimensions.

2. **JavaScript timing issue**: The `renderMap()` function was called immediately when switching tabs, before the browser had completed layout calculations for the now-visible container.

## Fix Applied

### 1. CSS Fix (line ~598 in web/index.html)
Changed `.map-container` from using `aspect-ratio` to fixed dimensions:
```css
/* Before */
.map-container {
    aspect-ratio: 16/9;
}

/* After */
.map-container {
    height: 400px;
    min-height: 300px;
}
```

### 2. JavaScript Fix (renderMap function)
- Wrapped render logic in `requestAnimationFrame()` to wait for layout completion
- Added dimension validation with retry logic:
```javascript
if (width <= 0 || height <= 0) {
    console.warn('Map container has no dimensions, will retry...');
    setTimeout(renderMap, 100);
    return;
}
```

## Verification

**Test Results:**
- Canvas dimensions: 1232 x 400 ✅
- Container dimensions: 1232 x 400 ✅
- Canvas renders content correctly ✅

**Playwright test output:**
```
Canvas found: world-map
Canvas dimensions: 1232 x 400
✅ Canvas has valid dimensions!
Container dimensions: 1232 x 400
```

## Files Modified
- `web/index.html` - CSS and JavaScript fixes
- `web/dist/index.html` - Copied updated file for distribution

## Status
✅ **RESOLVED** - Canvas now renders with valid dimensions

## Related Issues
- Parent: WOR-1190 - Map rendering issues