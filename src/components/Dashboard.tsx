/**
 * World Factory - Dashboard Component
 * 
 * Displays world state summary with status, progress, and quick actions
 * 
 * @see WOR-42: Dashboard - World State Summary
 */

import { useEffect, useState, useCallback } from 'react';

// =============================================================================
// Types
// =============================================================================

export interface WorldSummary {
  id: string;
  name: string;
  status: 'generating' | 'ready' | 'failed';
  progress: number;
  createdAt: string;
  parameters?: {
    seed?: number;
    size?: string;
  };
}

/**
 * World-level state summary metrics (per-world statistics)
 */
export interface WorldStateMetrics {
  /** Current in-world year being displayed */
  currentYear: number;
  /** Total population across all species */
  totalPopulation: number;
  /** Population breakdown by species type */
  populationBySpecies: {
    species: string;
    population: number;
    percentage: number;
  }[];
  /** Number of active societies/civilizations */
  activeSocieties: number;
  /** Resource summary statistics */
  resources: {
    type: string;
    total: number;
    scarcity: 'abundant' | 'common' | 'rare' | 'critical';
  }[];
}

export interface DashboardState {
  loading: boolean;
  error: Error | null;
  worlds: WorldSummary[];
  total: number;
  /** Current selected world's metrics (null if no world selected) */
  selectedWorldMetrics: WorldStateMetrics | null;
}

export interface DashboardProps {
  /** Initial worlds to display (skip fetch if provided) */
  initialWorlds?: WorldSummary[];
  /** Maximum worlds to display */
  limit?: number;
  /** CSS class name */
  className?: string;
  /** Called when a world is selected */
  onWorldSelect?: (worldId: string) => void;
  /** Called when create new world is clicked */
  onCreateWorld?: () => void;
  /** Initial world state metrics to display */
  initialMetrics?: WorldStateMetrics | null;
}

// =============================================================================
// Component
// =============================================================================

/**
 * Dashboard component displaying world state summary
 * 
 * Features:
 * - World list with status indicators
 * - World state summary (year, population, societies, resources)
 * - Progress bars for generating worlds
 * - Responsive grid layout
 * - Empty state with CTA
 * - Error state with retry
 */
