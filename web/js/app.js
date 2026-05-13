/**
 * World Factory - Main Application
 * World selector page (index.html) initialization and state management
 */

// ============================================================================
// Application State
// ============================================================================

// Global state object (primary reference)
const appState = window.appState = {
    worlds: [],
    serverOnline: false,
    pollingInterval: null,
    modalOpen: false
};

// Alias for backward compatibility with existing code that uses 'state'
const state = appState;

// ============================================================================
// Initialization
// ============================================================================

document.addEventListener('DOMContentLoaded', () => {
    // Check server health first
    checkServerStatus();
    
    // Setup UI controls
    setupModalControls();
    setupSliders();
    
    // Load world list
    loadWorlds();
    
    // Start polling for status updates
    startPolling();
});

// ============================================================================
// Server Status
// ============================================================================

async function checkServerStatus() {
    try {
        await api.checkHealth();
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

function startPolling() {
    if (state.pollingInterval) return;
    state.pollingInterval = setInterval(async () => {
        try {
            await api.checkHealth();
            state.serverOnline = true;
            updateServerStatus(true);
        } catch (e) {
            state.serverOnline = false;
            updateServerStatus(false);
        }
    }, 30000);
}

function stopPolling() {
    if (state.pollingInterval) {
        clearInterval(state.pollingInterval);
        state.pollingInterval = null;
    }
}

// ============================================================================
// Modal Controls
// ============================================================================

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

function setupSliders() {
    setupSlider('width-slider', 'width-display', 'width-value');
    setupSlider('height-slider', 'height-display', 'height-value');
    setupSlider('years-slider', 'years-display', 'years-value');
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
// World List Loading
// ============================================================================

async function loadWorlds() {
    const loadingState = document.getElementById('loading-state');
    const emptyState = document.getElementById('empty-state');
    const worldGrid = document.getElementById('world-grid');
    
    if (loadingState) loadingState.style.display = 'block';
    
    try {
        if (state.serverOnline) {
            state.worlds = await api.fetchWorlds();
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
    // Use clean URL paths per SPEC.md §6.0
    window.location.href = `/worlds/${worldId}/${tab}`;
}

// ============================================================================
// Generate New World Modal
// ============================================================================

function openGenerateModal() {
    const modal = document.getElementById('generate-modal');
    if (modal) {
        modal.classList.add('active');
        state.modalOpen = true;
        
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
    state.modalOpen = false;
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
        
        const newWorld = await api.createWorld(config);
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
