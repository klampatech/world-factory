/**
 * World Factory - Society/Faction List Component
 * 
 * Displays societies and factions with territory, population, and relationship data.
 * 
 * @see WOR-21: R-15: Dashboard - society/faction list with relationships
 */

import { useEffect, useState, useCallback } from 'react';

// =============================================================================
// Types
// =============================================================================

export interface SettlementInfo {
  id: string;
  name: string;
  settlement_type: string | null;
  population: number | null;
  location: {
    latitude: number;
    longitude: number;
    elevation_m: number | null;
  };
  description: string | null;
  species_id: string | null;
}

export interface Society {
  species_id: string;
  species_name: string;
  settlements: SettlementInfo[];
  total_population: number;
  settlement_count: number;
  dominant_settlement_type: string | null;
}

export interface SocietiesResponse {
  world_id: string;
  societies: Society[];
  total_societies: number;
  total_settlements: number;
}

export interface RelationshipInfo {
  source_species: string;
  target_species: string;
  status: 'allied' | 'neutral' | 'hostile' | 'unknown';
  description?: string;
}

export interface SocietyListProps {
  /** World ID to fetch societies for */
  worldId: string;
  /** Initial societies data to skip fetch */
  initialSocieties?: Society[];
  /** Base URL for API (default: http://localhost:3000/api/v1) */
  apiBase?: string;
  /** Called when a settlement is clicked */
  onSettlementClick?: (settlement: SettlementInfo) => void;
  /** Called when a society is clicked */
  onSocietyClick?: (society: Society) => void;
  /** CSS class name */
  className?: string;
}

// =============================================================================
// API Client
// =============================================================================

async function fetchSocieties(
  worldId: string,
  apiBase: string = 'http://localhost:3000/api/v1'
): Promise<SocietiesResponse> {
  const response = await fetch(`${apiBase}/worlds/${worldId}/societies`);
  
  if (!response.ok) {
    throw new Error(`Failed to fetch societies: ${response.status}`);
  }
  
  const json = await response.json();
  // Handle wrapped response format
  return json.data || json;
}

// =============================================================================
// Mock Relationships (for demo - replace with real API when available)
// =============================================================================

function generateMockRelationships(societies: Society[]): RelationshipInfo[] {
  const relationships: RelationshipInfo[] = [];
  const statusOptions: Array<'allied' | 'neutral' | 'hostile'> = ['allied', 'neutral', 'hostile'];
  
  for (let i = 0; i < societies.length; i++) {
    for (let j = i + 1; j < societies.length; j++) {
      const status = statusOptions[Math.floor(Math.random() * statusOptions.length)];
      const desc = status === 'allied' 
        ? 'Trade agreements and mutual defense pact'
        : status === 'hostile'
        ? 'Border disputes and trade embargoes'
        : 'Limited diplomatic contact';
      
      relationships.push({
        source_species: societies[i].species_id,
        target_species: societies[j].species_id,
        status,
        description: desc,
      });
    }
  }
  
  return relationships;
}

// =============================================================================
// Component
// =============================================================================

/**
 * Society/Faction List Component
 * 
 * Displays a list of societies/factions with:
 * - Total population
 * - Settlement count
 * - Dominant settlement type
 * - Territory visualization (settlement dots)
 * - Relationship status between societies
 */
