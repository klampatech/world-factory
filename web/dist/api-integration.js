/**
 * World Factory — Real API Integration Module
 * 
 * Replaces mock data generation with actual API calls.
 * Falls back to mock data if API is unavailable.
 * 
 * Usage:
 *   - Set API_BASE to your backend URL (default: /api/v1 via Vite proxy to localhost:3000)
 *   - Create a world first, then use the returned world ID
 * 
 * API Endpoints:
 *   POST /worlds          → Create world, returns { id, name, ... }
 *   POST /worlds/:id/generate → Generate world content
 *   GET  /worlds/:id/planet  → Get planet/world data
 *   GET  /worlds/:id/map     → Get map polygon data
 *   GET  /worlds/:id/timeline → Get timeline/events
 *   GET  /worlds/:id/wonders → Get natural wonders
 *   GET  /worlds/:id/events   → Get events
 */

// API base URL configuration
// Default to correct backend port (3000) for direct browser access
// Set window.API_BASE to override
const API_BASE = (typeof window !== 'undefined' && window.API_BASE) || 
                  'http://localhost:3000/api/v1';

// Current world state
let currentWorldId = null;
let currentSeed = 42;

// =============================================================================
// API Client
// =============================================================================

class WorldFactoryAPI {
  constructor(baseUrl = API_BASE) {
    this.baseUrl = baseUrl;
    this.useMockFallback = false; // Set to true for development/demo
  }

  async request(endpoint, options = {}) {
    const url = `${this.baseUrl}${endpoint}`;
    const defaultOptions = {
      headers: {
        'Content-Type': 'application/json',
      },
    };

    try {
      const response = await fetch(url, { ...defaultOptions, ...options });
      const data = await response.json();

      return {
        ok: response.ok,
        status: response.status,
        data: data,
      };
    } catch (error) {
      console.warn(`API request failed: ${endpoint}`, error.message);
      return { ok: false, status: 0, error: error.message, fallback: true };
    }
  }

  async createWorld(name, seed = Date.now()) {
    const result = await this.request('/worlds', {
      method: 'POST',
      body: JSON.stringify({
        name: name || `World ${seed}`,
        parameters: { seed, size: 'medium' }
      }),
    });

    if (result.ok && result.data) {
      // Handle ApiResponse wrapper: { success: true, data: { id, ... } }
      const worldData = result.data.data || result.data;
      return { ok: true, id: worldData.id, data: worldData };
    }

    if (this.useMockFallback) {
      return this.mockCreateWorld(name, seed);
    }

    return result;
  }

  async generateWorld(worldId, seed = null) {
    const result = await this.request(`/worlds/${worldId}/generate`, {
      method: 'POST',
      body: JSON.stringify(seed ? { seed } : {}),
    });

    if (result.ok) {
      return { ok: true, status: result.status };
    }

    if (this.useMockFallback) {
      return { ok: true, status: 202, mock: true };
    }

    return result;
  }

  async getWorld(worldId) {
    const result = await this.request(`/worlds/${worldId}`);

    if (result.ok && result.data) {
      const worldData = result.data.data || result.data;
      return { ok: true, data: worldData };
    }

    if (this.useMockFallback) {
      return this.mockGetWorld(worldId);
    }

    return result;
  }

  async getPlanetData(worldId) {
    const result = await this.request(`/worlds/${worldId}/planet`);

    if (result.ok && result.data) {
      const planetData = result.data.data || result.data;
      return { ok: true, data: planetData };
    }

    if (this.useMockFallback) {
      return { ok: true, data: this.mockGenerateWorld(), mock: true };
    }

    return result;
  }

  async getMapData(worldId) {
    const result = await this.request(`/worlds/${worldId}/map`);

    if (result.ok && result.data) {
      const mapData = result.data.data || result.data;
      return { ok: true, data: mapData };
    }

    if (this.useMockFallback) {
      // Mock map data
      const mockWorld = this.mockGenerateWorld();
      return {
        ok: true,
        data: {
          worldId: worldId,
          dimensions: { width: mockWorld.metadata.width, height: mockWorld.metadata.height },
          polygons: mockWorld.regions.map(r => ({
            id: r.id,
            vertices: r.polygon,
            biome: r.biome,
            center: r.center,
            regionInfo: r.regionInfo,
          })),
        },
        mock: true
      };
    }

    return result;
  }

