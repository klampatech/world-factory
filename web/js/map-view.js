/**
 * World Factory - Map View Module
 * Canvas-based rendering of world maps
 */

// ============================================================================
// Terrain Color Palette
// ============================================================================

const TERRAIN_COLORS = {
    'ocean': '#1e3a5f',
    'shallow_water': '#2d5a87',
    'beach': '#c2b280',
    'grassland': '#7cb342',
    'forest': '#2e7d32',
    'dense_forest': '#1b5e20',
    'desert': '#d4a574',
    'mountain': '#757575',
    'high_mountain': '#9e9e9e',
    'snow': '#ffffff',
    'tundra': '#b0bec5'
};

// ============================================================================
// Map Rendering
// ============================================================================

function renderMap(canvasId, mapData, options = {}) {
    const canvas = document.getElementById(canvasId);
    if (!canvas) return;
    
    const ctx = canvas.getContext('2d');
    const container = canvas.parentElement;
    
    // Set canvas size to container
    canvas.width = container.clientWidth;
    canvas.height = container.clientHeight;
    
    // Handle polygon data (Voronoi)
    if (mapData?.polygons && mapData.polygons.length > 0) {
        renderPolygonMap(ctx, canvas, mapData, options);
        return;
    }
    
    // Handle tile data (grid-based)
    if (!mapData || !mapData.tiles) {
        renderPlaceholder(ctx, canvas, 'No map data available');
        return;
    }
    
    renderTileMap(ctx, canvas, mapData, options);
}

function renderTileMap(ctx, canvas, mapData, options = {}) {
    const tiles = mapData.tiles;
    const tileWidth = canvas.width / mapData.width;
    const tileHeight = canvas.height / mapData.height;
    
    tiles.forEach(tile => {
        const x = tile.x * tileWidth;
        const y = tile.y * tileHeight;
        
        let color;
        if (tile.terrain && TERRAIN_COLORS[tile.terrain]) {
            color = TERRAIN_COLORS[tile.terrain];
        } else if (tile.elevation !== undefined) {
            color = elevationToColor(tile.elevation);
        } else {
            color = '#333333';
        }
        
        ctx.fillStyle = color;
        ctx.fillRect(x, y, tileWidth + 1, tileHeight + 1);
    });
    
    // Apply overlay if specified
    if (options.overlay === 'elevation') {
        renderElevationOverlay(ctx, canvas, mapData, tileWidth, tileHeight);
    } else if (options.overlay === 'resources') {
        renderResourceOverlay(ctx, canvas, mapData, tileWidth, tileHeight);
    } else if (options.overlay === 'political') {
        renderPoliticalOverlay(ctx, canvas, mapData, tileWidth, tileHeight);
    }
}

function renderPolygonMap(ctx, canvas, mapData, options = {}) {
    const polygons = mapData.polygons;
    const width = mapData.dimensions?.width || canvas.width;
    const height = mapData.dimensions?.height || canvas.height;
    
    const scaleX = canvas.width / width;
    const scaleY = canvas.height / height;
    
    polygons.forEach(polygon => {
        const vertices = polygon.vertices;
        if (!vertices || vertices.length < 3) return;
        
        const fillColor = polygonToColor(polygon, options);
        
        ctx.fillStyle = fillColor;
        ctx.strokeStyle = 'rgba(0, 0, 0, 0.1)';
        ctx.lineWidth = 1;
        
        ctx.beginPath();
        const firstV = vertices[0];
        ctx.moveTo(firstV.x * scaleX, firstV.y * scaleY);
        
        for (let i = 1; i < vertices.length; i++) {
            ctx.lineTo(vertices[i].x * scaleX, vertices[i].y * scaleY);
        }
        ctx.closePath();
        ctx.fill();
        ctx.stroke();
    });
    
    // Render hex grid overlay if enabled
    if (options.showHexGrid && mapData.width && mapData.height) {
        renderHexGridOverlay(ctx, canvas, mapData, scaleX, scaleY);
    }
    
    // Hover/highlight effect for selected polygon
    if (options.selectedPolygon) {
        highlightPolygon(ctx, options.selectedPolygon, scaleX, scaleY);
    }
}

/**
 * Render a hex grid overlay for visual verification
 */
function renderHexGridOverlay(ctx, canvas, mapData, scaleX, scaleY) {
    const hexSize = 20; // Size of each hex cell
    const cols = Math.ceil(canvas.width / (hexSize * 1.5)) + 1;
    const rows = Math.ceil(canvas.height / (hexSize * Math.sqrt(3))) + 1;
    
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.15)';
    ctx.lineWidth = 1;
    
    for (let row = 0; row < rows; row++) {
        for (let col = 0; col < cols; col++) {
            const x = col * hexSize * 1.5;
            const y = row * hexSize * Math.sqrt(3) + (col % 2 === 1 ? hexSize * Math.sqrt(3) / 2 : 0);
            
            drawHexagon(ctx, x, y, hexSize * 0.9);
        }
    }
}

