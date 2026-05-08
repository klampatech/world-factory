/**
 * ProceduralWorld API Integration Layer
 * Handles all API communication with the backend server
 */

const API_BASE_URL = '/api/v1';

// ============================================================================
// API Response Types
// ============================================================================

/**
 * @typedef {Object} WorldMetadata
 * @property {string} id - World UUID
 * @property {string} name - World display name
 * @property {string} seed - Generation seed
 * @property {number} width - World width in tiles
 * @property {number} height - World height in tiles
 * @property {string} created_at - ISO timestamp of creation
 * @property {WorldConfig} config - Generation configuration
 * @property {WorldStatus} status - Current generation/simulation status
 */

/**
 * @typedef {Object} WorldConfig
 * @property {number} elevation_scale - Terrain elevation multiplier
 * @property {number} temperature_scale - Temperature distribution scale
 * @property {number} moisture_scale - Precipitation distribution scale
 * @property {string} terrain_type - Base terrain generation algorithm
 * @property {number} biome_seed - Biome distribution seed offset
 * @property {number} tectonic_scale - Tectonic plate scale
 * @property {number} erosion_iterations - Number of erosion passes
 */

/**
 * @typedef {Object} WorldStatus
 * @property {string} phase - Current phase: 'idle' | 'generating' | 'ready' | 'simulating' | 'error'
 * @property {number} progress - Progress percentage 0-100
 * @property {string} message - Human-readable status message
 * @property {string} [error] - Error message if phase is 'error'
 */

/**
 * @typedef {Object} WorldMap
 * @property {string} world_id - Associated world UUID
 * @property {number} width - Map width
 * @property {number} height - Map height
 * @property {MapTile[]} tiles - Tile data array
 */

/**
 * @typedef {Object} MapTile
 * @property {number} x - X coordinate
 * @property {number} y - Y coordinate
 * @property {string} terrain - Terrain type (ocean, land, mountain, etc.)
 * @property {number} elevation - Elevation value 0-1
 * @property {number} temperature - Temperature value 0-1
 * @property {number} moisture - Moisture value 0-1
 * @property {string} [biome] - Biome type if generated
 */

/**
 * @typedef {Object} SimulationEvent
 * @property {string} id - Event UUID
 * @property {string} world_id - Associated world UUID
 * @property {number} tick - Simulation tick number
 * @property {string} type - Event type (migration, extinction, adaptation, etc.)
 * @property {string} description - Human-readable event description
 * @property {Object} data - Event-specific data
 * @property {string} timestamp - ISO timestamp
 */

/**
 * @typedef {Object} DashboardStats
 * @property {number} total_tiles - Total map tiles
 * @property {number} land_tiles - Land tile count
 * @property {number} water_tiles - Water tile count
 * @property {number} species_count - Number of species
 * @property {number} active_biomes - Number of active biomes
 * @property {Object} elevation_distribution - Elevation histogram
 * @property {Object} temperature_distribution - Temperature histogram
 * @property {Object} moisture_distribution - Moisture histogram
 */

/**
 * @typedef {Object} ApiError
 * @property {string} type - Error type code
 * @property {string} message - Human-readable error message
 */

// ============================================================================
// API Client Class
// ============================================================================

class WorldApiClient {
    constructor(baseUrl = API_BASE_URL) {
        this.baseUrl = baseUrl;
    }

    /**
     * Make an authenticated API request
     * @param {string} endpoint - API endpoint path
     * @param {Object} options - Fetch options
     * @returns {Promise<any>} Parsed JSON response
     */
    async request(endpoint, options = {}) {
        const url = `${this.baseUrl}${endpoint}`;
        
        const defaultHeaders = {
            'Content-Type': 'application/json',
            'Accept': 'application/json'
        };

        const response = await fetch(url, {
            ...options,
            headers: {
                ...defaultHeaders,
                ...options.headers
            }
        });

        if (!response.ok) {
            const errorBody = await response.json().catch(() => ({}));
            const error = new Error(errorBody.message || `HTTP ${response.status}`);
            error.status = response.status;
            error.type = errorBody.type || 'unknown_error';
            throw error;
        }

        // Handle 204 No Content
        if (response.status === 204) {
            return null;
        }

        return response.json();
    }