  async getTimelineData(worldId) {
    const result = await this.request(`/worlds/${worldId}/timeline`);

    if (result.ok && result.data) {
      const timelineData = result.data.data || result.data;
      return { ok: true, data: timelineData };
    }

    if (this.useMockFallback) {
      return { ok: true, data: this.mockGenerateTimeline(), mock: true };
    }

    return result;
  }

  /**
   * Get history events from the API.
   * Maps to GET /api/v1/worlds/:id/history
   * Returns events with full details: id, event_type, year, title, description, etc.
   */
  async getHistoryData(worldId, options = {}) {
    const params = new URLSearchParams();
    if (options.limit) params.set('limit', options.limit);
    if (options.offset) params.set('offset', options.offset);
    if (options.eventTypes) params.set('eventTypes', options.eventTypes.join(','));
    if (options.startYear) params.set('startYear', options.startYear);
    if (options.endYear) params.set('endYear', options.endYear);
    
    const queryString = params.toString() ? '?' + params.toString() : '';
    const result = await this.request(`/worlds/${worldId}/history${queryString}`);

    if (result.ok && result.data) {
      return { ok: true, data: result.data };
    }

    if (this.useMockFallback) {
      // Fall back to mock timeline, formatted as history
      const mockTimeline = this.mockGenerateTimeline();
      return { 
        ok: true, 
        data: {
          events: mockTimeline.map(e => ({
            id: e.id,
            event_type: e.type,
            year: e.year,
            title: e.title,
            description: e.description,
            region: e.region,
            societies: e.societies,
            significance: e.significance
          })),
          total: mockTimeline.length,
          mock: true
        }, 
        mock: true 
      };
    }

    return result;
  }

  async getWondersData(worldId) {
    const result = await this.request(`/worlds/${worldId}/wonders`);

    if (result.ok && result.data) {
      const wondersData = result.data.data || result.data;
      return { ok: true, data: wondersData };
    }

    if (this.useMockFallback) {
      return { ok: true, data: this.mockGenerateWonders(), mock: true };
    }

    return result;
  }

  async getDisastersData(worldId) {
    const result = await this.request(`/worlds/${worldId}/disasters?limit=50`);

    if (result.ok && result.data) {
      const disastersData = result.data.data || result.data;
      return { ok: true, data: disastersData };
    }

    if (this.useMockFallback) {
      return { ok: true, data: this.mockGenerateDisasters(), mock: true };
    }

    return result;
  }

  async getResourcesData(worldId) {
    const result = await this.request(`/worlds/${worldId}/resources`);
    if (result.ok && result.data) {
      const resourcesData = result.data.data || result.data;
      return { ok: true, data: resourcesData };
    }
    if (this.useMockFallback) {
      return { ok: true, data: this.mockGenerateResources(), mock: true };
    }
    return result;
  }
  async getEventsData(worldId, params = {}) {
    const query = new URLSearchParams(params).toString();
    const endpoint = `/worlds/${worldId}/events${query ? '?' + query : ''}`;
    const result = await this.request(endpoint);

    if (result.ok && result.data) {
      const eventsData = result.data.data || result.data;
      return { ok: true, data: eventsData };
    }

    if (this.useMockFallback) {
      const timeline = this.mockGenerateTimeline();
      return { ok: true, data: timeline, mock: true };
    }

    return result;
  }

  async pollForGeneration(worldId, maxAttempts = 60, interval = 2000) {
    for (let i = 0; i < maxAttempts; i++) {
      const mapResult = await this.getMapData(worldId);

      if (mapResult.ok && mapResult.data?.polygons?.length > 0) {
        return { ok: true, ready: true, data: mapResult.data };
      }

      await new Promise(resolve => setTimeout(resolve, interval));
    }

    return { ok: false, ready: false, error: 'Generation timeout' };
  }

