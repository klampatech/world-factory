// ============================================================================
// State Management
// ============================================================================

const state = {
    worlds: [],
    world: null,
    worldId: null,
    serverOnline: false,
    pollingInterval: null,
    events: [],
    figures: [],
    map: null,
    stats: null
};

// Initialize API client (api instance already created in api-integration.js)

// ============================================================================
// Initialization
// ============================================================================

document.addEventListener('DOMContentLoaded', () => {
    // Initialize router FIRST (before other init)
    initRouter();
    
    // Check server health first
    checkServerStatus();
    
    // Setup generate button and modal controls first (before async operations)
    setupModalControls();
    
    // Setup sliders
    setupSlider('width-slider', 'width-display', 'width-value');
    setupSlider('height-slider', 'height-display', 'height-value');
    setupSlider('years-slider', 'years-display', 'years-value');
    
    // Initialize tabs
    initTabs();
    // NOTE: World loading and polling are now handled by the router
    // based on the current route (see initRouter in hash-router script above)
});

function setupModalControls() {
    const generateBtn = document.getElementById('generate-btn');
    const emptyGenerateBtn = document.getElementById('empty-generate-btn');
    const modalCloseBtn = document.getElementById('modal-close');
    const modalCancelBtn = document.getElementById('modal-cancel');
    const modalCreateBtn = document.getElementById('modal-create');
    const modalOverlay = document.getElementById('generate-modal');
    
    if (generateBtn) {
        generateBtn.addEventListener('click', openGenerateModal);
    }
    if (emptyGenerateBtn) {
        emptyGenerateBtn.addEventListener('click', openGenerateModal);
    }
    if (modalCloseBtn) {
        modalCloseBtn.addEventListener('click', closeGenerateModal);
    }
    if (modalCancelBtn) {
        modalCancelBtn.addEventListener('click', closeGenerateModal);
    }
    if (modalCreateBtn) {
        modalCreateBtn.addEventListener('click', createNewWorld);
    }
    if (modalOverlay) {
        modalOverlay.addEventListener('click', (e) => {
            if (e.target.id === 'generate-modal') closeGenerateModal();
        });
    }
}

function setupSlider(sliderId, displayId, valueId) {
    const slider = document.getElementById(sliderId);
    const display = document.getElementById(displayId);
    const value = document.getElementById(valueId);
    if (slider) {
        slider.addEventListener('input', () => {
            display.textContent = slider.value;
            value.textContent = slider.value;
        });
    }
}

// ============================================================================
// Server Status
// ============================================================================

async function checkServerStatus() {
    try {
        await checkHealth();
        state.serverOnline = true;
        updateServerStatus(true);
    } catch (e) {
        state.serverOnline = false;
        updateServerStatus(false);
    }
}

function updateServerStatus(online) {
    const statusEl = document.getElementById('server-status');
    const textEl = document.getElementById('server-status-text');
    if (statusEl && textEl) {
        statusEl.className = 'server-status ' + (online ? 'online' : 'offline');
        textEl.textContent = online ? 'Server Online' : 'Server Offline';
    }
}

// ============================================================================
// World List Loading
// ============================================================================

async function loadWorlds() {
    const loadingState = document.getElementById('loading-state');
    const emptyState = document.getElementById('empty-state');
    const worldGrid = document.getElementById('world-grid');
    
    if (loadingState) loadingState.style.display = 'block';
    
    try {
        if (state.serverOnline) {
            state.worlds = await fetchWorlds();
        } else {
            // Use demo worlds when server is offline
            state.worlds = getDemoWorlds();
        }
    } catch (error) {
        console.error('Failed to load worlds:', error);
        state.worlds = getDemoWorlds();
    }
    
    if (loadingState) loadingState.style.display = 'none';
    
    renderWorldList();
}

function renderWorldList() {
    const emptyState = document.getElementById('empty-state');
    const worldGrid = document.getElementById('world-grid');
    
    if (state.worlds.length === 0) {
        if (emptyState) emptyState.style.display = 'block';
        if (worldGrid) worldGrid.style.display = 'none';
        return;
    }
    
    if (emptyState) emptyState.style.display = 'none';
    if (worldGrid) {
        worldGrid.style.display = 'grid';
        worldGrid.innerHTML = state.worlds.map(world => renderWorldCard(world)).join('');
    }
}

function renderWorldCard(world) {
    const phaseInfo = getPhaseInfo(world.status?.phase || 'idle');
    const createdDate = world.created_at ? new Date(world.created_at) : new Date();
    const age = formatRelativeTime(createdDate.toISOString());
    
    return `
        <div class="world-list-card" data-world-id="${world.id}">
            <div class="world-list-card-header">
                <div>
                    <h3 class="world-name">${world.name || 'Unnamed World'}</h3>
                    <span class="world-id">${world.id?.substring(0, 8) || '—'}...</span>
                </div>
                <span class="status-badge ${world.status?.phase || 'idle'}">
                    <span class="status-dot" style="background: ${phaseInfo.color}"></span>
                    ${phaseInfo.name}
                </span>
            </div>
            <div class="world-list-card-body">
                <div class="metadata-grid" style="grid-template-columns: repeat(2, 1fr);">
                    <div class="metadata-item">
                        <span class="metadata-label">Dimensions</span>
                        <span class="metadata-value">${world.width || 64} × ${world.height || 64}</span>
                    </div>
                    <div class="metadata-item">
                        <span class="metadata-label">Pre-History</span>
                        <span class="metadata-value">${world.config?.prehistory_years || 1000} years</span>
                    </div>
                    <div class="metadata-item">
                        <span class="metadata-label">Events</span>
                        <span class="metadata-value">${world.event_count || 0}</span>
                    </div>
                    <div class="metadata-item">
                        <span class="metadata-label">Age</span>
                        <span class="metadata-value">${age}</span>
                    </div>
                </div>
            </div>
            <div class="world-list-card-footer">
                <button class="view-btn" onclick="viewWorld('${world.id}', 'map')">View Map</button>
                <button class="view-btn" onclick="viewWorld('${world.id}', 'timeline')">View Timeline</button>
                <button class="view-btn" onclick="viewWorld('${world.id}', 'dashboard')">View Dashboard</button>
            </div>
        </div>
    `;
}

