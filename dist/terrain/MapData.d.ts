/**
 * World Factory - Map Data Types
 *
 * Type definitions for GET /api/worlds/:id/map endpoint
 * These interfaces define the render-ready map data contract
 */
/**
 * Represents a 2D point on the map
 */
export interface MapPoint {
    x: number;
    y: number;
}
/**
 * Polygon boundary for a map region (territory, biome area, etc.)
 */
export interface MapPolygon {
    /** Unique identifier for this polygon */
    id: string;
    /** Type of polygon: 'territory' | 'biome' | 'region' | 'resource' */
    type: 'territory' | 'biome' | 'region' | 'resource';
    /** Array of vertices forming the polygon (at least 3 points) */
    vertices: MapPoint[];
    /** Optional holes within this polygon */
    holes?: MapPoint[][];
}
/**
 * Biome classification with associated color for rendering
 */
export interface Biome {
    /** Unique identifier for this biome */
    id: string;
    /** Biome type name (e.g., 'forest', 'desert', 'ocean', 'mountain') */
    type: string;
    /** RGB color values for canvas rendering [r, g, b] */
    color: [number, number, number];
    /** Human-readable name */
    name: string;
}
/**
 * Resource deposit or point of interest on the map
 */
export interface ResourceLocation {
    /** Unique identifier */
    id: string;
    /** Resource type (e.g., 'iron', 'gold', 'water', 'wood') */
    type: string;
    /** Center position of the resource */
    position: MapPoint;
    /** Size/radius indicator (1-5 scale) */
    magnitude: number;
    /** Human-readable label */
    name: string;
}
/**
 * Geographic entity (city, settlement, landmark)
 */
export interface GeographicEntity {
    /** Unique identifier */
    id: string;
    /** Entity type: 'city' | 'settlement' | 'landmark' | 'fortress' */
    type: 'city' | 'settlement' | 'landmark' | 'fortress';
    /** Center position */
    position: MapPoint;
    /** Display name */
    name: string;
    /** Population or importance indicator */
    significance: number;
}
/**
 * Complete render-ready map data structure
 * Optimized for canvas rendering - no computed styles or layout properties
 */
export interface MapData {
    /** World ID this map represents */
    worldId: string;
    /** Map dimensions */
    dimensions: {
        width: number;
        height: number;
    };
    /** Map scale (pixels per world unit) */
    scale: number;
    /** Polygon boundaries for regions/territories */
    polygons: MapPolygon[];
    /** Biome classifications with colors */
    biomes: Biome[];
    /** Resource deposit locations */
    resources: ResourceLocation[];
    /** Geographic entities */
    entities: GeographicEntity[];
    /** Optional: terrain height data as grid (for elevation shading) */
    elevationGrid?: number[][];
    /** Metadata */
    metadata: {
        /** Last generated/updated timestamp */
        generatedAt: string;
        /** Map version for caching */
        version: string;
    };
}
/**
 * API response wrapper for map data
 */
export interface MapDataResponse {
    success: boolean;
    data: MapData;
    error?: string;
}
/**
 * Request parameters for map data
 */
export interface MapDataRequest {
    worldId: string;
    /** Optional: viewport bounds for partial loading */
    bounds?: {
        minX: number;
        minY: number;
        maxX: number;
        maxY: number;
    };
    /** Optional: level of detail (0=low, 1=medium, 2=high) */
    lod?: 0 | 1 | 2;
}
//# sourceMappingURL=MapData.d.ts.map