  // =============================================================================
  // Mock Data Fallbacks
  // =============================================================================

  mockCreateWorld(name, seed) {
    return {
      ok: true,
      id: `mock-${Date.now()}-${seed}`,
      data: {
        id: `mock-${Date.now()}-${seed}`,
        name: name || `World ${seed}`,
        seed: seed,
        createdAt: new Date().toISOString(),
      },
      mock: true,
    };
  }

  mockGetWorld(worldId) {
    return {
      ok: true,
      data: {
        id: worldId,
        name: `World ${worldId.slice(-8)}`,
        seed: currentSeed,
        status: 'generated',
      },
      mock: true,
    };
  }

  mockGenerateWorld() {
    const world = {
      name: 'Aethon Prime',
      seed: currentSeed,
      regions: [],
      metadata: {
        width: 2000,
        height: 1500,
      }
    };

    const centerX = world.metadata.width / 2;
    const centerY = world.metadata.height / 2;
    const gridSize = 80;

    const BIOME_COLORS = {
      ocean: '#1a3a5c',
      shallow_ocean: '#2d5a7b',
      desert: '#c4a35a',
      scrubland: '#8b9a6b',
      grassland: '#5c8a4d',
      forest: '#2d5a3d',
      rainforest: '#1a4a2d',
      tundra: '#a8b4c4',
      ice: '#e8f0f4',
      mountain: '#6b6b78',
      highland: '#4a4a55',
      swamp: '#3a5a3a',
      beach: '#d4c090',
    };

    const RESOURCE_COLORS = {
      iron: '#8b4513',
      gold: '#ffd700',
      gems: '#9370db',
      timber: '#228b22',
      spices: '#ff8c00',
      fish: '#4682b4',
      stone: '#808080',
      magic: '#9932cc',
    };

    const FACTION_NAMES = [
      'Ironhold Clan', 'Meridian Empire', 'Mountain Kings', 'Forest Covenant',
      'Desert Nomads', 'Coastal Alliance', 'Frozen Dominion', 'Sky Realms'
    ];

    // Regular flat-top hex: R = gridSize = 80, hexWidth = hexHeight = 2*R = 160 (uniform tiling)
    const R = gridSize;                   // ~80
    const hexWidth  = R * 2;               // ~160
    const hexHeight = R * 2;               // ~160
    
    for (let row = 0; row * hexHeight < world.metadata.height; row++) {
      for (let col = 0; col * hexWidth < world.metadata.width; col++) {
        const x = col * hexWidth + (row % 2 === 1 ? hexWidth / 2 : 0);
        const y = row * hexHeight;
        const hexCenterX = x + R;
        const hexCenterY = y + R;
        const dx = (hexCenterX - centerX) / (world.metadata.width / 2);
        const dy = (hexCenterY - centerY) / (world.metadata.height / 2);
        const dist = Math.sqrt(dx * dx + dy * dy);
        const noise = Math.sin(hexCenterX * 0.01) * Math.cos(hexCenterY * 0.01) * 0.3;
        const adjustedDist = dist + noise;

        let biome;
        if (adjustedDist > 0.8) biome = 'ocean';
        else if (adjustedDist > 0.7) biome = Math.random() > 0.7 ? 'shallow_ocean' : 'ocean';
        else if (adjustedDist > 0.65) biome = Math.random() > 0.6 ? 'beach' : 'shallow_ocean';
        else if (adjustedDist > 0.5) {
          const r = Math.random();
          if (r > 0.85) biome = 'mountain';
          else if (r > 0.7) biome = 'highland';
          else if (r > 0.5) biome = 'forest';
          else biome = 'grassland';
        } else if (adjustedDist > 0.3) {
          const r = Math.random();
          if (r > 0.9) biome = 'swamp';
          else if (r > 0.7) biome = 'rainforest';
          else if (r > 0.5) biome = 'forest';
          else biome = 'grassland';
        } else {
          const r = Math.random();
          if (r > 0.8) biome = 'desert';
          else if (r > 0.5) biome = 'scrubland';
          else biome = 'grassland';
        }

        const points = createHexPolygon(hexCenterX, hexCenterY, R);

        world.regions.push({
          id: `region-${col}-${row}`,
          name: generateRegionName(col, row),
          biome: biome,
          polygon: points,
          center: { x: hexCenterX, y: hexCenterY },
          regionInfo: {
            population: Math.floor(Math.random() * 100000) + 10000,
            resource: Object.keys(RESOURCE_COLORS)[Math.floor(Math.random() * Object.keys(RESOURCE_COLORS).length)],
            elevation: Math.floor(adjustedDist * 5000),
            faction: Math.floor(Math.random() * FACTION_NAMES.length),
          },
        });
      }
    }

    return world;
  }