function viewWorld(worldId, tab) {
    // Use hash-based navigation instead of page navigation
    navigateToWorld(worldId, tab);
}

// Compatibility wrapper for old link format
function openWorld(id) {
    navigateToWorld(id);
}

// ============================================================================
// Generate New World Modal
// ============================================================================

function openGenerateModal() {
    const modal = document.getElementById('generate-modal');
    if (modal) {
        modal.classList.add('active');
        
        // Add escape key handler
        const closeOnEscape = (e) => {
            if (e.key === 'Escape') {
                closeGenerateModal();
                document.removeEventListener('keydown', closeOnEscape);
            }
        };
        document.addEventListener('keydown', closeOnEscape);
    }
}

function closeGenerateModal() {
    const modal = document.getElementById('generate-modal');
    if (modal) modal.classList.remove('active');
}

async function createNewWorld() {
    const nameInput = document.getElementById('world-name-input');
    const seedInput = document.getElementById('world-seed-input');
    const createBtn = document.getElementById('modal-create');
    
    const name = nameInput?.value?.trim() || generateWorldName();
    const seedStr = seedInput?.value?.trim();
    const width = parseInt(document.getElementById('width-slider')?.value || 64);
    const height = parseInt(document.getElementById('height-slider')?.value || 64);
    const prehistoryYears = parseInt(document.getElementById('years-slider')?.value || 1000);
    const resourceRichness = document.getElementById('resource-richness')?.value || 'medium';
    const disasterFreq = document.getElementById('disaster-freq')?.value || 'medium';
    
    // Get selected species
    const speciesCheckboxes = document.querySelectorAll('input[name="species"]:checked');
    const species = Array.from(speciesCheckboxes).map(cb => cb.value);
    
    if (createBtn) {
        createBtn.disabled = true;
        createBtn.textContent = 'Generating...';
    }
    
    try {
        const config = {
            name,
            seed: seedStr ? parseInt(seedStr) : undefined,
            width,
            height,
            prehistory_years: prehistoryYears,
            resource_richness: resourceRichness,
            disaster_frequency: disasterFreq,
            species
        };
        
        const newWorld = await createWorld(config);
        closeGenerateModal();
        
        // Reload world list
        await loadWorlds();
        
        // Reset modal
        if (nameInput) nameInput.value = '';
        if (seedInput) seedInput.value = '';
        if (createBtn) {
            createBtn.disabled = false;
            createBtn.textContent = 'Generate World';
        }
    } catch (error) {
        console.error('Failed to create world:', error);
        alert('Failed to create world. Please try again.');
        if (createBtn) {
            createBtn.disabled = false;
            createBtn.textContent = 'Generate World';
        }
    }
}

function generateWorldName() {
    const adjectives = ['Ancient', 'Mystic', 'Eternal', 'Primeval', 'Cosmic', 'Verdant', 'Frozen', 'Burning'];
    const nouns = ['Terra', 'Gaia', 'World', 'Realm', 'Sphere', 'Domain', 'Expanse', 'Horizon'];
    const adj = adjectives[Math.floor(Math.random() * adjectives.length)];
    const noun = nouns[Math.floor(Math.random() * nouns.length)];
    return `${adj} ${noun}`;
}

// stopPolling is defined in World Detail script and exposed via window
// Expose routing functions globally for onclick handlers

// ============================================================================
// Tab Navigation
// ============================================================================

function initTabs() {
    const tabButtons = document.querySelectorAll('.tab-button');
    
    tabButtons.forEach(button => {
        button.addEventListener('click', () => {
            const tabId = button.dataset.tab;
            
            // Switch tab (this also updates the URL via the router)
            switchTab(tabId);
        });
    });
}

function loadTabContent(tabId) {
    switch (tabId) {
        case 'map':
            if (!state.map) loadMapData();
            break;
        case 'timeline':
            if (state.events.length === 0) loadTimeline();
            break;
        case 'dashboard':
            if (!state.stats) loadDashboard();
            break;
    }
}

// Expose routing functions globally for onclick handlers
// Also expose polling controls for router access
window.navigateToWorld = navigateToWorld;
window.navigateToHome = navigateToHome;
window.switchTab = switchTab;

// ============================================================================
// World Data Loading
// ============================================================================

async function loadWorld() {
    try {
        // For demo purposes, use sample data if API fails
        try {
            state.world = await api.getWorld(state.worldId);
        } catch (e) {
            // Use demo data when API is unavailable
            state.world = getDemoWorld();
        }
        
        renderWorldMetadata();
        
        // Start polling if world is still generating/simulating
        if (['generating', 'simulating'].includes(state.world?.status?.phase)) {
            startPolling();
        }
        
    } catch (error) {
        console.error('Failed to load world:', error);
        showError('Failed to load world data');
    }
}