function drawHexagon(ctx, x, y, size) {
    ctx.beginPath();
    for (let i = 0; i < 6; i++) {
        const angle = (Math.PI / 3) * i - Math.PI / 6;
        const hx = x + size * Math.cos(angle);
        const hy = y + size * Math.sin(angle);
        if (i === 0) {
            ctx.moveTo(hx, hy);
        } else {
            ctx.lineTo(hx, hy);
        }
    }
    ctx.closePath();
    ctx.stroke();
}

function polygonToColor(polygon, options = {}) {
    // Ocean
    if (polygon.is_ocean) {
        return TERRAIN_COLORS.ocean;
    }
    
    // Coastal
    if (polygon.is_coastal) {
        return TERRAIN_COLORS.beach;
    }
    
    // Elevation-based coloring
    if (polygon.elevation !== undefined) {
        return elevationToColor(polygon.elevation);
    }
    
    // Biome-based coloring
    if (polygon.biome) {
        return TERRAIN_COLORS[polygon.biome] || '#333333';
    }
    
    return '#555555';
}

function elevationToColor(elevation) {
    // Elevation bands from spec
    if (elevation < 0.2) {
        return TERRAIN_COLORS.ocean;
    } else if (elevation < 0.25) {
        return TERRAIN_COLORS.shallow_water;
    } else if (elevation < 0.3) {
        return TERRAIN_COLORS.beach;
    } else if (elevation < 0.5) {
        return TERRAIN_COLORS.grassland;
    } else if (elevation < 0.7) {
        return TERRAIN_COLORS.forest;
    } else if (elevation < 0.85) {
        return TERRAIN_COLORS.mountain;
    } else {
        return TERRAIN_COLORS.snow;
    }
}

function renderPlaceholder(ctx, canvas, message) {
    ctx.fillStyle = '#1a1a2e';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = '#ffffff';
    ctx.font = '16px sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText(message, canvas.width / 2, canvas.height / 2);
}

function highlightPolygon(ctx, polygon, scaleX, scaleY) {
    const vertices = polygon.vertices;
    if (!vertices || vertices.length < 3) return;
    
    ctx.strokeStyle = '#ffcc00';
    ctx.lineWidth = 3;
    
    ctx.beginPath();
    const firstV = vertices[0];
    ctx.moveTo(firstV.x * scaleX, firstV.y * scaleY);
    
    for (let i = 1; i < vertices.length; i++) {
        ctx.lineTo(vertices[i].x * scaleX, vertices[i].y * scaleY);
    }
    ctx.closePath();
    ctx.stroke();
}

function renderElevationOverlay(ctx, canvas, mapData, tileWidth, tileHeight) {
    const tiles = mapData.tiles;
    const alpha = 0.4;
    
    ctx.fillStyle = `rgba(0, 0, 0, ${alpha})`;
    
    tiles.forEach(tile => {
        if (tile.elevation !== undefined) {
            const x = tile.x * tileWidth;
            const y = tile.y * tileHeight;
            ctx.fillRect(x, y, tileWidth, tileHeight);
        }
    });
}

function renderResourceOverlay(ctx, canvas, mapData, tileWidth, tileHeight) {
    const tiles = mapData.tiles;
    
    tiles.forEach(tile => {
        if (tile.resources && tile.resources.length > 0) {
            const x = tile.x * tileWidth;
            const y = tile.y * tileHeight;
            
            // Draw resource indicator
            ctx.fillStyle = '#ffd700';
            ctx.beginPath();
            ctx.arc(x + tileWidth / 2, y + tileHeight / 2, tileWidth / 4, 0, Math.PI * 2);
            ctx.fill();
        }
    });
}

function renderPoliticalOverlay(ctx, canvas, mapData, tileWidth, tileHeight) {
    const tiles = mapData.tiles;
    const factionColors = ['#e74c3c', '#3498db', '#2ecc71', '#9b59b6', '#f39c12'];
    
    tiles.forEach(tile => {
        if (tile.faction_id !== undefined) {
            const x = tile.x * tileWidth;
            const y = tile.y * tileHeight;
            const colorIndex = tile.faction_id % factionColors.length;
            
            ctx.fillStyle = factionColors[colorIndex];
            ctx.globalAlpha = 0.3;
            ctx.fillRect(x, y, tileWidth, tileHeight);
            ctx.globalAlpha = 1;
        }
    });
}

// ============================================================================
// Interactive Map
// ============================================================================

class InteractiveMap {
    constructor(canvasId, mapData, options = {}) {
        this.canvas = document.getElementById(canvasId);
        this.ctx = this.canvas.getContext('2d');
        this.mapData = mapData;
        this.options = options;
        
        this.zoom = 1;
        this.panX = 0;
        this.panY = 0;
        this.isDragging = false;
        this.lastMouseX = 0;
        this.lastMouseY = 0;
        
        this.selectedPolygon = null;
        this.hoveredPolygon = null;
        this.onPolygonClick = options.onPolygonClick || null;
        this.onPolygonHover = options.onPolygonHover || null;
        
        this.init();
    }
    
    init() {
        this.resize();
        this.setupEventListeners();
        this.render();
    }
    