  mockGenerateTimeline() {
    const events = [];
    const societies = ['Kingdom of Aldoria', 'Empire of Brenn', 'Confederation of Caldara', 'Realm of Drevon'];
    const regions = ['Northern Plains', 'Eastern Highlands', 'Western Forests', 'Southern Shores'];

    const eventTemplates = {
      war: ['Battle of {region}', '{society} invades {region}', 'Great War of {region}'],
      discovery: ['New land discovered in {region}', 'Expedition reaches {region}'],
      settlement: ['{society} founds new settlement', 'Colony established in {region}'],
      plague: ['Plague sweeps {region}', 'Disease spreads through {region}'],
      treaty: ['Peace treaty signed', 'Alliance formed at {region}'],
      innovation: ['{society} develops new technology', 'Breakthrough in {region}'],
    };

    const descriptions = {
      war: 'Armies clash in a decisive battle that will shape the future.',
      discovery: 'Bold explorers venture into uncharted territory.',
      settlement: 'Colonists establish a new foothold.',
      plague: 'A devastating illness spreads across the land.',
      treaty: 'Diplomats negotiate a landmark agreement.',
      innovation: 'Brilliant minds create new technologies.',
    };

    let eventId = 0;
    for (let year = 100; year <= 2200; year += Math.random() * 80 + 20) {
      const types = ['war', 'discovery', 'settlement', 'plague', 'treaty', 'innovation'];
      const type = types[Math.floor(Math.random() * types.length)];
      const template = eventTemplates[type][Math.floor(Math.random() * eventTemplates[type].length)];
      const society = societies[Math.floor(Math.random() * societies.length)];
      const region = regions[Math.floor(Math.random() * regions.length)];

      events.push({
        id: `event-${eventId++}`,
        title: template.replace('{society}', society).replace('{region}', region),
        type: type,
        year: Math.round(year),
        region: region,
        societies: [society],
        description: descriptions[type],
        significance: Math.random() * 0.5 + 0.5,
      });
    }

    return events.sort((a, b) => a.year - b.year);
  }

  mockGenerateWonders() {
    const wonders = [];
    const WONDER_TYPES = {
      SacredMountain: { name: 'Sacred Mountain', category: 'geological', icon: '⛰️' },
      GrandCanyon: { name: 'Grand Canyon', category: 'geological', icon: '🏜️' },
      AncientTree: { name: 'Ancient Tree', category: 'biological', icon: '🌳' },
      CrystalCavern: { name: 'Crystal Cavern', category: 'geological', icon: '💎' },
      ActiveVolcano: { name: 'Active Volcano', category: 'geological', icon: '🌋' },
      GreatLake: { name: 'Great Lake', category: 'hydrological', icon: '🌊' },
      LeyLineNexus: { name: 'Ley Line Nexus', category: 'magical', icon: '🔮' },
      ManaSpring: { name: 'Mana Spring', category: 'magical', icon: '💫' },
    };

    const WONDER_CATEGORIES = {
      geological: { label: 'Geological', color: '#8b6b4a' },
      hydrological: { label: 'Hydrological', color: '#4a8bc4' },
      biological: { label: 'Biological', color: '#4a8b4a' },
      magical: { label: 'Magical', color: '#c44ac4' },
    };

    const wonderCount = 5;
    const worldWidth = 2000;
    const worldHeight = 1500;

    for (let i = 0; i < wonderCount; i++) {
      const typeKey = Object.keys(WONDER_TYPES)[i % Object.keys(WONDER_TYPES).length];
      const typeDef = WONDER_TYPES[typeKey];
      const catDef = WONDER_CATEGORIES[typeDef.category];

      wonders.push({
        id: `wonder-${i}`,
        type: typeKey,
        name: typeDef.name,
        x: Math.random() * worldWidth,
        y: Math.random() * worldHeight,
        elevation: Math.floor(Math.random() * 3000),
        influenceRadius: 50 + Math.floor(Math.random() * 100),
        category: typeDef.category,
        icon: typeDef.icon,
        bonuses: [
          { type: 'Culture', magnitude: 1.3, radius: 2 },
          { type: 'Gold', magnitude: 1.2, radius: 1 },
        ],
        description: `A remarkable natural wonder of ${typeDef.category} significance.`,
      });
    }

    return { wonders, stats: { total: wonders.length } };
  }