function renderWorldMetadata() {
    const world = state.world;
    
    // Update header
    document.getElementById('page-title').textContent = world.name;
    document.getElementById('world-name').textContent = world.name;
    document.getElementById('world-id').textContent = `world:${world.id}`;
    
    // Update status badge
    const phaseInfo = getPhaseInfo(world.status.phase);
    const badge = document.getElementById('status-badge');
    badge.className = `status-badge ${world.status.phase}`;
    badge.querySelector('.status-dot').style.background = phaseInfo.color;
    document.getElementById('status-text').textContent = phaseInfo.name;
    
    // Update metadata
    document.getElementById('meta-dimensions').textContent = `${world.width} × ${world.height}`;
    document.getElementById('meta-seed').textContent = formatSeed(world.seed);
    document.getElementById('meta-created').textContent = formatDate(world.created_at);
    document.getElementById('meta-age').textContent = formatRelativeTime(world.created_at);
    
    // Update progress bar
    const progressSection = document.getElementById('progress-section');
    if (world.status.phase === 'generating' || world.status.phase === 'simulating') {
        progressSection.style.display = 'block';
        document.getElementById('progress-bar').style.width = `${world.status.progress}%`;
        document.getElementById('progress-message').textContent = world.status.message || 'Processing...';
        document.getElementById('progress-percent').textContent = `${Math.round(world.status.progress)}%`;
    } else {
        progressSection.style.display = 'none';
    }
    
    // Update configuration
    if (world.config) {
        document.getElementById('cfg-elevation').textContent = world.config.elevation_scale?.toFixed(2) ?? '—';
        document.getElementById('cfg-temperature').textContent = world.config.temperature_scale?.toFixed(2) ?? '—';
        document.getElementById('cfg-moisture').textContent = world.config.moisture_scale?.toFixed(2) ?? '—';
        document.getElementById('cfg-terrain').textContent = world.config.terrain_type ?? '—';
        document.getElementById('cfg-biome').textContent = world.config.biome_seed ?? '—';
        document.getElementById('cfg-tectonic').textContent = world.config.tectonic_scale?.toFixed(2) ?? '—';
        document.getElementById('cfg-erosion').textContent = world.config.erosion_iterations ?? '—';
    }
}

// ============================================================================
// Map Tab
// ============================================================================

async function loadMapData() {
    const loading = document.getElementById('map-loading');
    
    try {
        state.map = await api.getWorldMap(state.worldId);
        renderMap();
    } catch (error) {
        console.error('Failed to load map:', error);
        state.map = getDemoMap();
        renderMap();
    } finally {
        loading.style.display = 'none';
    }
}

function renderMap() {
    const canvas = document.getElementById('world-map');
    const ctx = canvas.getContext('2d');
    
    // Set canvas size
    const container = canvas.parentElement;
    canvas.width = container.clientWidth;
    canvas.height = container.clientHeight;
    
    if (!state.map || !state.map.tiles) {
        // Draw placeholder
        ctx.fillStyle = '#1a1a2e';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        ctx.fillStyle = '#ffffff';
        ctx.font = '16px sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText('No map data available', canvas.width / 2, canvas.height / 2);
        return;
    }
    
    const tiles = state.map.tiles;
    const tileWidth = canvas.width / state.map.width;
    const tileHeight = canvas.height / state.map.height;
    
    // Color mapping for terrain types
    const terrainColors = {
        'ocean': '#1e3a5f',
        'shallow_water': '#2d5a87',
        'beach': '#c2b280',
        'grassland': '#7cb342',
        'forest': '#2e7d32',
        'dense_forest': '#1b5e20',
        'desert': '#d4a574',
        'mountain': '#757575',
        'high_mountain': '#9e9e9e',
        'snow': '#ffffff',
        'tundra': '#b0bec5'
    };
    
    // Render tiles
    tiles.forEach(tile => {
        const x = tile.x * tileWidth;
        const y = tile.y * tileHeight;
        
        // Get color based on terrain or elevation
        let color;
        if (tile.terrain && terrainColors[tile.terrain]) {
            color = terrainColors[tile.terrain];
        } else if (tile.elevation !== undefined) {
            // Gradient from deep water to mountain
            if (tile.elevation < 0.3) {
                color = interpolateColor('#1e3a5f', '#c2b280', tile.elevation / 0.3);
            } else if (tile.elevation < 0.7) {
                color = interpolateColor('#c2b280', '#2e7d32', (tile.elevation - 0.3) / 0.4);
            } else {
                color = interpolateColor('#2e7d32', '#ffffff', (tile.elevation - 0.7) / 0.3);
            }
        } else {
            color = '#333';
        }
        
        ctx.fillStyle = color;
        ctx.fillRect(x, y, tileWidth + 1, tileHeight + 1);
    });
}

function interpolateColor(color1, color2, factor) {
    const c1 = hexToRgb(color1);
    const c2 = hexToRgb(color2);
    const r = Math.round(c1.r + (c2.r - c1.r) * factor);
    const g = Math.round(c1.g + (c2.g - c1.g) * factor);
    const b = Math.round(c1.b + (c2.b - c1.b) * factor);
    return `rgb(${r}, ${g}, ${b})`;
}

function hexToRgb(hex) {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16)
    } : { r: 0, g: 0, b: 0 };
}

// ============================================================================
// Timeline Tab
// ============================================================================

async function loadTimeline() {
    try {
        state.events = await api.getSimulationHistory(state.worldId);
    } catch (error) {
        console.error('Failed to load timeline:', error);
        state.events = getDemoEvents();
    }
    
    // Sort events by tick (descending - newest first)
    state.events.sort((a, b) => b.tick - a.tick);
    
    renderTimeline();
}