export function SocietyList({
  worldId,
  initialSocieties,
  apiBase = 'http://localhost:3000/api/v1',
  onSettlementClick,
  onSocietyClick,
  className,
}: SocietyListProps) {
  const [loading, setLoading] = useState(!initialSocieties);
  const [error, setError] = useState<Error | null>(null);
  const [societies, setSocieties] = useState<Society[]>(initialSocieties || []);
  const [relationships, setRelationships] = useState<RelationshipInfo[]>([]);
  const [activeTab, setActiveTab] = useState<'list' | 'relationships'>('list');
  const [selectedSociety, setSelectedSociety] = useState<string | null>(null);

  // Fetch societies
  useEffect(() => {
    if (initialSocieties) {
      setSocieties(initialSocieties);
      setRelationships(generateMockRelationships(initialSocieties));
      return;
    }

    async function load() {
      setLoading(true);
      setError(null);
      
      try {
        const data = await fetchSocieties(worldId, apiBase);
        setSocieties(data.societies);
        setRelationships(generateMockRelationships(data.societies));
      } catch (err) {
        setError(err instanceof Error ? err : new Error('Unknown error'));
      } finally {
        setLoading(false);
      }
    }

    load();
  }, [worldId, apiBase, initialSocieties]);

  // Handlers
  const handleSettlementClick = useCallback((settlement: SettlementInfo) => {
    onSettlementClick?.(settlement);
  }, [onSettlementClick]);

  const handleSocietyClick = useCallback((society: Society) => {
    setSelectedSociety(selectedSociety === society.species_id ? null : society.species_id);
    onSocietyClick?.(society);
  }, [selectedSociety, onSocietyClick]);

  const handleRetry = useCallback(() => {
    setLoading(true);
    setError(null);
    // Re-trigger useEffect
    setSocieties(prev => {
      setLoading(false);
      return prev;
    });
  }, []);

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'allied': return '#00ff88';
      case 'neutral': return '#fbbf24';
      case 'hostile': return '#ff6b6b';
      default: return '#888';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'allied': return '🤝';
      case 'neutral': return '⚖️';
      case 'hostile': return '⚔️';
      default: return '❓';
    }
  };

  const formatPopulation = (pop: number) => {
    if (pop >= 1000000) return `${(pop / 1000000).toFixed(1)}M`;
    if (pop >= 1000) return `${(pop / 1000).toFixed(1)}K`;
    return pop.toString();
  };

  const getSpeciesColor = (speciesId: string) => {
    const colors: Record<string, string> = {
      'human': '#4ade80',
      'elf': '#a78bfa',
      'dwarf': '#fb923c',
      'orc': '#ef4444',
      'halfling': '#38bdf8',
    };
    return colors[speciesId] || '#888';
  };

  const getSpeciesIcon = (speciesId: string) => {
    const icons: Record<string, string> = {
      'human': '👤',
      'elf': '🧝',
      'dwarf': '⛏️',
      'orc': '💀',
      'halfling': '🧒',
    };
    return icons[speciesId] || '🏛️';
  };

  // Loading state
  if (loading) {
    return (
      <div className={`society-list ${className || ''}`}>
        <div className="society-header">
          <h3>Societies & Factions</h3>
        </div>
        <div className="society-loading">
          {[1, 2, 3].map(i => (
            <div key={i} className="society-card skeleton">
              <div className="skeleton-icon" />
              <div className="skeleton-info">
                <div className="skeleton-title" />
                <div className="skeleton-text" />
              </div>
            </div>
          ))}
        </div>
        <style>{STYLES}</style>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className={`society-list ${className || ''}`}>
        <div className="society-header">
          <h3>Societies & Factions</h3>
        </div>
        <div className="society-error">
          <span className="error-icon">⚠️</span>
          <p>Failed to load societies</p>
          <button onClick={handleRetry} className="retry-btn">Retry</button>
        </div>
        <style>{STYLES}</style>
      </div>
    );
  }

  // Empty state
  if (societies.length === 0) {
    return (
      <div className={`society-list ${className || ''}`}>
        <div className="society-header">
          <h3>Societies & Factions</h3>
        </div>
        <div className="society-empty">
          <span className="empty-icon">🏛️</span>
          <p>No societies discovered yet</p>
        </div>
        <style>{STYLES}</style>
      </div>
    );
  }

  return (
    <div className={`society-list ${className || ''}`}>
      {/* Header with tabs */}
      <div className="society-header">
        <h3>Societies & Factions</h3>
        <div className="society-tabs">
          <button 
            className={`tab-btn ${activeTab === 'list' ? 'active' : ''}`}
            onClick={() => setActiveTab('list')}
          >
            List ({societies.length})
          </button>
          <button 
            className={`tab-btn ${activeTab === 'relationships' ? 'active' : ''}`}
            onClick={() => setActiveTab('relationships')}
          >
            Relationships ({relationships.length})
          </button>
        </div>
      </div>

      {/* Content */}
      {activeTab === 'list' ? (
        <div className="society-grid">
          {societies.map(society => (
            <div 
              key={society.species_id}
              className={`society-card ${selectedSociety === society.species_id ? 'selected' : ''}`}
              onClick={() => handleSocietyClick(society)}
            >
              {/* Society Header */}
              <div className="society-card-header">
                <div 
                  className="species-badge"
                  style={{ backgroundColor: getSpeciesColor(society.species_id) + '30' }}
                >
                  <span className="species-icon">{getSpeciesIcon(society.species_id)}</span>
                  <span className="species-name">{society.species_name}</span>
                </div>
                <span className="settlement-count">
                  {society.settlement_count} {society.settlement_count === 1 ? 'settlement' : 'settlements'}
                </span>
              </div>

              {/* Population */}
              <div className="society-stats">
                <div className="stat-item">
                  <span className="stat-label">Population</span>
                  <span className="stat-value">{formatPopulation(society.total_population)}</span>
                </div>
                <div className="stat-item">
                  <span className="stat-label">Type</span>
                  <span className="stat-value">
                    {society.dominant_settlement_type || 'Various'}
                  </span>
                </div>
              </div>

              {/* Territory visualization */}
              <div className="territory-viz">
                <span className="territory-label">Territory</span>
                <div className="settlement-dots">
                  {society.settlements.slice(0, 8).map((settlement, idx) => (
                    <div 
                      key={settlement.id}
                      className="settlement-dot"
                      style={{ 
                        backgroundColor: getSpeciesColor(society.species_id),
                        left: `${10 + (idx % 4) * 25}%`,
                        top: `${20 + Math.floor(idx / 4) * 40}%`,
                      }}
                      onClick={(e) => {
                        e.stopPropagation();
                        handleSettlementClick(settlement);
                      }}
                      title={`${settlement.name} (${settlement.population?.toLocaleString() || '?'})`}
                    />
                  ))}
                  {society.settlements.length > 8 && (
                    <span className="more-indicator">+{society.settlements.length - 8}</span>
                  )}
                </div>
              </div>

              {/* Expandable settlements list */}
              {selectedSociety === society.species_id && (
                <div className="settlements-list">
                  <h4>Settlements</h4>
                  {society.settlements.map(settlement => (
                    <div 
                      key={settlement.id}
                      className="settlement-row"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleSettlementClick(settlement);
                      }}
                    >
                      <span className="settlement-name">{settlement.name}</span>
                      <span className="settlement-type">{settlement.settlement_type}</span>
                      <span className="settlement-pop">
                        {settlement.population?.toLocaleString() || '-'}
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      ) : (
        /* Relationships view */
        <div className="relationships-list">
          {relationships.map((rel, idx) => (
            <div key={idx} className="relationship-card">
              <div className="relationship-parties">
                <span 
                  className="party-badge"
                  style={{ backgroundColor: getSpeciesColor(rel.source_species) + '30' }}
                >
                  {getSpeciesIcon(rel.source_species)}
                  <span>{rel.source_species}</span>
                </span>
                <span className="vs-label">vs</span>
                <span 
                  className="party-badge"
                  style={{ backgroundColor: getSpeciesColor(rel.target_species) + '30' }}
                >
                  {getSpeciesIcon(rel.target_species)}
                  <span>{rel.target_species}</span>
                </span>
              </div>
              <div 
                className="relationship-status"
                style={{ color: getStatusColor(rel.status) }}
              >
                <span className="status-icon">{getStatusIcon(rel.status)}</span>
                <span className="status-text">{rel.status.toUpperCase()}</span>
              </div>
              {rel.description && (
                <p className="relationship-desc">{rel.description}</p>
              )}
            </div>
          ))}
        </div>
      )}

      <style>{STYLES}</style>
    </div>
  );
}

export default SocietyList;

// =============================================================================
// Styles
// =============================================================================

const STYLES = `
  .society-list {
    padding: 20px;
    background: #0f0f1a;
    border-radius: 12px;
  }

  .society-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .society-header h3 {
    margin: 0;
    font-size: 20px;
    font-weight: 600;
    color: #fff;
  }

  .society-tabs {
    display: flex;
    gap: 8px;
  }

  .tab-btn {
    padding: 8px 16px;
    background: #1a1a2e;
    border: 1px solid #2a2a4e;
    border-radius: 8px;
    color: #888;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .tab-btn:hover {
    background: #2a2a4e;
    color: #fff;
  }

  .tab-btn.active {
    background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
    border-color: transparent;
    color: #fff;
  }

  /* Loading skeleton */
  .society-loading {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .society-card.skeleton {
    background: #1a1a2e;
    border-radius: 12px;
    padding: 20px;
    display: flex;
    align-items: center;
    gap: 16px;
    animation: pulse 1.5s ease-in-out infinite;
  }

  .skeleton-icon {
    width: 48px;
    height: 48px;
    border-radius: 8px;
    background: #2a2a4e;
  }

  .skeleton-info {
    flex: 1;
  }

  .skeleton-title {
    height: 20px;
    width: 60%;
    background: #2a2a4e;
    border-radius: 4px;
    margin-bottom: 8px;
  }

  .skeleton-text {
    height: 14px;
    width: 40%;
    background: #2a2a4e;
    border-radius: 4px;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  /* Error state */
  .society-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 40px;
    text-align: center;
  }

  .error-icon {
    font-size: 40px;
    margin-bottom: 12px;
  }

  .society-error p {
    color: #888;
    margin: 0 0 16px 0;
  }

  .retry-btn {
    padding: 10px 20px;
    background: #3a3a6e;
    border: none;
    border-radius: 8px;
    color: #fff;
    cursor: pointer;
    transition: background 0.2s;
  }

  .retry-btn:hover {
    background: #4a4a8e;
  }

  /* Empty state */
  .society-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 60px;
    text-align: center;
  }

  .empty-icon {
    font-size: 48px;
    margin-bottom: 12px;
    opacity: 0.5;
  }

  .society-empty p {
    color: #666;
    margin: 0;
  }

  /* Society grid */
  .society-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
  }

  .society-card {
    background: #1a1a2e;
    border-radius: 12px;
    padding: 20px;
    cursor: pointer;
    transition: all 0.2s;
    border: 1px solid transparent;
  }

  .society-card:hover {
    transform: translateY(-2px);
    border-color: #3a3a6e;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  }

  .society-card.selected {
    border-color: #6366f1;
  }

  .society-card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .species-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border-radius: 20px;
  }

  .species-icon {
    font-size: 16px;
  }

  .species-name {
    font-weight: 600;
    color: #fff;
  }

  .settlement-count {
    font-size: 12px;
    color: #888;
  }

  .society-stats {
    display: flex;
    gap: 24px;
    margin-bottom: 16px;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
  }

  .stat-label {
    font-size: 11px;
    color: #666;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .stat-value {
    font-size: 20px;
    font-weight: 700;
    color: #fff;
  }

  .territory-viz {
    position: relative;
    height: 80px;
    background: #0f0f1a;
    border-radius: 8px;
    margin-bottom: 12px;
  }

  .territory-label {
    position: absolute;
    top: 8px;
    left: 8px;
    font-size: 10px;
    color: #666;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .settlement-dots {
    position: relative;
    width: 100%;
    height: 100%;
    padding: 20px;
  }

  .settlement-dot {
    position: absolute;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    cursor: pointer;
    transition: transform 0.2s, box-shadow 0.2s;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .settlement-dot:hover {
    transform: scale(1.3);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
  }

  .more-indicator {
    position: absolute;
    bottom: 8px;
    right: 8px;
    font-size: 11px;
    color: #666;
    background: #1a1a2e;
    padding: 2px 8px;
    border-radius: 10px;
  }

  .settlements-list {
    border-top: 1px solid #2a2a4e;
    padding-top: 12px;
    margin-top: 12px;
  }

  .settlements-list h4 {
    margin: 0 0 8px 0;
    font-size: 12px;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .settlement-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid #1a1a2e;
    cursor: pointer;
    transition: background 0.2s;
    border-radius: 4px;
    padding: 8px;
    margin: 0 -8px;
  }

  .settlement-row:hover {
    background: #2a2a4e;
  }

  .settlement-row:last-child {
    border-bottom: none;
  }

  .settlement-name {
    font-weight: 500;
    color: #fff;
  }

  .settlement-type {
    font-size: 12px;
    color: #888;
    text-transform: capitalize;
  }

  .settlement-pop {
    font-size: 12px;
    color: #6366f1;
    font-variant-numeric: tabular-nums;
  }

  /* Relationships list */
  .relationships-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .relationship-card {
    background: #1a1a2e;
    border-radius: 12px;
    padding: 20px;
  }

  .relationship-parties {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
    margin-bottom: 12px;
  }

  .party-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    border-radius: 20px;
  }

  .vs-label {
    color: #666;
    font-size: 12px;
  }

  .relationship-status {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .status-icon {
    font-size: 18px;
  }

  .status-text {
    font-size: 14px;
    font-weight: 600;
    letter-spacing: 0.5px;
  }

  .relationship-desc {
    text-align: center;
    color: #888;
    font-size: 13px;
    margin: 0;
  }

  /* Responsive */
  @media (max-width: 768px) {
    .society-list {
      padding: 16px;
    }

    .society-header {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }

    .society-grid {
      grid-template-columns: 1fr;
    }

    .society-stats {
      flex-direction: column;
      gap: 12px;
    }
  }
`;