    // =========================================================================
    // World CRUD Operations
    // =========================================================================

    /**
     * List all worlds
     * @returns {Promise<WorldMetadata[]>}
     */
    async listWorlds() {
        return this.request('/worlds');
    }

    /**
     * Get a single world by ID
     * @param {string} worldId - World UUID (with or without 'world:' prefix)
     * @returns {Promise<WorldMetadata>}
     */
    async getWorld(worldId) {
        const normalizedId = normalizeWorldId(worldId);
        return this.request(`/worlds/${normalizedId}`);
    }

    /**
     * Create a new world
     * @param {Object} config - World creation configuration
     * @returns {Promise<WorldMetadata>}
     */
    async createWorld(config) {
        return this.request('/worlds', {
            method: 'POST',
            body: JSON.stringify(config)
        });
    }

    /**
     * Delete a world
     * @param {string} worldId - World UUID
     * @returns {Promise<void>}
     */
    async deleteWorld(worldId) {
        const normalizedId = normalizeWorldId(worldId);
        return this.request(`/worlds/${normalizedId}`, {
            method: 'DELETE'
        });
    }

    // =========================================================================
    // Map Operations
    // =========================================================================

    /**
     * Get world map data
     * @param {string} worldId - World UUID
     * @returns {Promise<WorldMap>}
     */
    async getWorldMap(worldId) {
        const normalizedId = normalizeWorldId(worldId);
        return this.request(`/worlds/${normalizedId}/map`);
    }

    // =========================================================================
    // Simulation Operations
    // =========================================================================

    /**
     * Advance world simulation
     * @param {string} worldId - World UUID
     * @param {number} [ticks=1] - Number of simulation ticks
     * @returns {Promise<{tick: number, events: SimulationEvent[]}>}
     */
    async simulate(worldId, ticks = 1) {
        const normalizedId = normalizeWorldId(worldId);
        return this.request(`/worlds/${normalizedId}/simulate`, {
            method: 'POST',
            body: JSON.stringify({ ticks })
        });
    }

    /**
     * Get simulation history
     * @param {string} worldId - World UUID
     * @param {Object} [options] - Query options
     * @param {number} [options.limit=100] - Max events to return
     * @param {number} [options.offset=0] - Offset for pagination
     * @returns {Promise<SimulationEvent[]>}
     */
    async getSimulationHistory(worldId, options = {}) {
        const normalizedId = normalizeWorldId(worldId);
        const params = new URLSearchParams({
            limit: options.limit || 100,
            offset: options.offset || 0
        });
        return this.request(`/worlds/${normalizedId}/history?${params}`);
    }

    // =========================================================================
    // Dashboard Statistics
    // =========================================================================

    /**
     * Get world dashboard statistics
     * @param {string} worldId - World UUID
     * @returns {Promise<DashboardStats>}
     */
    async getDashboardStats(worldId) {
        const normalizedId = normalizeWorldId(worldId);
        return this.request(`/worlds/${normalizedId}/stats`);
    }

    /**
     * Get active disasters for a world
     * @param {string} worldId - World UUID
     * @returns {Promise<Object[]>}
     */
    async getDisasters(worldId) {
        const normalizedId = normalizeWorldId(worldId);
        return this.request(`/worlds/${normalizedId}/disasters`);
    }

    /**
     * Get resource summary for a world
     * @param {string} worldId - World UUID
     * @returns {Promise<Object>}
     */
    async getResourceSummary(worldId) {
        const normalizedId = normalizeWorldId(worldId);
        return this.request(`/worlds/${normalizedId}/resources/summary`);
    }

    /**
     * Get notable figures for a world
     * @param {string} worldId - World UUID
     * @param {number} [limit=5] - Max number of figures to return
     * @returns {Promise<Object[]>}
     */
    async getNotableFigures(worldId, limit = 5) {
        const normalizedId = normalizeWorldId(worldId);
        return this.request(`/worlds/${normalizedId}/figures?limit=${limit}&sort=impact_score`);
    }

    // =========================================================================
    // Polling Helpers
    // =========================================================================