function renderTimeline() {
    const container = document.getElementById('timeline-content');
    
    if (state.events.length === 0) {
        container.innerHTML = `
            <div class="empty-state">
                <div class="empty-state-icon">📜</div>
                <p>No simulation events yet</p>
                <button class="btn btn-primary" style="margin-top: 16px;" id="simulate-btn">
                    Run Simulation
                </button>
            </div>
        `;
        document.getElementById('simulate-btn')?.addEventListener('click', runSimulation);
        return;
    }
    
    // Get unique event types for filter
    const eventTypes = [...new Set(state.events.map(e => e.type))];
    
    // Build filter types list (All + unique types)
    const filterOptions = ['all', ...eventTypes].map(type => 
        `<option value="${type}">${type === 'all' ? 'All Types' : type.charAt(0).toUpperCase() + type.slice(1)}</option>`
    ).join('');
    
    // Get total events count
    const totalEvents = state.events.length;
    const minTick = state.events[state.events.length - 1]?.tick || 0;
    const maxTick = state.events[0]?.tick || 0;
    
    const timelineWrapper = document.createElement('div');
    timelineWrapper.innerHTML = `
        <!-- Search and Filter Controls -->
        <div class="timeline-search">
            <input type="text" class="timeline-search-input" 
                   id="timeline-search-input" 
                   placeholder="Search events... (e.g., 'migration', 'figure name')">
            <select class="timeline-filter" id="timeline-type-filter">
                ${filterOptions}
            </select>
            <select class="timeline-filter" id="timeline-year-filter">
                <option value="all">All Years</option>
                <option value="recent">Recent (Last 50)</option>
                <option value="early">Early History</option>
            </select>
        </div>
        
        <!-- Timeline Stats -->
        <div class="timeline-stats">
            <span>Showing <strong id="timeline-showing-count">${totalEvents}</strong> events</span>
            <span>Years <strong>${minTick}</strong> to <strong>${maxTick}</strong></span>
        </div>
        
        <!-- Timeline Container -->
        <div class="timeline" id="timeline-events-list"></div>
        
        <!-- Simulate Button -->
        <div style="margin-top: 24px;">
            <button class="btn btn-primary" id="simulate-btn">
                Run Simulation
            </button>
        </div>
    `;
    
    container.innerHTML = '';
    container.appendChild(timelineWrapper);
    
    // Setup event listeners
    setupTimelineSearch();
    
    document.getElementById('simulate-btn')?.addEventListener('click', runSimulation);
    
    // Initial render of all events
    renderTimelineEvents(state.events);
}

function setupTimelineSearch() {
    const searchInput = document.getElementById('timeline-search-input');
    const typeFilter = document.getElementById('timeline-type-filter');
    const yearFilter = document.getElementById('timeline-year-filter');
    
    const filterEvents = () => {
        const searchTerm = searchInput?.value?.toLowerCase() || '';
        const selectedType = typeFilter?.value || 'all';
        const yearRange = yearFilter?.value || 'all';
        
        let filtered = state.events;
        
        // Filter by type
        if (selectedType !== 'all') {
            filtered = filtered.filter(e => e.type === selectedType);
        }
        
        // Filter by year range
        if (yearRange === 'recent') {
            const maxTick = Math.max(...state.events.map(e => e.tick));
            filtered = filtered.filter(e => e.tick >= maxTick - 50);
        } else if (yearRange === 'early') {
            const minTick = Math.min(...state.events.map(e => e.tick));
            const maxTick = Math.max(...state.events.map(e => e.tick));
            filtered = filtered.filter(e => e.tick <= minTick + (maxTick - minTick) * 0.25);
        }
        
        // Filter by search term
        if (searchTerm) {
            filtered = filtered.filter(e => 
                e.description?.toLowerCase().includes(searchTerm) ||
                e.type?.toLowerCase().includes(searchTerm) ||
                e.affected_entities?.some(ent => ent.toLowerCase().includes(searchTerm))
            );
        }
        
        // Update count
        const countEl = document.getElementById('timeline-showing-count');
        if (countEl) countEl.textContent = filtered.length;
        
        renderTimelineEvents(filtered);
    };
    
    searchInput?.addEventListener('input', debounce(filterEvents, 300));
    typeFilter?.addEventListener('change', filterEvents);
    yearFilter?.addEventListener('change', filterEvents);
}

function renderTimelineEvents(events) {
    const container = document.getElementById('timeline-events-list');
    if (!container) return;
    
    container.innerHTML = '';
    
    if (events.length === 0) {
        container.innerHTML = `
            <div class="no-events-message">
                <p>No events match your search criteria</p>
            </div>
        `;
        return;
    }
    
    events.forEach(event => {
        const eventEl = document.createElement('div');
        eventEl.className = 'timeline-event';
        
        // Highlight figure names in description
        const descriptionHtml = highlightFigureLinks(event.description, event.affected_entities);
        
        eventEl.innerHTML = `
            <div class="event-header" onclick="toggleEventExpand(this)">
                <span class="event-type">${event.type}</span>
                <div style="display: flex; align-items: center; gap: 8px;">
                    <span class="event-tick">Year ${event.year || event.tick}</span>
                    <span class="event-expand-icon">▼</span>
                </div>
            </div>
            <p class="event-description">${descriptionHtml}</p>
            <span class="event-time">${formatRelativeTime(event.timestamp)}</span>
            <div class="event-expanded-content">
                <div class="event-details">
                    ${event.significance ? `
                    <div class="event-detail-item">
                        <span class="event-detail-label">Significance</span>
                        <span class="event-detail-value">${(event.significance * 100).toFixed(0)}%</span>
                    </div>
                    ` : ''}
                    ${event.affected_entities?.length ? `
                    <div class="event-detail-item">
                        <span class="event-detail-label">Affected</span>
                        <span class="event-detail-value">${event.affected_entities.length} entities</span>
                    </div>
                    ` : ''}
                    ${event.figures?.length ? `
                    <div class="event-detail-item">
                        <span class="event-detail-label">Figures</span>
                        <span class="event-detail-value">${event.figures.length} involved</span>
                    </div>
                    ` : ''}
                </div>
                ${event.affected_entities?.length ? `
                <div style="margin-top: 12px;">
                    <span class="event-detail-label">Involved Entities:</span>
                    <div style="margin-top: 6px; display: flex; flex-wrap: wrap; gap: 6px;">
                        ${event.affected_entities.map(entity => `
                            <span class="view-btn" onclick="showFigureBiography('${entity}')" style="cursor: pointer;">
                                ${entity}
                            </span>
                        `).join('')}
                    </div>
                </div>
                ` : ''}
                <span class="expand-hint">Click header to collapse</span>
            </div>
        `;
        container.appendChild(eventEl);
    });
}