export function Dashboard({
  initialWorlds,
  limit = 20,
  className,
  onWorldSelect,
  onCreateWorld,
  initialMetrics,
}: DashboardProps) {
  const [state, setState] = useState<DashboardState>({
    loading: !initialWorlds,
    error: null,
    worlds: initialWorlds || [],
    total: 0,
    selectedWorldMetrics: initialMetrics || null,
  });

  // Fetch worlds list (simulated for now - backend integration pending)
  useEffect(() => {
    if (initialWorlds) {
      setState(prev => ({ ...prev, worlds: initialWorlds }));
      return;
    }

    // Simulated data for development - replace with actual API call
    // When backend is ready: GET /api/worlds?limit={limit}
    const simulatedWorlds: WorldSummary[] = [
      {
        id: 'demo-world-1',
        name: 'Thornvald',
        status: 'ready',
        progress: 1,
        createdAt: '2026-04-30T10:00:00Z',
        parameters: { seed: 12345, size: 'large' },
      },
      {
        id: 'demo-world-2',
        name: 'Azure Shores',
        status: 'generating',
        progress: 0.65,
        createdAt: '2026-04-29T15:30:00Z',
        parameters: { seed: 67890, size: 'medium' },
      },
      {
        id: 'demo-world-3',
        name: 'Emerald Highlands',
        status: 'generating',
        progress: 0.25,
        createdAt: '2026-04-30T08:00:00Z',
        parameters: { seed: 11111, size: 'small' },
      },
    ];

    setState({
      loading: false,
      error: null,
      worlds: simulatedWorlds,
      total: simulatedWorlds.length,
      selectedWorldMetrics: initialMetrics || null,
    });
  }, [initialWorlds, initialMetrics]);

  const handleWorldClick = useCallback((worldId: string) => {
    onWorldSelect?.(worldId);
    // Simulated metrics for selected world
    setState(prev => ({
      ...prev,
      selectedWorldMetrics: {
        currentYear: 1247,
        totalPopulation: 2847293,
        populationBySpecies: [
          { species: 'Human', population: 1823456, percentage: 64 },
          { species: 'Elf', population: 423891, percentage: 15 },
          { species: 'Dwarf', population: 347821, percentage: 12 },
          { species: 'Orc', population: 152734, percentage: 5 },
          { species: 'Other', population: 100991, percentage: 4 },
        ],
        activeSocieties: 47,
        resources: [
          { type: 'Iron', total: 8934, scarcity: 'common' },
          { type: 'Gold', total: 1247, scarcity: 'rare' },
          { type: 'Gems', total: 456, scarcity: 'critical' },
          { type: 'Timber', total: 45230, scarcity: 'abundant' },
          { type: 'Stone', total: 28947, scarcity: 'abundant' },
        ],
      },
    }));
  }, [onWorldSelect]);

  const handleCreateClick = useCallback(() => {
    onCreateWorld?.();
  }, [onCreateWorld]);

  const handleRetry = useCallback(() => {
    setState(prev => ({ ...prev, loading: true, error: null }));
    // Re-trigger fetch logic
    setTimeout(() => {
      setState(prev => ({ ...prev, loading: false }));
    }, 500);
  }, []);

  // Loading state
  if (state.loading) {
    return (
      <div className={`dashboard ${className || ''}`}>
        <div className="dashboard-header">
          <h2>World Dashboard</h2>
        </div>
        <div className="dashboard-content">
          <div className="world-grid">
            {[1, 2, 3].map(i => (
              <div key={i} className="world-card skeleton">
                <div className="skeleton-header" />
                <div className="skeleton-body">
                  <div className="skeleton-line" />
                  <div className="skeleton-line short" />
                </div>
              </div>
            ))}
          </div>
        </div>
        <style>{`
          .dashboard {
            padding: 24px;
            background: #0f0f1a;
            min-height: 100vh;
          }
          .dashboard-header {
            margin-bottom: 24px;
          }
          .dashboard-header h2 {
            margin: 0;
            font-size: 24px;
            font-weight: 600;
            color: #fff;
          }
          .world-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
            gap: 16px;
          }
          .world-card.skeleton {
            background: #1a1a2e;
            border-radius: 12px;
            padding: 20px;
            animation: pulse 1.5s ease-in-out infinite;
          }
          .skeleton-header {
            height: 24px;
            background: #2a2a4e;
            border-radius: 4px;
            margin-bottom: 16px;
            width: 60%;
          }
          .skeleton-body {
            display: flex;
            flex-direction: column;
            gap: 8px;
          }
          .skeleton-line {
            height: 16px;
            background: #2a2a4e;
            border-radius: 4px;
          }
          .skeleton-line.short {
            width: 40%;
          }
          @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
          }
        `}</style>
      </div>
    );
  }

  // Error state
  if (state.error) {
    return (
      <div className={`dashboard ${className || ''}`}>
        <div className="dashboard-header">
          <h2>World Dashboard</h2>
        </div>
        <div className="dashboard-error">
          <div className="error-icon">⚠️</div>
          <h3>Failed to load worlds</h3>
          <p>{state.error.message}</p>
          <button onClick={handleRetry} className="retry-button">
            Retry
          </button>
        </div>
        <style>{`
          .dashboard {
            padding: 24px;
            background: #0f0f1a;
            min-height: 100vh;
          }
          .dashboard-header {
            margin-bottom: 24px;
          }
          .dashboard-header h2 {
            margin: 0;
            font-size: 24px;
            font-weight: 600;
            color: #fff;
          }
          .dashboard-error {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 60px 24px;
            text-align: center;
          }
          .error-icon {
            font-size: 48px;
            margin-bottom: 16px;
          }
          .dashboard-error h3 {
            margin: 0 0 8px 0;
            color: #ff6b6b;
            font-size: 18px;
          }
          .dashboard-error p {
            margin: 0 0 24px 0;
            color: #888;
          }
          .retry-button {
            padding: 12px 24px;
            background: #3a3a6e;
            color: #fff;
            border: none;
            border-radius: 8px;
            font-size: 14px;
            cursor: pointer;
            transition: background 0.2s;
          }
          .retry-button:hover {
            background: #4a4a8e;
          }
        `}</style>
      </div>
    );
  }

  // Empty state
  if (state.worlds.length === 0) {
    return (
      <div className={`dashboard ${className || ''}`}>
        <div className="dashboard-header">
          <h2>World Dashboard</h2>
        </div>
        <div className="dashboard-empty">
          <div className="empty-icon">🌍</div>
          <h3>No worlds yet</h3>
          <p>Create your first world to begin the journey.</p>
          <button onClick={handleCreateClick} className="create-button">
            Create World
          </button>
        </div>
        <style>{`
          .dashboard {
            padding: 24px;
            background: #0f0f1a;
            min-height: 100vh;
          }
          .dashboard-header {
            margin-bottom: 24px;
          }
          .dashboard-header h2 {
            margin: 0;
            font-size: 24px;
            font-weight: 600;
            color: #fff;
          }
          .dashboard-empty {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 60px 24px;
            text-align: center;
          }
          .empty-icon {
            font-size: 64px;
            margin-bottom: 16px;
          }
          .dashboard-empty h3 {
            margin: 0 0 8px 0;
            color: #fff;
            font-size: 20px;
          }
          .dashboard-empty p {
            margin: 0 0 24px 0;
            color: #888;
          }
          .create-button {
            padding: 14px 28px;
            background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
            color: #fff;
            border: none;
            border-radius: 8px;
            font-size: 16px;
            font-weight: 500;
            cursor: pointer;
            transition: transform 0.2s, box-shadow 0.2s;
          }
          .create-button:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 24px rgba(99, 102, 241, 0.4);
          }
        `}</style>
      </div>
    );
  }

  // Helper functions
  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const formatNumber = (num: number) => {
    return new Intl.NumberFormat('en-US').format(num);
  };

  const getStatusColor = (status: WorldSummary['status']) => {
    switch (status) {
      case 'ready': return '#00ff88';
      case 'generating': return '#ffd700';
      case 'failed': return '#ff6b6b';
    }
  };

  const getStatusLabel = (status: WorldSummary['status']) => {
    switch (status) {
      case 'ready': return 'Ready';
      case 'generating': return 'Generating';
      case 'failed': return 'Failed';
    }
  };

  const getScarcityColor = (scarcity: string) => {
    switch (scarcity) {
      case 'abundant': return '#00ff88';
      case 'common': return '#4ade80';
      case 'rare': return '#fbbf24';
      case 'critical': return '#ff6b6b';
      default: return '#888';
    }
  };

  return (
    <div className={`dashboard ${className || ''}`}>
      {/* World State Summary Section */}
      {state.selectedWorldMetrics && (
        <div className="world-state-summary">
          <h2 className="summary-title">World State</h2>
          <div className="summary-grid">
            {/* Current Year - Large Display */}
            <div className="metric-card year-card">
              <span className="metric-label">Current Year</span>
              <span className="metric-value year-value">
                {state.selectedWorldMetrics.currentYear}
              </span>
            </div>
            
            {/* Total Population */}
            <div className="metric-card">
              <span className="metric-label">Total Population</span>
              <span className="metric-value">
                {formatNumber(state.selectedWorldMetrics.totalPopulation)}
              </span>
              <div className="species-breakdown">
                {state.selectedWorldMetrics.populationBySpecies.slice(0, 4).map(sp => (
                  <div key={sp.species} className="species-row">
                    <span className="species-name">{sp.species}</span>
                    <span className="species-pop">{formatNumber(sp.population)}</span>
                    <span className="species-pct">({sp.percentage}%)</span>
                  </div>
                ))}
              </div>
            </div>
            
            {/* Active Societies */}
            <div className="metric-card">
              <span className="metric-label">Active Societies</span>
              <span className="metric-value">
                {state.selectedWorldMetrics.activeSocieties}
              </span>
              <span className="metric-sublabel">Civilizations</span>
            </div>
            
            {/* Resources Summary */}
            <div className="metric-card">
              <span className="metric-label">Resources</span>
              <div className="resources-grid">
                {state.selectedWorldMetrics.resources.map(res => (
                  <div key={res.type} className="resource-row">
                    <span className="resource-type">{res.type}</span>
                    <span 
                      className="resource-scarcity" 
                      style={{ color: getScarcityColor(res.scarcity) }}
                    >
                      {res.scarcity}
                    </span>
                    <span className="resource-total">{formatNumber(res.total)}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Header */}
      <div className="dashboard-header">
        <h2>World Dashboard</h2>
        <button onClick={handleCreateClick} className="create-button">
          + New World
        </button>
      </div>

      {/* Stats Cards */}
      <div className="dashboard-stats">
        <div className="stat-card">
          <span className="stat-value">{state.total}</span>
          <span className="stat-label">Total Worlds</span>
        </div>
        <div className="stat-card">
          <span className="stat-value">
            {state.worlds.filter(w => w.status === 'ready').length}
          </span>
          <span className="stat-label">Ready</span>
        </div>
        <div className="stat-card">
          <span className="stat-value">
            {state.worlds.filter(w => w.status === 'generating').length}
          </span>
          <span className="stat-label">Generating</span>
        </div>
      </div>

      {/* World List */}
      <div className="world-grid">
        {state.worlds.map(world => (
          <div
            key={world.id}
            className="world-card"
            onClick={() => handleWorldClick(world.id)}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => e.key === 'Enter' && handleWorldClick(world.id)}
          >
            <div className="world-card-header">
              <h3 className="world-name">{world.name}</h3>
              <span
                className="world-status"
                style={{ color: getStatusColor(world.status) }}
              >
                {getStatusLabel(world.status)}
              </span>
            </div>
            
            {world.status === 'generating' && (
              <div className="progress-container">
                <div className="progress-bar">
                  <div
                    className="progress-fill"
                    style={{ width: `${world.progress * 100}%` }}
                  />
                </div>
                <span className="progress-text">
                  {Math.round(world.progress * 100)}%
                </span>
              </div>
            )}
            
            <div className="world-card-meta">
              <span className="meta-item">
                📅 {formatDate(world.createdAt)}
              </span>
              {world.parameters?.seed && (
                <span className="meta-item">
                  🎲 Seed: {world.parameters.seed}
                </span>
              )}
            </div>
          </div>
        ))}
      </div>

      <style>{`
        .dashboard {
          padding: 24px;
          background: #0f0f1a;
          min-height: 100vh;
        }

        /* World State Summary */
        .world-state-summary {
          background: linear-gradient(135deg, #1a1a3e 0%, #1a2a4e 100%);
          border-radius: 16px;
          padding: 24px;
          margin-bottom: 32px;
          border: 1px solid #2a2a5e;
        }
        .summary-title {
          margin: 0 0 20px 0;
          font-size: 20px;
          font-weight: 600;
          color: #fff;
        }
        .summary-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
          gap: 16px;
        }
        .metric-card {
          background: rgba(0, 0, 0, 0.3);
          border-radius: 12px;
          padding: 20px;
          display: flex;
          flex-direction: column;
        }
        .metric-label {
          font-size: 12px;
          color: #888;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          margin-bottom: 8px;
        }
        .metric-value {
          font-size: 28px;
          font-weight: 700;
          color: #fff;
        }
        .metric-sublabel {
          font-size: 12px;
          color: #666;
          margin-top: 4px;
        }
        .year-value {
          font-size: 48px;
          font-weight: 800;
          background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
          -webkit-background-clip: text;
          -webkit-text-fill-color: transparent;
          background-clip: text;
        }
        .species-breakdown {
          margin-top: 12px;
          padding-top: 12px;
          border-top: 1px solid rgba(255, 255, 255, 0.1);
        }
        .species-row {
          display: flex;
          justify-content: space-between;
          font-size: 13px;
          margin: 4px 0;
        }
        .species-name {
          color: #ccc;
        }
        .species-pop {
          color: #888;
        }
        .species-pct {
          color: #666;
          font-size: 11px;
        }
        .resources-grid {
          margin-top: 8px;
          display: flex;
          flex-direction: column;
          gap: 6px;
        }
        .resource-row {
          display: flex;
          justify-content: space-between;
          align-items: center;
          font-size: 13px;
        }
        .resource-type {
          color: #ccc;
          flex: 1;
        }
        .resource-scarcity {
          font-size: 11px;
          text-transform: uppercase;
          font-weight: 500;
          padding: 2px 8px;
          background: rgba(0, 0, 0, 0.3);
          border-radius: 4px;
          margin: 0 12px;
        }
        .resource-total {
          color: #888;
          font-variant-numeric: tabular-nums;
        }

        /* Header */
        .dashboard-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 24px;
        }
        .dashboard-header h2 {
          margin: 0;
          font-size: 24px;
          font-weight: 600;
          color: #fff;
        }
        .create-button {
          padding: 10px 20px;
          background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
          color: #fff;
          border: none;
          border-radius: 8px;
          font-size: 14px;
          font-weight: 500;
          cursor: pointer;
          transition: transform 0.2s, box-shadow 0.2s;
        }
        .create-button:hover {
          transform: translateY(-2px);
          box-shadow: 0 8px 24px rgba(99, 102, 241, 0.4);
        }

        /* Stats */
        .dashboard-stats {
          display: flex;
          gap: 16px;
          margin-bottom: 24px;
          flex-wrap: wrap;
        }
        .stat-card {
          background: #1a1a2e;
          border-radius: 12px;
          padding: 16px 24px;
          display: flex;
          flex-direction: column;
          min-width: 120px;
        }
        .stat-value {
          font-size: 28px;
          font-weight: 700;
          color: #fff;
        }
        .stat-label {
          font-size: 12px;
          color: #888;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }

        /* World Grid */
        .world-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
          gap: 16px;
        }
        .world-card {
          background: #1a1a2e;
          border-radius: 12px;
          padding: 20px;
          cursor: pointer;
          transition: transform 0.2s, box-shadow 0.2s, border-color 0.2s;
          border: 1px solid transparent;
        }
        .world-card:hover {
          transform: translateY(-4px);
          box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
          border-color: #3a3a6e;
        }
        .world-card:focus {
          outline: none;
          border-color: #6366f1;
        }
        .world-card-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 12px;
        }
        .world-name {
          margin: 0;
          font-size: 18px;
          font-weight: 600;
          color: #fff;
        }
        .world-status {
          font-size: 12px;
          font-weight: 500;
          text-transform: uppercase;
        }
        .progress-container {
          display: flex;
          align-items: center;
          gap: 12px;
          margin-bottom: 12px;
        }
        .progress-bar {
          flex: 1;
          height: 6px;
          background: #2a2a4e;
          border-radius: 3px;
          overflow: hidden;
        }
        .progress-fill {
          height: 100%;
          background: linear-gradient(90deg, #6366f1 0%, #8b5cf6 100%);
          border-radius: 3px;
          transition: width 0.3s ease;
        }
        .progress-text {
          font-size: 12px;
          color: #888;
          min-width: 40px;
          text-align: right;
        }
        .world-card-meta {
          display: flex;
          flex-wrap: wrap;
          gap: 12px;
        }
        .meta-item {
          font-size: 12px;
          color: #888;
        }

        /* Mobile */
        @media (max-width: 768px) {
          .dashboard {
            padding: 16px;
          }
          .summary-grid {
            grid-template-columns: 1fr;
          }
          .year-value {
            font-size: 36px;
          }
          .dashboard-header {
            flex-direction: column;
            align-items: flex-start;
            gap: 12px;
          }
          .dashboard-stats {
            flex-direction: column;
          }
          .stat-card {
            width: 100%;
          }
          .world-grid {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
}

export default Dashboard;