  mockGenerateDisasters() {
    return {
      disasters: [
        {
          id: 'mock-disaster-1',
          disaster_type: 'famine',
          name: 'The Great Famine',
          description: 'A devastating famine has struck the northern territories.',
          severity: 0.85,
          start_year: 1340,
          end_year: 1350,
          is_resolved: false,
          affected_regions: ['Northern Plains', 'Eastern Highlands'],
          population_affected: 50000,
          recovery_estimate_years: 5,
        },
        {
          id: 'mock-disaster-2',
          disaster_type: 'plague',
          name: 'The Crimson Death',
          description: 'A deadly plague spreads through the coastal cities.',
          severity: 0.92,
          start_year: 1347,
          end_year: null,
          is_resolved: false,
          affected_regions: ['Southern Shores', 'Western Forests'],
          population_affected: 150000,
          recovery_estimate_years: 10,
        },
        {
          id: 'mock-disaster-3',
          disaster_type: 'drought',
          name: 'The Burning Years',
          description: 'A prolonged drought has devastated agricultural regions.',
          severity: 0.72,
          start_year: 1280,
          end_year: 1295,
          is_resolved: true,
          affected_regions: ['Eastern Highlands'],
          population_affected: 25000,
          recovery_estimate_years: null,
        },
        {
          id: 'mock-disaster-4',
          disaster_type: 'earthquake',
          name: 'The Shattering',
          description: 'A massive earthquake split the western mountain range.',
          severity: 0.78,
          start_year: 890,
          end_year: 892,
          is_resolved: true,
          affected_regions: ['Western Forests', 'Northern Plains'],
          population_affected: 30000,
          recovery_estimate_years: null,
        },
      ],
      total_disasters: 4,
      ongoing_count: 2,
      resolved_count: 2,
      total_population_affected: 255000,
    };
  }

  mockGenerateResources() {
    // Mock resource summary data matching the backend API response shape
    const resources = [
      { resourceType: 'Iron', depositCount: 24, totalUnits: 8934, avgQuality: 0.78, scarcity: 'common' },
      { resourceType: 'Gold', depositCount: 8, totalUnits: 1247, avgQuality: 0.85, scarcity: 'rare' },
      { resourceType: 'Gems', depositCount: 3, totalUnits: 456, avgQuality: 0.92, scarcity: 'critical' },
      { resourceType: 'Copper', depositCount: 18, totalUnits: 5621, avgQuality: 0.72, scarcity: 'common' },
      { resourceType: 'Stone', depositCount: 45, totalUnits: 28947, avgQuality: 0.65, scarcity: 'abundant' },
      { resourceType: 'Timber', depositCount: 52, totalUnits: 45230, avgQuality: 0.70, scarcity: 'abundant' },
      { resourceType: 'Coal', depositCount: 15, totalUnits: 7823, avgQuality: 0.68, scarcity: 'common' },
      { resourceType: 'Silver', depositCount: 6, totalUnits: 892, avgQuality: 0.81, scarcity: 'rare' },
    ];
    const byCategory = [
      { category: 'Metals', depositCount: 38, totalUnits: 15694 },
      { category: 'Minerals', depositCount: 48, totalUnits: 29403 },
      { category: 'Organic', depositCount: 52, totalUnits: 45230 },
    ];
    return {
      worldId: currentWorldId || 'mock-world',
      resources,
      totalDeposits: resources.reduce((sum, r) => sum + r.depositCount, 0),
      byCategory,
    };
  }
}