function toggleEventExpand(headerElement) {
    const eventEl = headerElement.closest('.timeline-event');
    eventEl.classList.toggle('expanded');
}

function highlightFigureLinks(description, affectedEntities) {
    if (!affectedEntities?.length) return description;
    
    let result = description;
    affectedEntities.forEach(entity => {
        // Match entity name in description (case-insensitive, whole word)
        const regex = new RegExp(`(${entity})`, 'gi');
        result = result.replace(regex, '<span class="figure-link" onclick="showFigureBiography(\'$1\')">$1</span>');
    });
    return result;
}

function showFigureBiography(figureId) {
    // Find figure in events data or state
    const figure = findFigureById(figureId);
    
    if (!figure) {
        showError('Figure not found');
        return;
    }
    
    // Create and show biography modal
    const modalContent = `
        <div class="biography-header">
            <div class="biography-avatar">${figure.name?.charAt(0) || '?'}</div>
            <div class="biography-info">
                <h3>${figure.name || 'Unknown Figure'}</h3>
                <p class="biography-meta">${figure.figure_type || 'Historical Figure'} • ${figure.species_id || 'Unknown species'}</p>
            </div>
        </div>
        <div class="biography-section">
            <h4>Life Statistics</h4>
            <div class="biography-stats">
                <div class="biography-stat">
                    <div class="biography-stat-label">Birth Year</div>
                    <div class="biography-stat-value">${figure.birth_year || '?'}</div>
                </div>
                <div class="biography-stat">
                    <div class="biography-stat-label">Death Year</div>
                    <div class="biography-stat-value">${figure.death_year || figure.deathYear || 'Present'}</div>
                </div>
                <div class="biography-stat">
                    <div class="biography-stat-label">Lifespan</div>
                    <div class="biography-stat-value">${calculateLifespan(figure)} years</div>
                </div>
                <div class="biography-stat">
                    <div class="biography-stat-label">Significance</div>
                    <div class="biography-stat-value">${((figure.significance || 0.5) * 100).toFixed(0)}%</div>
                </div>
            </div>
        </div>
        ${figure.achievements?.length ? `
        <div class="biography-section">
            <h4>Notable Achievements</h4>
            <div class="biography-achievements">
                ${figure.achievements.map(a => `
                    <div class="biography-achievement">
                        <span class="achievement-icon">🏆</span>
                        <span>${a}</span>
                    </div>
                `).join('')}
            </div>
        </div>
        ` : ''}
        ${figure.description ? `
        <div class="biography-section">
            <h4>Biography</h4>
            <p>${figure.description}</p>
        </div>
        ` : ''}
    `;
    
    showModal('Figure Biography', modalContent, 'biography-modal');
}

function findFigureById(figureId) {
    // Search in state data
    if (state.figures) {
        const found = state.figures.find(f => f.id === figureId || f.name === figureId);
        if (found) return found;
    }
    
    // Search in events affected entities
    for (const event of state.events || []) {
        if (event.figures) {
            const found = event.figures.find(f => f.id === figureId || f.name === figureId);
            if (found) return found;
        }
    }
    
    // Create a placeholder figure from entity name
    return {
        id: figureId,
        name: figureId,
        figure_type: 'Historical Figure',
        birth_year: 0,
        significance: 0.5
    };
}

function calculateLifespan(figure) {
    const birth = figure.birth_year || figure.birthYear || 0;
    const death = figure.death_year || figure.deathYear || (figure.year || figure.tick);
    return death - birth || '?';
}

function showModal(title, content, extraClass = '') {
    // Remove existing modal if any
    const existingModal = document.querySelector('.modal-overlay.biography-modal');
    if (existingModal) existingModal.remove();
    
    const modal = document.createElement('div');
    modal.className = `modal-overlay biography-modal ${extraClass}`;
    modal.innerHTML = `
        <div class="modal">
            <div class="modal-header">
                <h2>${title}</h2>
                <button class="modal-close" onclick="this.closest('.modal-overlay').remove()">&times;</button>
            </div>
            <div class="modal-body">
                ${content}
            </div>
        </div>
    `;
    
    document.body.appendChild(modal);
    
    // Close on overlay click
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });
    
    // Close on Escape
    const closeOnEscape = (e) => {
        if (e.key === 'Escape') {
            modal.remove();
            document.removeEventListener('keydown', closeOnEscape);
        }
    };
    document.addEventListener('keydown', closeOnEscape);
    
    modal.classList.add('active');
}

function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

async function runSimulation() {
    const btn = document.getElementById('simulate-btn');
    btn.disabled = true;
    btn.innerHTML = '<span class="loading-spinner"></span> Simulating...';
    
    try {
        await api.simulate(state.worldId, 10);
        
        // Reload timeline
        state.events = [];
        await loadTimeline();
        
    } catch (error) {
        console.error('Simulation failed:', error);
        showError('Simulation failed. Please try again.');
    } finally {
        btn.disabled = false;
    }
}

// ============================================================================
// Dashboard Tab
// ============================================================================

// Dashboard state
const dashboardState = {
    disasters: [],
    resources: [],
    figures: [],
    recentEvents: [],
    populationBySpecies: []
};

async function loadDashboard() {
    // Load all dashboard data in parallel
    const [statsResult, disastersResult, resourcesResult, figuresResult] = await Promise.allSettled([
        api.getDashboardStats(state.worldId).catch(() => null),
        api.getDisasters(state.worldId).catch(() => []),
        api.getResourceSummary(state.worldId).catch(() => null),
        api.getNotableFigures(state.worldId, 5).catch(() => [])
    ]);
    
    state.stats = statsResult.value || getDemoStats();
    dashboardState.disasters = disastersResult.value || [];
    dashboardState.resources = resourcesResult.value?.resources || [];
    dashboardState.figures = figuresResult.value || [];
    dashboardState.populationBySpecies = statsResult.value?.population_by_species || getDemoPopulationBySpecies();
    
    // Load recent events from timeline
    if (state.events.length === 0) {
        try {
            state.events = await api.getSimulationHistory(state.worldId, { limit: 10 });
        } catch (e) {
            state.events = getDemoEvents().slice(0, 10);
        }
    }
    dashboardState.recentEvents = state.events.slice(0, 10);
    
    renderDashboard();
}