    resize() {
        const container = this.canvas.parentElement;
        this.canvas.width = container.clientWidth;
        this.canvas.height = container.clientHeight;
    }
    
    setupEventListeners() {
        // Mouse wheel for zoom
        this.canvas.addEventListener('wheel', (e) => {
            e.preventDefault();
            const delta = e.deltaY > 0 ? 0.9 : 1.1;
            this.zoom = Math.max(0.5, Math.min(4, this.zoom * delta));
            this.render();
        });
        
        // Mouse drag for pan
        this.canvas.addEventListener('mousedown', (e) => {
            this.isDragging = true;
            this.lastMouseX = e.clientX;
            this.lastMouseY = e.clientY;
            this.canvas.style.cursor = 'grabbing';
        });
        
        this.canvas.addEventListener('mousemove', (e) => {
            if (this.isDragging) {
                const dx = e.clientX - this.lastMouseX;
                const dy = e.clientY - this.lastMouseY;
                this.panX += dx;
                this.panY += dy;
                this.lastMouseX = e.clientX;
                this.lastMouseY = e.clientY;
                this.render();
            }
            
            // Polygon hover detection
            this.handleHover(e);
        });
        
        this.canvas.addEventListener('mouseup', () => {
            this.isDragging = false;
            this.canvas.style.cursor = 'crosshair';
        });
        
        this.canvas.addEventListener('mouseleave', () => {
            this.isDragging = false;
            this.hoveredPolygon = null;
            this.canvas.style.cursor = 'crosshair';
        });
        
        // Click for polygon selection
        this.canvas.addEventListener('click', (e) => {
            this.handleClick(e);
        });
        
        // Resize handler
        window.addEventListener('resize', () => {
            this.resize();
            this.render();
        });
    }
    
    handleHover(e) {
        if (!this.mapData?.polygons) return;
        
        const rect = this.canvas.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;
        
        const transformedX = (mouseX - this.panX) / this.zoom;
        const transformedY = (mouseY - this.panY) / this.zoom;
        
        let found = null;
        for (const polygon of this.mapData.polygons) {
            if (this.pointInPolygon(transformedX, transformedY, polygon.vertices)) {
                found = polygon;
                break;
            }
        }
        
        if (found !== this.hoveredPolygon) {
            this.hoveredPolygon = found;
            this.render();
            if (this.onPolygonHover && found) {
                this.onPolygonHover(found);
            }
        }
    }
    
    handleClick(e) {
        if (!this.mapData?.polygons || !this.onPolygonClick) return;
        
        const rect = this.canvas.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;
        
        const transformedX = (mouseX - this.panX) / this.zoom;
        const transformedY = (mouseY - this.panY) / this.zoom;
        
        for (const polygon of this.mapData.polygons) {
            if (this.pointInPolygon(transformedX, transformedY, polygon.vertices)) {
                this.selectedPolygon = polygon;
                this.render();
                this.onPolygonClick(polygon);
                return;
            }
        }
        
        this.selectedPolygon = null;
        this.render();
    }
    
    pointInPolygon(x, y, vertices) {
        let inside = false;
        for (let i = 0, j = vertices.length - 1; i < vertices.length; j = i++) {
            const xi = vertices[i].x, yi = vertices[i].y;
            const xj = vertices[j].x, yj = vertices[j].y;
            
            const intersect = ((yi > y) !== (yj > y)) &&
                (x < (xj - xi) * (y - yi) / (yj - yi) + xi);
            if (intersect) inside = !inside;
        }
        return inside;
    }
    
    render() {
        this.ctx.save();
        
        // Clear canvas
        this.ctx.fillStyle = '#1a1a2e';
        this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
        
        // Apply transformations
        this.ctx.translate(this.panX, this.panY);
        this.ctx.scale(this.zoom, this.zoom);
        
        // Render map
        renderMap(this.canvas.id, this.mapData, {
            ...this.options,
            selectedPolygon: this.selectedPolygon
        });
        
        this.ctx.restore();
    }
    
    setOverlay(overlayType) {
        this.options.overlay = overlayType;
        this.render();
    }
    
    resetView() {
        this.zoom = 1;
        this.panX = 0;
        this.panY = 0;
        this.selectedPolygon = null;
        this.render();
    }
    
    fitToView() {
        this.zoom = 1;
        this.panX = 0;
        this.panY = 0;
        this.render();
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

function hexToRgb(hex) {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16)
    } : { r: 0, g: 0, b: 0 };
}

function interpolateColor(color1, color2, factor) {
    const c1 = hexToRgb(color1);
    const c2 = hexToRgb(color2);
    const r = Math.round(c1.r + (c2.r - c1.r) * factor);
    const g = Math.round(c1.g + (c2.g - c1.g) * factor);
    const b = Math.round(c1.b + (c2.b - c1.b) * factor);
    return `rgb(${r}, ${g}, ${b})`;
}

function exportMapAsPng(canvasId) {
    const canvas = document.getElementById(canvasId);
    if (!canvas) return;
    
    const link = document.createElement('a');
    link.download = 'world-map.png';
    link.href = canvas.toDataURL('image/png');
    link.click();
}
