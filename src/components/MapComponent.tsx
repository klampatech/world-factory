/**
 * World Factory - Map Component
 * 
 * React component for rendering world maps with the MapViewer
 */

import { useEffect, useRef, useState, useCallback } from 'react';
import { MapViewer } from '../terrain/MapViewer';
import type { MapData, MapDataRequest } from '../terrain/MapData';
import { worldApi } from '../api/WorldApiClient';

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

export function MapComponent({
  worldId,
  initialBounds,
  lod = 1,
  className,
  onLoad,
  onError,
}: MapComponentProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const viewerRef = useRef<MapViewer | null>(null);
  
  const [state, setState] = useState<MapComponentState>({
    loading: true,
    error: null,
    mapData: null,
  });

  // Load map data
  useEffect(() => {
    let cancelled = false;

    async function loadMapData() {
      setState(prev => ({ ...prev, loading: true, error: null }));

      try {
        const request: MapDataRequest = { worldId, bounds: initialBounds, lod };
        const data = await worldApi.getMapData(request);
        
        if (!cancelled) {
          setState({ loading: false, error: null, mapData: data });
          onLoad?.(data);
        }
      } catch (err) {
        if (!cancelled) {
          const error = err instanceof Error ? err : new Error('Failed to load map');
          setState({ loading: false, error, mapData: null });
          onError?.(error);
        }
      }
    }

    loadMapData();

    return () => {
      cancelled = true;
    };
  }, [worldId, initialBounds, lod, onLoad, onError]);

  // Initialize/update MapViewer when canvas or data changes
  useEffect(() => {
    if (!canvasRef.current) return;

    if (state.mapData && !viewerRef.current) {
      viewerRef.current = new MapViewer({
        canvas: canvasRef.current,
        mapData: state.mapData,
        onReady: () => console.log('MapViewer ready'),
        onError: (error) => {
          console.error('MapViewer error:', error);
          onError?.(error);
        },
      });
    } else if (viewerRef.current && state.mapData) {
      viewerRef.current.setMapData(state.mapData);
    }

    return () => {
      viewerRef.current?.destroy();
      viewerRef.current = null;
    };
  }, [state.mapData, onError]);

  // Handle resize
  useEffect(() => {
    function handleResize() {
      if (canvasRef.current) {
        canvasRef.current.width = canvasRef.current.offsetWidth;
        canvasRef.current.height = canvasRef.current.offsetHeight;
        viewerRef.current?.render();
      }
    }

    handleResize();
    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
    };
  }, []);

  // Loading state with skeleton UI
  if (state.loading) {
    return (
      <div className={`map-loading ${className || ''}`} aria-label="Loading map">
        <div className="map-skeleton">
          <div className="skeleton-layer biome-layer" />
          <div className="skeleton-layer territories-layer" />
          <div className="skeleton-layer resources-layer" />
        </div>
        <style>{`
          .map-loading {
            position: relative;
            width: 100%;
            height: 100%;
            min-height: 400px;
            background: #1a1a2e;
            border-radius: 8px;
            overflow: hidden;
          }
          .map-skeleton {
            position: absolute;
            inset: 0;
          }
          .skeleton-layer {
            position: absolute;
            inset: 0;
            opacity: 0.3;
          }
          .biome-layer {
            background: linear-gradient(45deg, #2a2a4e 25%, #3a3a6e 50%, #2a2a4e 75%);
            background-size: 200% 200%;
            animation: shimmer 1.5s infinite;
          }
          .territories-layer {
            background: repeating-linear-gradient(
              0deg,
              transparent,
              transparent 20px,
              rgba(255, 215, 0, 0.1) 20px,
              rgba(255, 215, 0, 0.1) 21px
            ),
            repeating-linear-gradient(
              90deg,
              transparent,
              transparent 20px,
              rgba(255, 215, 0, 0.1) 20px,
              rgba(255, 215, 0, 0.1) 21px
            );
          }
          .resources-layer {
            background: radial-gradient(circle at 30% 40%, rgba(255, 215, 0, 0.2) 0%, transparent 10%),
                       radial-gradient(circle at 70% 30%, rgba(139, 69, 19, 0.2) 0%, transparent 8%),
                       radial-gradient(circle at 50% 70%, rgba(65, 105, 225, 0.2) 0%, transparent 12%);
          }
          @keyframes shimmer {
            0% { background-position: 200% 0; }
            100% { background-position: -200% 0; }
          }
        `}</style>
      </div>
    );
  }

  // Error state
  if (state.error) {
    return (
      <div className={`map-error ${className || ''}`} role="alert">
        <div className="error-content">
          <h3>Failed to load map</h3>
          <p>{state.error.message}</p>
          <button onClick={() => window.location.reload()}>
            Retry
          </button>
        </div>
        <style>{`
          .map-error {
            display: flex;
            align-items: center;
            justify-content: center;
            width: 100%;
            height: 100%;
            min-height: 400px;
            background: #1a1a2e;
            border-radius: 8px;
            color: #ff6b6b;
          }
          .error-content {
            text-align: center;
          }
          .error-content h3 {
            margin: 0 0 8px 0;
            color: #ff6b6b;
          }
          .error-content p {
            margin: 0 0 16px 0;
            color: #888;
          }
          .error-content button {
            padding: 8px 16px;
            background: #3a3a6e;
            color: #fff;
            border: none;
            border-radius: 4px;
            cursor: pointer;
          }
          .error-content button:hover {
            background: #4a4a8e;
          }
        `}</style>
      </div>
    );
  }

  // Map canvas
  return (
    <div className={`map-container ${className || ''}`}>
      <canvas
        ref={canvasRef}
        className="map-canvas"
        aria-label={`World map for ${worldId}`}
        role="img"
      />
      <div className="map-legend">
        <h4>Legend</h4>
        <ul>
          <li><span className="legend-icon territory" /> Territory</li>
          <li><span className="legend-icon biome" /> Biome</li>
          <li><span className="legend-icon resource" /> Resource</li>
          <li><span className="legend-icon entity" /> City/Settlement</li>
        </ul>
      </div>
      <div className="map-controls">
        <span>Scroll to zoom • Drag to pan</span>
      </div>
      <style>{`
        .map-container {
          position: relative;
          width: 100%;
          height: 100%;
          min-height: 400px;
          background: #1a1a2e;
          border-radius: 8px;
          overflow: hidden;
        }
        .map-canvas {
          width: 100%;
          height: 100%;
          cursor: grab;
        }
        .map-canvas:active {
          cursor: grabbing;
        }
        .map-legend {
          position: absolute;
          bottom: 16px;
          left: 16px;
          padding: 12px;
          background: rgba(0, 0, 0, 0.7);
          border-radius: 6px;
          font-size: 12px;
          color: #ccc;
        }
        .map-legend h4 {
          margin: 0 0 8px 0;
          font-size: 14px;
          color: #fff;
        }
        .map-legend ul {
          list-style: none;
          margin: 0;
          padding: 0;
        }
        .map-legend li {
          display: flex;
          align-items: center;
          gap: 8px;
          margin: 4px 0;
        }
        .legend-icon {
          display: inline-block;
          width: 12px;
          height: 12px;
          border-radius: 2px;
        }
        .legend-icon.territory { background: #ffd700; }
        .legend-icon.biome { background: #00ff88; }
        .legend-icon.resource { background: #ff6b6b; }
        .legend-icon.entity { background: #ff4500; }
        .map-controls {
          position: absolute;
          top: 16px;
          right: 16px;
          padding: 8px 12px;
          background: rgba(0, 0, 0, 0.7);
          border-radius: 6px;
          font-size: 12px;
          color: #888;
        }
      `}</style>
    </div>
  );
}

export default MapComponent;