function renderDashboard() {
    const stats = state.stats;
    const currentYear = state.world?.current_year || state.world?.year || 1000;
    
    // Update current year display
    document.getElementById('current-year').textContent = currentYear.toLocaleString();
    
    // Update stats
    document.getElementById('stat-total').textContent = formatNumber(stats.total_tiles || 0);
    document.getElementById('stat-land').textContent = formatNumber(stats.land_tiles || 0);
    document.getElementById('stat-water').textContent = formatNumber(stats.water_tiles || 0);
    document.getElementById('stat-species').textContent = stats.species_count || stats.population_by_species?.length || 0;
    document.getElementById('stat-disasters').textContent = dashboardState.disasters.length;
    
    renderDisasters();
    renderSpeciesPieChart();
    renderResourceChart();
    renderFiguresSpotlight();
    renderRecentEvents();
}

function renderDisasters() {
    const container = document.getElementById('disasters-section');
    const listEl = document.getElementById('disasters-list');
    const disasters = dashboardState.disasters;
    
    if (!disasters || disasters.length === 0) {
        container.style.display = 'none';
        return;
    }
    
    container.style.display = 'block';
    listEl.innerHTML = disasters.map(d => `
        <div class="disaster-item">
            <span class="disaster-icon">${getDisasterIcon(d.type)}</span>
            <div class="disaster-info">
                <div class="disaster-name">${d.name || d.type}</div>
                <div class="disaster-type">${d.type} - ${d.affected_area || 'Unknown area'}</div>
            </div>
            <span class="disaster-severity ${d.severity || 'medium'}">${d.severity || 'Medium'}</span>
        </div>
    `).join('');
}

function getDisasterIcon(type) {
    const icons = {
        earthquake: '\ud83c\udf0e',
        flood: '\ud83c\udf3a',
        drought: '\ud83c\udf21\ufe0f',
        wildfire: '\ud83d\udd25',
        volcanic: '\ud83c\udf0b',
        plague: '\ud83e\ude7a',
        famine: '\ud83e\uded2',
        storm: '\u26c5',
        default: '\ud83d\udea8'
    };
    return icons[type?.toLowerCase()] || icons.default;
}