    /**
     * Poll world status until generation completes
     * @param {string} worldId - World UUID
     * @param {Function} onProgress - Callback with progress updates
     * @param {number} [timeout=300000] - Max wait time in ms
     * @returns {Promise<WorldMetadata>}
     */
    async waitForWorldReady(worldId, onProgress = null, timeout = 300000) {
        const startTime = Date.now();
        
        while (Date.now() - startTime < timeout) {
            const world = await this.getWorld(worldId);
            
            if (onProgress) {
                onProgress(world.status);
            }
            
            if (world.status.phase === 'ready') {
                return world;
            }
            
            if (world.status.phase === 'error') {
                throw new Error(world.status.error || 'World generation failed');
            }
            
            // Wait 1 second before next poll
            await sleep(1000);
        }
        
        throw new Error('World generation timed out');
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Normalize world ID to UUID format (strip 'world:' prefix if present)
 * @param {string} worldId - World ID in any format
 * @returns {string} Normalized UUID
 */
function normalizeWorldId(worldId) {
    if (!worldId) return '';
    return worldId.replace(/^world:/, '');
}

/**
 * Sleep for specified milliseconds
 * @param {number} ms - Milliseconds to sleep
 * @returns {Promise<void>}
 */
function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Format a date for display
 * @param {string} dateString - ISO date string
 * @returns {string} Formatted date string
 */
function formatDate(dateString) {
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
    });
}

/**
 * Format a relative time (e.g., "2 hours ago")
 * @param {string} dateString - ISO date string
 * @returns {string} Relative time string
 */
function formatRelativeTime(dateString) {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now - date;
    const diffSec = Math.floor(diffMs / 1000);
    const diffMin = Math.floor(diffSec / 60);
    const diffHour = Math.floor(diffMin / 60);
    const diffDay = Math.floor(diffHour / 24);

    if (diffDay > 0) return `${diffDay} day${diffDay > 1 ? 's' : ''} ago`;
    if (diffHour > 0) return `${diffHour} hour${diffHour > 1 ? 's' : ''} ago`;
    if (diffMin > 0) return `${diffMin} minute${diffMin > 1 ? 's' : ''} ago`;
    return 'Just now';
}

/**
 * Format a seed for display (truncated if too long)
 * @param {string|number} seed - World seed
 * @returns {string} Formatted seed
 */
function formatSeed(seed) {
    const str = String(seed);
    if (str.length > 16) {
        return str.substring(0, 8) + '...' + str.substring(str.length - 8);
    }
    return str;
}

/**
 * Get phase display name and color
 * @param {string} phase - World phase
 * @returns {{name: string, color: string, bgClass: string}}
 */
function getPhaseInfo(phase) {
    const phaseMap = {
        'idle': { name: 'Idle', color: '#6b7280', bgClass: 'bg-gray-500' },
        'generating': { name: 'Generating', color: '#3b82f6', bgClass: 'bg-blue-500' },
        'ready': { name: 'Ready', color: '#22c55e', bgClass: 'bg-green-500' },
        'simulating': { name: 'Simulating', color: '#a855f7', bgClass: 'bg-purple-500' },
        'error': { name: 'Error', color: '#ef4444', bgClass: 'bg-red-500' }
    };
    return phaseMap[phase] || { name: phase, color: '#6b7280', bgClass: 'bg-gray-500' };
}

// ============================================================================
// Standalone Export Functions (for HTML script module usage)
// ============================================================================

// Create a default client instance
const api = new WorldApiClient();

/**
 * Fetch all worlds from the server
 * @returns {Promise<Array>} Array of world objects
 */
async function fetchWorlds() {
    const response = await api.listWorlds();
    // API returns { success, data: { totalWorlds, worlds, pagination } }
    const worlds = response.data?.worlds || response.worlds || [];
    // Normalize API response to frontend format
    return worlds.map(normalizeWorld);
}

/**
 * Normalize API world response to frontend format
 * API uses: status (string), createdAt (camelCase), no config object
 * Frontend expects: status.phase, created_at (snake_case), config.prehistory_years
 */