function createHexPolygon(cx, cy, r) {
  const rVert = r;  // regular hex: R_horiz = R_vert = r (= 80)
  const points = [];
  for (let i = 0; i < 6; i++) {
    const angle = (Math.PI / 6) + (Math.PI / 3) * i;
    points.push({ x: cx + r * Math.cos(angle), y: cy + rVert * Math.sin(angle) });
  }
  return points;
}

function seededRandom(seed) {
  let s = seed;
  return function() {
    s = Math.sin(s * 9999) * 10000;
    return s - Math.floor(s);
  };
}

function generateRegionName(x, y) {
  const prefixes = ['North', 'South', 'East', 'West', 'Upper', 'Lower', 'Greater', 'Lesser'];
  const roots = ['Aldoria', 'Brenn', 'Caldara', 'Drevon', 'Elmoor', 'Farath', 'Grenholde', 'Haver'];
  const suffixes = ['land', 'mark', 'vale', 'fell', 'reach', 'mere', 'wood', 'crest'];

  const seed = x * 1000 + y;
  const rng = seededRandom(seed);

  if (rng() > 0.7) {
    return prefixes[Math.floor(rng() * prefixes.length)] + ' ' + roots[Math.floor(rng() * roots.length)];
  }
  return roots[Math.floor(rng() * roots.length)] + ' ' + suffixes[Math.floor(rng() * suffixes.length)];
}

// =============================================================================
// Integration with Frontend
// =============================================================================

// Create API client instance
const worldAPI = new WorldFactoryAPI();

// Update fetch functions in the frontend to use real API
async function fetchPlanetDataAPI(worldId) {
  if (worldId) {
    const result = await worldAPI.getPlanetData(worldId);
    if (result.ok) {
      return result.data;
    }
  }
  // Fallback to mock if no worldId or API failed
  return worldAPI.mockGenerateWorld();
}

async function fetchTimelineDataAPI(worldId) {
  if (worldId) {
    const result = await worldAPI.getTimelineData(worldId);
    if (result.ok) {
      return result.data;
    }
  }
  return worldAPI.mockGenerateTimeline();
}

async function fetchWondersDataAPI(worldId) {
  if (worldId) {
    const result = await worldAPI.getWondersData(worldId);
    if (result.ok) {
      return result.data;
    }
  }
  return worldAPI.mockGenerateWonders();
}

async function fetchResourcesDataAPI(worldId) {
  if (worldId) {
    const result = await worldAPI.getResourcesData(worldId);
    if (result.ok) {
      return result.data;
    }
  }
  return worldAPI.mockGenerateResources();
}

async function generateWorldAPI(seed = null) {
  if (!currentWorldId) {
    // Create new world first
    const createResult = await worldAPI.createWorld(`World ${Date.now()}`, seed || currentSeed);
    if (createResult.ok) {
      currentWorldId = createResult.id;
      currentSeed = seed || currentSeed;
    } else {
      return { ok: false, error: 'Failed to create world' };
    }
  }

  // Trigger generation
  const genResult = await worldAPI.generateWorld(currentWorldId, seed);
  if (genResult.ok) {
    // Poll for completion
    const pollResult = await worldAPI.pollForGeneration(currentWorldId);
    return pollResult;
  }

  return genResult;
}

// Export for use in frontend
if (typeof window !== 'undefined') {
  window.WorldFactoryAPI = WorldFactoryAPI;
  window.worldAPI = worldAPI;
  window.fetchPlanetDataAPI = fetchPlanetDataAPI;
  window.fetchTimelineDataAPI = fetchTimelineDataAPI;
  window.fetchWondersDataAPI = fetchWondersDataAPI;
  window.fetchResourcesDataAPI = fetchResourcesDataAPI;
  window.generateWorldAPI = generateWorldAPI;
  window.currentWorldId = currentWorldId;
  window.currentSeed = currentSeed;
}