function renderSpeciesPieChart() {
    const canvas = document.getElementById('species-pie-chart');
    const ctx = canvas.getContext('2d');
    const legend = document.getElementById('species-legend');
    const data = dashboardState.populationBySpecies;
    
    if (!data || data.length === 0) {
        ctx.fillStyle = '#9ca3af';
        ctx.beginPath();
        ctx.arc(100, 100, 80, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = 'white';
        ctx.font = '14px sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText('No Data', 100, 105);
        return;
    }
    
    const colors = ['#3b82f6', '#22c55e', '#f59e0b', '#ef4444', '#8b5cf6', '#ec4899', '#06b6d4', '#84cc16'];
    const total = data.reduce((sum, d) => sum + (d.population || d.count || 0), 0);
    let startAngle = 0;
    
    canvas.width = 200;
    canvas.height = 200;
    ctx.clearRect(0, 0, 200, 200);
    
    legend.innerHTML = data.map((d, i) => {
        const percentage = total > 0 ? ((d.population || d.count || 0) / total * 100).toFixed(1) : 0;
        return `
            <div class="pie-legend-item">
                <div class="pie-legend-color" style="background: ${colors[i % colors.length]}"></div>
                <span>${d.species || d.name}: ${percentage}%</span>
            </div>
        `;
    }).join('');
    
    data.forEach((d, i) => {
        const value = d.population || d.count || 0;
        const sliceAngle = (value / total) * Math.PI * 2;
        
        ctx.fillStyle = colors[i % colors.length];
        ctx.beginPath();
        ctx.moveTo(100, 100);
        ctx.arc(100, 100, 80, startAngle, startAngle + sliceAngle);
        ctx.closePath();
        ctx.fill();
        
        startAngle += sliceAngle;
    });
}

function renderResourceChart() {
    const container = document.getElementById('resource-chart');
    const data = dashboardState.resources;
    const colors = ['#3b82f6', '#22c55e', '#f59e0b', '#8b5cf6', '#ef4444'];
    
    if (!data || data.length === 0) {
        // Demo data
        const demoResources = [
            { name: 'Minerals', amount: 4500 },
            { name: 'Timber', amount: 3200 },
            { name: 'Water', amount: 5800 },
            { name: 'Food', amount: 4100 },
            { name: 'Energy', amount: 2800 }
        ];
        container.innerHTML = renderResourceBars(demoResources, colors);
        return;
    }
    
    container.innerHTML = renderResourceBars(data, colors);
}

function renderResourceBars(data, colors) {
    const max = Math.max(...data.map(d => d.amount || d.value || 0));
    return data.map((r, i) => {
        const value = r.amount || r.value || 0;
        const percentage = max > 0 ? (value / max) * 100 : 0;
        return `
            <div style="display: flex; align-items: center; gap: 8px;">
                <span style="width: 60px; font-size: 12px;">${r.name || r.type}</span>
                <div style="flex: 1; background: #e5e7eb; border-radius: 4px; height: 20px;">
                    <div style="width: ${percentage}%; height: 100%; background: ${colors[i % colors.length]}; border-radius: 4px;"></div>
                </div>
                <span style="width: 50px; text-align: right; font-size: 12px;">${formatNumber(value)}</span>
            </div>
        `;
    }).join('');
}

function renderFiguresSpotlight() {
    const container = document.getElementById('figures-section');
    const spotlight = document.getElementById('figures-spotlight');
    const figures = dashboardState.figures;
    
    if (!figures || figures.length === 0) {
        container.style.display = 'none';
        return;
    }
    
    container.style.display = 'block';
    spotlight.innerHTML = figures.map(f => `
        <div class="figure-card" onclick="showFigureBiography('${f.id || f.name}')">
            <div class="figure-avatar">${(f.name || '?').charAt(0).toUpperCase()}</div>
            <div class="figure-info">
                <div class="figure-name">${f.name || 'Unknown'}</div>
                <div class="figure-title">${f.title || f.role || f.figure_type || 'Historical Figure'}</div>
                <div class="figure-stats">
                    <div class="figure-stat">
                        <span class="figure-stat-label">Impact</span>
                        <span class="figure-stat-value">${((f.impact_score || f.significance || 0.5) * 100).toFixed(0)}%</span>
                    </div>
                    <div class="figure-stat">
                        <span class="figure-stat-label">Born</span>
                        <span class="figure-stat-value">${f.birth_year || '?'}</span>
                    </div>
                </div>
            </div>
        </div>
    `).join('');
}

function renderRecentEvents() {
    const container = document.getElementById('recent-events-list');
    const events = dashboardState.recentEvents;
    
    if (!events || events.length === 0) {
        container.innerHTML = '<div class="empty-state"><p>No recent events</p></div>';
        return;
    }
    
    container.innerHTML = events.map(e => `
        <div class="recent-event-item">
            <span class="recent-event-icon">${getEventIcon(e.type)}</span>
            <div class="recent-event-content">
                <div class="recent-event-type">${e.type}</div>
                <div class="recent-event-desc">${truncate(e.description, 80)}</div>
            </div>
            <span class="recent-event-year">Year ${e.year || e.tick || '?'}</span>
        </div>
    `).join('');
}

function getEventIcon(type) {
    const icons = {
        migration: '\ud83c\udfe0',
        founding: '\ud83c\udff4\u200d\ud83c\udffb',
        war: '\u2694\ufe0f',
        discovery: '\ud83d\udd2d',
        climate: '\ud83c\udf21\ufe0f',
        extinction: '\ud83e\ude7a',
        adaptation: '\ud83e\udeb4',
        trade: '\ud83d\uded2',
        default: '\ud83d\udccb'
    };
    return icons[type?.toLowerCase()] || icons.default;
}

function truncate(str, len) {
    if (!str) return '';
    return str.length > len ? str.substring(0, len) + '...' : str;
}

function getDemoPopulationBySpecies() {
    return [
        { species: 'Humans', population: 45000 },
        { species: 'Elves', population: 12000 },
        { species: 'Dwarves', population: 8000 },
        { species: 'Orcs', population: 15000 },
        { species: 'Others', population: 5000 }
    ];
}

function renderBarChart(containerId, data) {
    const container = document.getElementById(containerId);
    if (!container) return;
    container.innerHTML = '';
    
    const max = Math.max(...data);
    
    data.forEach(value => {
        const bar = document.createElement('div');
        bar.className = 'bar';
        bar.style.height = `${(value / max) * 100}%`;
        container.appendChild(bar);
    });
}

// ============================================================================
// Polling for Status Updates
// ============================================================================

function startPolling() {
    // Only poll on world detail view
    const { route } = parseHash();
    if (route !== 'world') return;
    
    if (!state.worldId) return;
    if (state.pollingInterval) return;
    
    state.pollingInterval = setInterval(async () => {
        // Check we're still on world detail view
        const { route: currentRoute } = parseHash();
        if (currentRoute !== 'world') {
            stopPolling();
            return;
        }
        
        try {
            state.world = await api.getWorld(state.worldId);
            renderWorldMetadata();
            
            // Stop polling when ready or error
            if (['ready', 'error'].includes(state.world?.status?.phase)) {
                stopPolling();
            }
        } catch (error) {
            console.error('Polling failed:', error);
        }
    }, 2000);
}

function stopPolling() {
    if (state.pollingInterval) {
        clearInterval(state.pollingInterval);
        state.pollingInterval = null;
    }
}

// Expose stopPolling for hash router script access
window.stopPolling = stopPolling;

// ============================================================================
// Utility Functions
// ============================================================================

function formatNumber(num) {
    if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M';
    if (num >= 1000) return (num / 1000).toFixed(1) + 'K';
    return num.toString();
}

function showError(message) {
    // Could implement a toast notification here
    console.error(message);
}

// ============================================================================
// Helper Functions
// ============================================================================

function getPhaseInfo(phase) {
    const phases = {
        'idle': { name: 'Idle', color: '#9ca3af' },
        'generating': { name: 'Generating', color: '#3b82f6' },
        'ready': { name: 'Ready', color: '#22c55e' },
        'simulating': { name: 'Simulating', color: '#8b5cf6' },
        'error': { name: 'Error', color: '#ef4444' }
    };
    return phases[phase] || phases['idle'];
}

function formatRelativeTime(isoString) {
    if (!isoString) return '—';
    const date = new Date(isoString);
    const now = new Date();
    const diffMs = now - date;
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);
    
    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 30) return `${diffDays}d ago`;
    return date.toLocaleDateString();
}

function formatDate(isoString) {
    if (!isoString) return '—';
    return new Date(isoString).toLocaleDateString();
}

function formatSeed(seed) {
    if (!seed) return '—';
    return seed.toString();
}

