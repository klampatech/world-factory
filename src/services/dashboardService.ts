/**
 * Dashboard Stats Service
 * 
 * Fetches world statistics from the backend API including:
 * - Population data by species
 * - Societies/civilizations data
 * - Resource summary statistics
 * 
 * @see WOR-1324: Phase 4: Dashboard stats — population, societies, resources
 */

import type { WorldStateMetrics } from '../components/Dashboard';

// API base URL - uses env var if set, otherwise assumes Vite proxy or direct backend access
// For development with Vite: /api/v1 (proxied to localhost:3000)
// For production: set VITE_API_BASE_URL to full backend URL (e.g., http://localhost:3000/api/v1)
const API_BASE = import.meta.env.VITE_API_BASE_URL || '/api/v1';

export interface PopulationBySpecies {
  species: string;
  population: number;
  percentage: number;
}

export interface ResourceSummary {
  type: string;
  total: number;
  scarcity: 'abundant' | 'common' | 'rare' | 'critical';
}

export interface SocietySummary {
  id: string;
  name: string;
  species: string;
  settlements: number;
  population: number;
}

export interface WorldStatsResponse {
  currentYear: number;
  totalPopulation: number;
  populationBySpecies: PopulationBySpecies[];
  activeSocieties: number;
  societies: SocietySummary[];
  resources: ResourceSummary[];
}

/**
 * Calculate scarcity level based on resource total
 */
function calculateScarcity(total: number, worldMax: number): ResourceSummary['scarcity'] {
  const ratio = total / worldMax;
  if (ratio > 0.5) return 'abundant';
  if (ratio > 0.25) return 'common';
  if (ratio > 0.1) return 'rare';
  return 'critical';
}

/**
 * Fetch world statistics from the backend API
 */
export async function fetchWorldStats(worldId: string): Promise<WorldStatsResponse> {
  try {
    // Fetch societies data (contains population)
    const societiesResponse = await fetch(`${API_BASE}/worlds/${worldId}/societies`);
    
    // Fetch map data (contains resources)
    const mapResponse = await fetch(`${API_BASE}/worlds/${worldId}/map`);
    
    // Fetch planet data for geography
    const planetResponse = await fetch(`${API_BASE}/worlds/${worldId}/planet`);
    
    // Handle API responses
    let societies: SocietySummary[] = [];
    let resources: ResourceSummary[] = [];
    let totalPopulation = 0;
    
    if (societiesResponse.ok) {
      const societiesData = await societiesResponse.json();
      const societiesList = societiesData?.data?.societies || [];
      
      // Aggregate population and settlements by species
      societies = societiesList.map((s: any) => ({
        id: s.species_id || s.id,
        name: s.species_name || 'Unknown Society',
        species: s.species_id || 'unknown',
        settlements: s.settlement_count || 0,
        population: s.total_population || 0,
      }));
      
      totalPopulation = societies.reduce((sum: number, s: SocietySummary) => sum + s.population, 0);
    }
    
    if (mapResponse.ok) {
      const mapData = await mapResponse.json();
      const resourceList = mapData?.data?.resources || [];
      
      // Aggregate resources by type
      const resourceMap = new Map<string, number>();
      for (const r of resourceList) {
        const type = r.resource_type || 'unknown';
        resourceMap.set(type, (resourceMap.get(type) || 0) + (r.magnitude || 1));
      }
      
      const worldMax = Math.max(...resourceMap.values(), 1);
      resources = Array.from(resourceMap.entries()).map(([type, total]) => ({
        type,
        total,
        scarcity: calculateScarcity(total, worldMax),
      }));
    }
    
    // Calculate population by species
    const populationBySpecies: PopulationBySpecies[] = societies.map(s => ({
      species: s.name,
      population: s.population,
      percentage: totalPopulation > 0 ? Math.round((s.population / totalPopulation) * 100) : 0,
    })).sort((a, b) => b.population - a.population);
    
    // Get current year from planet data if available
    let currentYear = 1247; // Default historical year
    if (planetResponse.ok) {
      const planetData = await planetResponse.json();
      // Could derive year from planet metadata if available
    }
    
    return {
      currentYear,
      totalPopulation,
      populationBySpecies,
      activeSocieties: societies.length,
      societies,
      resources,
    };
  } catch (error) {
    console.error('Failed to fetch world stats:', error);
    // Return mock data on failure
    return generateMockStats();
  }
}

/**
 * Transform API response to WorldStateMetrics for Dashboard component
 */
export function transformToWorldStateMetrics(stats: WorldStatsResponse): WorldStateMetrics {
  return {
    currentYear: stats.currentYear,
    totalPopulation: stats.totalPopulation,
    populationBySpecies: stats.populationBySpecies,
    activeSocieties: stats.activeSocieties,
    resources: stats.resources,
  };
}

/**
 * Generate mock stats for development/demo
 */
export function generateMockStats(): WorldStatsResponse {
  return {
    currentYear: 1247,
    totalPopulation: 2847293,
    populationBySpecies: [
      { species: 'Human', population: 1823456, percentage: 64 },
      { species: 'Elf', population: 423891, percentage: 15 },
      { species: 'Dwarf', population: 347821, percentage: 12 },
      { species: 'Orc', population: 152734, percentage: 5 },
      { species: 'Halfling', population: 100991, percentage: 4 },
    ],
    activeSocieties: 47,
    societies: [
      { id: 'human-1', name: 'Kingdom of Aldoria', species: 'human', settlements: 12, population: 850000 },
      { id: 'human-2', name: 'Empire of Brenn', species: 'human', settlements: 8, population: 520000 },
      { id: 'elf-1', name: 'Forest Covenant', species: 'elf', settlements: 6, population: 423891 },
      { id: 'dwarf-1', name: 'Ironhold Clan', species: 'dwarf', settlements: 4, population: 347821 },
      { id: 'orc-1', name: 'Frostbane Horde', species: 'orc', settlements: 3, population: 152734 },
    ],
    resources: [
      { type: 'Iron', total: 8934, scarcity: 'common' },
      { type: 'Gold', total: 1247, scarcity: 'rare' },
      { type: 'Gems', total: 456, scarcity: 'critical' },
      { type: 'Timber', total: 45230, scarcity: 'abundant' },
      { type: 'Stone', total: 28947, scarcity: 'abundant' },
      { type: 'Copper', total: 5678, scarcity: 'common' },
    ],
  };
}

/**
 * Generate default/empty metrics for initial state
 */
export function getDefaultWorldMetrics(): WorldStateMetrics {
  return {
    currentYear: 0,
    totalPopulation: 0,
    populationBySpecies: [],
    activeSocieties: 0,
    resources: [],
  };
}
