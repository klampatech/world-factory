/**
 * World Factory - Map Component
 *
 * React component for rendering world maps with the MapViewer
 */
import type { MapData, MapDataRequest } from '../terrain/MapData';
export interface MapComponentProps {
    /** World ID to load */
    worldId: string;
    /** Initial viewport bounds */
    initialBounds?: MapDataRequest['bounds'];
    /** Level of detail */
    lod?: 0 | 1 | 2;
    /** CSS class name */
    className?: string;
    /** Called when map data loads successfully */
    onLoad?: (data: MapData) => void;
    /** Called on error */
    onError?: (error: Error) => void;
}
export interface MapComponentState {
    loading: boolean;
    error: Error | null;
    mapData: MapData | null;
}
export declare function MapComponent({ worldId, initialBounds, lod, className, onLoad, onError, }: MapComponentProps): any;
export default MapComponent;
//# sourceMappingURL=MapComponent.d.ts.map