function getDemoWorlds() {
    return [
        {
            id: 'b9aea887-f2de-4c2d-800d-be9f25362caa',
            name: 'Terra Prime',
            seed: 1778079711,
            width: 64,
            height: 64,
            created_at: new Date(Date.now() - 86400000 * 3).toISOString(),
            status: { phase: 'ready', progress: 100 },
            config: { prehistory_years: 1000 },
            event_count: 42
        },
        {
            id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
            name: 'Nordenmark',
            seed: 12345678,
            width: 128,
            height: 128,
            created_at: new Date(Date.now() - 86400000).toISOString(),
            status: { phase: 'generating', progress: 67, message: 'Generating terrain...' },
            config: { prehistory_years: 5000 },
            event_count: 0
        },
        {
            id: 'f1e2d3c4-b5a6-7890-1234-567890abcdef',
            name: 'Verdant Expanse',
            seed: 987654321,
            width: 64,
            height: 64,
            created_at: new Date(Date.now() - 86400000 * 7).toISOString(),
            status: { phase: 'simulating', progress: 100 },
            config: { prehistory_years: 2000 },
            event_count: 156
        }
    ];
}

function getDemoWorld() {
    return {
        id: 'b9aea887-f2de-4c2d-800d-be9f25362caa',
        name: 'Terra Prime',
        seed: 1778079711,
        width: 256,
        height: 256,
        created_at: new Date(Date.now() - 86400000 * 3).toISOString(),
        status: {
            phase: 'ready',
            progress: 100,
            message: 'World generation complete'
        },
        config: {
            elevation_scale: 1.5,
            temperature_scale: 1.0,
            moisture_scale: 1.2,
            terrain_type: 'hexagonal',
            biome_seed: 42,
            tectonic_scale: 0.8,
            erosion_iterations: 100
        }
    };
}

function getDemoMap() {
    const tiles = [];
    const width = 32;
    const height = 18;
    
    for (let y = 0; y < height; y++) {
        for (let x = 0; x < width; x++) {
            // Generate some noise-like pattern
            const elevation = Math.sin(x * 0.3) * Math.cos(y * 0.3) * 0.5 + 0.5;
            
            tiles.push({
                x,
                y,
                elevation,
                terrain: elevation < 0.3 ? 'ocean' : elevation < 0.4 ? 'beach' : 
                         elevation < 0.7 ? 'grassland' : elevation < 0.85 ? 'mountain' : 'snow'
            });
        }
    }
    
    return {
        world_id: state.worldId,
        width,
        height,
        tiles
    };
}

function getDemoEvents() {
    return [
        {
            id: '1',
            tick: 100,
            year: 100,
            type: 'migration',
            description: 'Korrath the Brave led a migration of humans from the eastern plains to settle new territories.',
            affected_entities: ['Korrath the Brave', 'Human Tribe Alpha'],
            significance: 0.85,
            figures: [
                { id: 'Korrath the Brave', name: 'Korrath the Brave', figure_type: 'Leader', species_id: 'human', birth_year: 40, significance: 0.85 }
            ],
            timestamp: new Date(Date.now() - 3600000).toISOString()
        },
        {
            id: '2',
            tick: 95,
            year: 95,
            type: 'climate',
            description: 'Temperature increased by 2°C across the northern regions, causing seasonal shifts.',
            affected_entities: ['Northern Tundra', 'Frost Peaks'],
            significance: 0.45,
            timestamp: new Date(Date.now() - 7200000).toISOString()
        },
        {
            id: '3',
            tick: 90,
            year: 90,
            type: 'extinction',
            description: 'The Aquatic Serpent species went extinct due to changing ocean currents.',
            affected_entities: ['Aquatic Serpent', 'Ocean Depths'],
            significance: 0.60,
            timestamp: new Date(Date.now() - 10800000).toISOString()
        },
        {
            id: '4',
            tick: 85,
            year: 85,
            type: 'founding',
            description: 'Thelmor the Elder established the settlement of Stonehearth in the mountain foothills.',
            affected_entities: ['Thelmor the Elder', 'Stonehearth', 'Dwarf Clan Stonehammer'],
            significance: 0.90,
            figures: [
                { id: 'Thelmor the Elder', name: 'Thelmor the Elder', figure_type: 'Founder', species_id: 'dwarf', birth_year: 50, significance: 0.90 }
            ],
            timestamp: new Date(Date.now() - 14400000).toISOString()
        },
        {
            id: '5',
            tick: 80,
            year: 80,
            type: 'war',
            description: 'A territorial conflict erupted between the Orc clans over control of the Fertile Valley.',
            affected_entities: ['Orc Clan Redfang', 'Orc Clan Blackscalp', 'Fertile Valley'],
            significance: 0.75,
            timestamp: new Date(Date.now() - 18000000).toISOString()
        },
        {
            id: '6',
            tick: 75,
            year: 75,
            type: 'discovery',
            description: 'Elven explorers discovered the Ancient Ruins hidden within the Dense Forest.',
            affected_entities: ['Aelindra of the Green', 'Dense Forest', 'Ancient Ruins'],
            significance: 0.80,
            figures: [
                { id: 'Aelindra of the Green', name: 'Aelindra of the Green', figure_type: 'Explorer', species_id: 'elf', birth_year: 40, significance: 0.80 }
            ],
            timestamp: new Date(Date.now() - 21600000).toISOString()
        }
    ];
}

function getDemoStats() {
    return {
        total_tiles: 65536,
        land_tiles: 45875,
        water_tiles: 19661,
        species_count: 42,
        active_biomes: 8,
        elevation_distribution: [0.05, 0.1, 0.25, 0.35, 0.2, 0.05],
        temperature_distribution: [0.15, 0.25, 0.3, 0.2, 0.1]
    };
}

// ============================================================================
// Window Resize Handler for Map
// ============================================================================

let resizeTimeout;
window.addEventListener('resize', () => {
    clearTimeout(resizeTimeout);
    resizeTimeout = setTimeout(() => {
        if (state.map) renderMap();
    }, 250);
});
