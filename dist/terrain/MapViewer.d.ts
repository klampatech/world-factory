/**
 * World Factory - Map Viewer Component
 *
 * Canvas-based map renderer for world visualization
 * Optimized for performance with render-ready polygon data
 */
import type { MapData } from '../terrain/MapData';
export interface MapViewerOptions {
    canvas: HTMLCanvasElement;
    mapData: MapData | null;
    onReady?: () => void;
    onError?: (error: Error) => void;
}
export declare class MapViewer {
    private canvas;
    private ctx;
    private mapData;
    private viewport;
    private isDragging;
    private lastMousePos;
    private animationFrameId;
    private onReady?;
    private onError?;
    constructor(options: MapViewerOptions);
    /**
     * Update map data and re-render
     */
    setMapData(data: MapData): void;
    /**
     * Fit viewport to show entire map
     */
    fitToWorld(): void;
    /**
     * Render the map to canvas
     */
    render(): void;
    private drawMap;
    private drawBiomes;
    private drawPolygons;
    private getPolygonColor;
    private drawResources;
    private getResourceColor;
    private drawEntities;
    private getEntityColor;
    private drawPlaceholder;
    /**
     * Convert world coordinates to screen coordinates
     */
    private worldToScreen;
    /**
     * Convert screen coordinates to world coordinates
     */
    screenToWorld(point: {
        x: number;
        y: number;
    }): {
        x: number;
        y: number;
    };
    private setupEventListeners;
    /**
     * Cleanup resources
     */
    destroy(): void;
}
//# sourceMappingURL=MapViewer.d.ts.map