function normalizeWorld(world) {
    return {
        id: world.id,
        name: world.name || 'Unnamed World',
        seed: world.seed || 0,
        width: world.width || world.mapWidth || 64,
        height: world.height || world.mapHeight || 64,
        created_at: world.createdAt || world.created_at || new Date().toISOString(),
        status: {
            phase: world.status || world.phase || 'idle',
            progress: world.progress || 0,
            message: world.message || ''
        },
        config: {
            prehistory_years: world.prehistoryYears || world.prehistory_years || 1000,
            elevation_scale: world.elevationScale || world.elevation_scale || 1.0,
            temperature_scale: world.temperatureScale || world.temperature_scale || 1.0,
            moisture_scale: world.moistureScale || world.moisture_scale || 1.0,
            terrain_type: world.terrainType || world.terrain_type || 'standard'
        },
        event_count: world.eventCount || world.event_count || 0
    };
}

/**
 * Fetch a single world by ID
 * @param {string} worldId - World UUID
 * @returns {Promise<Object>} World object
 */
async function fetchWorld(worldId) {
    return api.getWorld(worldId);
}

/**
 * Create a new world
 * @param {Object} config - World configuration
 * @returns {Promise<Object>} Created world object
 */
async function createWorld(config) {
    // Map frontend config format to backend API format
    const apiConfig = {
        name: config.name,
        seed: config.seed,
        width: config.width,
        height: config.height,
        prehistoryYears: config.prehistory_years || config.prehistoryYears,
        resourceRichness: config.resource_richness || config.resourceRichness,
        disasterFrequency: config.disaster_frequency || config.disasterFrequency
    };
    const response = await api.createWorld(apiConfig);
    return normalizeWorld(response);
}

/**
 * Delete a world by ID
 * @param {string} worldId - World UUID
 * @returns {Promise<void>}
 */
async function deleteWorld(worldId) {
    return api.deleteWorld(worldId);
}

/**
 * Advance world simulation
 * @param {string} worldId - World UUID
 * @param {number} years - Years to simulate
 * @returns {Promise<Object>} Simulation result
 */
async function simulateWorld(worldId, years = 100) {
    return api.simulate(worldId, years);
}

/**
 * Fetch map data for a world
 * @param {string} worldId - World UUID
 * @returns {Promise<Object>} Map data with polygons
 */
async function fetchMapData(worldId) {
    return api.getWorldMap(worldId);
}

/**
 * Check server health status
 * @returns {Promise<Object>} Health status
 */
async function checkHealth() {
    try {
        const response = await fetch('/health');
        if (response.ok) {
            return { status: 'healthy', online: true };
        }
        return { status: 'unhealthy', online: true, code: response.status };
    } catch (err) {
        return { status: 'offline', online: false, error: err.message };
    }
}

/**
 * Error class for API errors
 */
class ApiError extends Error {
    constructor(message, status, body) {
        super(message);
        this.name = 'ApiError';
        this.status = status;
        this.error = body?.error || body;
    }
    
    toJSON() {
        return {
            message: this.message,
            status: this.status,
            error: this.error
        };
    }
}

// ============================================================================
// Status Badge Helpers
// ============================================================================

const WORLD_STATUS = {
    GENERATING: 'generating',
    READY: 'ready',
    FAILED: 'failed',
    SIMULATING: 'simulating'
};

function getStatusClass(status) {
    switch (status) {
        case 'ready':
            return 'status-ready';
        case 'generating':
        case 'simulating':
            return 'status-generating';
        case 'error':
        case 'failed':
            return 'status-failed';
        default:
            return 'status-unknown';
    }
}

function formatStatus(status) {
    if (!status) return 'Unknown';
    // Handle phase/status mapping
    if (status === 'generating') return 'Generating';
    if (status === 'simulating') return 'Simulating';
    if (status === 'ready') return 'Ready';
    if (status === 'error' || status === 'failed') return 'Failed';
    if (status === 'idle') return 'Idle';
    return status.charAt(0).toUpperCase() + status.slice(1);
}

// ============================================================================
// Export for module usage
// ============================================================================

if (typeof module !== 'undefined' && module.exports) {
    module.exports = {
        WorldApiClient,
        normalizeWorldId,
        sleep,
        formatDate,
        formatRelativeTime,
        formatSeed,
        getPhaseInfo,
        // Standalone functions
        fetchWorlds,
        fetchWorld,
        createWorld,
        deleteWorld,
        simulateWorld,
        fetchMapData,
        checkHealth,
        ApiError,
        WORLD_STATUS,
        getStatusClass,
        formatStatus
    };
}
