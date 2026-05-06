"use strict";
/**
 * World Factory - Map Viewer Component
 *
 * Canvas-based map renderer for world visualization
 * Optimized for performance with render-ready polygon data
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.MapViewer = void 0;
class MapViewer {
    constructor(options) {
        this.mapData = null;
        this.viewport = { x: 0, y: 0, zoom: 1 };
        this.isDragging = false;
        this.lastMousePos = { x: 0, y: 0 };
        this.animationFrameId = null;
        this.canvas = options.canvas;
        this.mapData = options.mapData;
        this.onReady = options.onReady;
        this.onError = options.onError;
        const ctx = canvas.getContext('2d');
        if (!ctx) {
            this.onError?.(new Error('Could not get 2D context'));
            return;
        }
        this.ctx = ctx;
        this.setupEventListeners();
        this.render();
        this.onReady?.();
    }
    /**
     * Update map data and re-render
     */
    setMapData(data) {
        this.mapData = data;
        this.fitToWorld();
        this.render();
    }
    /**
     * Fit viewport to show entire map
     */
    fitToWorld() {
        if (!this.mapData)
            return;
        const { width, height } = this.mapData.dimensions;
        const scaleX = this.canvas.width / width;
        const scaleY = this.canvas.height / height;
        this.viewport.zoom = Math.min(scaleX, scaleY) * 0.9;
        this.viewport.x = (this.canvas.width - width * this.viewport.zoom) / 2;
        this.viewport.y = (this.canvas.height - height * this.viewport.zoom) / 2;
    }
    /**
     * Render the map to canvas
     */
    render() {
        if (this.animationFrameId !== null) {
            cancelAnimationFrame(this.animationFrameId);
        }
        this.animationFrameId = requestAnimationFrame(() => {
            this.drawMap();
            this.animationFrameId = null;
        });
    }
    drawMap() {
        const { ctx, canvas, viewport } = this;
        // Clear canvas
        ctx.fillStyle = '#1a1a2e';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        if (!this.mapData) {
            this.drawPlaceholder();
            return;
        }
        // Draw biomes (base layer)
        this.drawBiomes();
        // Draw polygons (territories)
        this.drawPolygons();
        // Draw resources
        this.drawResources();
        // Draw entities
        this.drawEntities();
    }
    drawBiomes() {
        if (!this.mapData)
            return;
        for (const biome of this.mapData.biomes) {
            const [r, g, b] = biome.color;
            this.ctx.fillStyle = `rgb(${r}, ${g}, ${b})`;
            // Create a simple grid representation based on biome type
            const gridSize = 20;
            for (let x = 0; x < this.mapData.dimensions.width; x += gridSize) {
                for (let y = 0; y < this.mapData.dimensions.height; y += gridSize) {
                    this.ctx.globalAlpha = 0.3;
                    this.ctx.fillRect(x, y, gridSize - 1, gridSize - 1);
                }
            }
        }
        this.ctx.globalAlpha = 1;
    }
    drawPolygons() {
        if (!this.mapData)
            return;
        for (const polygon of this.mapData.polygons) {
            if (polygon.vertices.length < 3)
                continue;
            this.ctx.beginPath();
            this.ctx.strokeStyle = this.getPolygonColor(polygon.type);
            this.ctx.lineWidth = polygon.type === 'territory' ? 2 : 1;
            const first = this.worldToScreen(polygon.vertices[0]);
            this.ctx.moveTo(first.x, first.y);
            for (let i = 1; i < polygon.vertices.length; i++) {
                const point = this.worldToScreen(polygon.vertices[i]);
                this.ctx.lineTo(point.x, point.y);
            }
            this.ctx.closePath();
            this.ctx.stroke();
            // Draw holes if present
            if (polygon.holes) {
                this.ctx.fillStyle = '#1a1a2e';
                for (const hole of polygon.holes) {
                    if (hole.length < 3)
                        continue;
                    this.ctx.beginPath();
                    const firstHole = this.worldToScreen(hole[0]);
                    this.ctx.moveTo(firstHole.x, firstHole.y);
                    for (let i = 1; i < hole.length; i++) {
                        const point = this.worldToScreen(hole[i]);
                        this.ctx.lineTo(point.x, point.y);
                    }
                    this.ctx.closePath();
                    this.ctx.fill();
                }
            }
        }
    }
    getPolygonColor(type) {
        const colors = {
            territory: '#ffd700',
            biome: '#00ff88',
            region: '#00aaff',
            resource: '#ff6b6b',
        };
        return colors[type] || '#ffffff';
    }
    drawResources() {
        if (!this.mapData)
            return;
        for (const resource of this.mapData.resources) {
            const pos = this.worldToScreen(resource.position);
            const radius = resource.magnitude * 4 + 4;
            // Draw resource indicator
            this.ctx.beginPath();
            this.ctx.arc(pos.x, pos.y, radius, 0, Math.PI * 2);
            this.ctx.fillStyle = this.getResourceColor(resource.type);
            this.ctx.fill();
            this.ctx.strokeStyle = '#ffffff';
            this.ctx.lineWidth = 1;
            this.ctx.stroke();
            // Draw label
            if (this.viewport.zoom > 0.5) {
                this.ctx.font = '10px sans-serif';
                this.ctx.fillStyle = '#ffffff';
                this.ctx.textAlign = 'center';
                this.ctx.fillText(resource.name, pos.x, pos.y - radius - 4);
            }
        }
    }
    getResourceColor(type) {
        const colors = {
            iron: '#8b4513',
            gold: '#ffd700',
            water: '#4169e1',
            wood: '#228b22',
            stone: '#808080',
            gems: '#9400d3',
        };
        return colors[type.toLowerCase()] || '#ffffff';
    }
    drawEntities() {
        if (!this.mapData)
            return;
        for (const entity of this.mapData.entities) {
            const pos = this.worldToScreen(entity.position);
            const size = Math.max(4, entity.significance * 2);
            // Draw entity marker
            this.ctx.beginPath();
            this.ctx.arc(pos.x, pos.y, size, 0, Math.PI * 2);
            this.ctx.fillStyle = this.getEntityColor(entity.type);
            this.ctx.fill();
            // Draw label
            this.ctx.font = '12px sans-serif';
            this.ctx.fillStyle = '#ffffff';
            this.ctx.textAlign = 'center';
            this.ctx.fillText(entity.name, pos.x, pos.y + size + 14);
        }
    }
    getEntityColor(type) {
        const colors = {
            city: '#ff4500',
            settlement: '#ff6b35',
            landmark: '#00ced1',
            fortress: '#8b0000',
        };
        return colors[type] || '#ffffff';
    }
    drawPlaceholder() {
        const { ctx, canvas } = this;
        ctx.font = '20px sans-serif';
        ctx.fillStyle = '#666666';
        ctx.textAlign = 'center';
        ctx.fillText('No map data loaded', canvas.width / 2, canvas.height / 2);
    }
    /**
     * Convert world coordinates to screen coordinates
     */
    worldToScreen(point) {
        return {
            x: point.x * this.viewport.zoom + this.viewport.x,
            y: point.y * this.viewport.zoom + this.viewport.y,
        };
    }
    /**
     * Convert screen coordinates to world coordinates
     */
    screenToWorld(point) {
        return {
            x: (point.x - this.viewport.x) / this.viewport.zoom,
            y: (point.y - this.viewport.y) / this.viewport.zoom,
        };
    }
    setupEventListeners() {
        // Mouse drag for panning
        this.canvas.addEventListener('mousedown', (e) => {
            this.isDragging = true;
            this.lastMousePos = { x: e.clientX, y: e.clientY };
        });
        this.canvas.addEventListener('mousemove', (e) => {
            if (this.isDragging) {
                const dx = e.clientX - this.lastMousePos.x;
                const dy = e.clientY - this.lastMousePos.y;
                this.viewport.x += dx;
                this.viewport.y += dy;
                this.lastMousePos = { x: e.clientX, y: e.clientY };
                this.render();
            }
        });
        this.canvas.addEventListener('mouseup', () => {
            this.isDragging = false;
        });
        this.canvas.addEventListener('mouseleave', () => {
            this.isDragging = false;
        });
        // Mouse wheel for zooming
        this.canvas.addEventListener('wheel', (e) => {
            e.preventDefault();
            const zoomFactor = e.deltaY > 0 ? 0.9 : 1.1;
            const mouseWorld = this.screenToWorld({ x: e.offsetX, y: e.offsetY });
            this.viewport.zoom *= zoomFactor;
            this.viewport.zoom = Math.max(0.1, Math.min(10, this.viewport.zoom));
            const newMouseScreen = this.worldToScreen(mouseWorld);
            this.viewport.x += e.offsetX - newMouseScreen.x;
            this.viewport.y += e.offsetY - newMouseScreen.y;
            this.render();
        });
        // Touch support
        this.canvas.addEventListener('touchstart', (e) => {
            if (e.touches.length === 1) {
                this.isDragging = true;
                this.lastMousePos = { x: e.touches[0].clientX, y: e.touches[0].clientY };
            }
        });
        this.canvas.addEventListener('touchmove', (e) => {
            if (this.isDragging && e.touches.length === 1) {
                e.preventDefault();
                const dx = e.touches[0].clientX - this.lastMousePos.x;
                const dy = e.touches[0].clientY - this.lastMousePos.y;
                this.viewport.x += dx;
                this.viewport.y += dy;
                this.lastMousePos = { x: e.touches[0].clientX, y: e.touches[0].clientY };
                this.render();
            }
        });
        this.canvas.addEventListener('touchend', () => {
            this.isDragging = false;
        });
    }
    /**
     * Cleanup resources
     */
    destroy() {
        if (this.animationFrameId !== null) {
            cancelAnimationFrame(this.animationFrameId);
        }
        // Note: Event listeners are not removed in this basic implementation
        // In production, use AbortController or store references for cleanup
    }
}
exports.MapViewer = MapViewer;
//# sourceMappingURL=MapViewer.js.map