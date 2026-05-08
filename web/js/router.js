// ============================================================================
// Hash-Based Router
// ============================================================================

// Route definitions
const ROUTES = {
    HOME: '',           // #/ - World Selector
    WORLD: 'worlds/:id',  // #/worlds/:id - World detail with optional tab
};

// Parse hash and extract route parameters
function parseHash() {
    const hash = window.location.hash || '';
    
    // Handle empty hash or just '#'
    if (!hash || hash === '#') {
        return { route: 'home', params: {} };
    }
    
    // Remove leading # or #!/
    let path = hash.replace(/^#!?\/?/, ''); // Handle both #/ and #!/ formats
    const segments = path.split('/').filter(Boolean);
    
    // Route: #/ or #!/ -> World Selector
    if (segments.length === 0 || (segments.length === 1 && segments[0] === '')) {
        return { route: 'home', params: {} };
    }
    
    // Route: #/worlds/:id or #!/worlds/:id -> World detail
    if (segments[0] === 'worlds' && segments.length >= 2) {
        const params = { id: segments[1] };
        // Optional tab parameter
        if (segments[2]) {
            params.tab = segments[2];
        }
        return { route: 'world', params };
    }
    
    // Unknown route, fallback to home
    return { route: 'home', params: {} };
}

// Navigate to a route
function navigate(path) {
    window.location.hash = path;
}

// Navigate to world detail
function navigateToWorld(worldId, tab) {
    const basePath = `/worlds/${worldId}`;
    const path = tab ? `${basePath}/${tab}` : basePath;
    navigate(path);
}

// Navigate to world selector
function navigateToHome() {
    navigate('/');
}

// Handle route changes
function handleRoute() {
    const { route, params } = parseHash();
    
    switch (route) {
        case 'home':
            showWorldSelector();
            break;
        case 'world':
            showWorldDetail(params.id, params.tab);
            break;
        default:
            showWorldSelector();
    }
}

// Show World Selector view (landing page)
function showWorldSelector() {
    const worldSelectorView = document.getElementById('view-world-selector');
    const worldDetailView = document.getElementById('view-world-detail');
    const tabsContainer = document.getElementById('tabs-container');
    const backLink = document.getElementById('back-link');
    const headerNav = document.getElementById('header-nav');
    
    // Show world selector, hide world detail and tabs
    if (worldSelectorView) worldSelectorView.style.display = 'block';
    if (worldDetailView) worldDetailView.style.display = 'none';
    if (tabsContainer) tabsContainer.style.display = 'none';
    if (backLink) backLink.style.display = 'none';
    if (headerNav) headerNav.style.display = 'none';
    
    // Update page title
    document.getElementById('page-title').textContent = 'World Selector';
    
    // Load worlds if not loaded
    if (state.worlds.length === 0) {
        loadWorlds();
    }
    
    // Stop world polling
    window.stopPolling?.();
}

// Show World Detail view
async function showWorldDetail(worldId, tab) {
    const worldSelectorView = document.getElementById('view-world-selector');
    const worldDetailView = document.getElementById('view-world-detail');
    const tabsContainer = document.getElementById('tabs-container');
    const backLink = document.getElementById('back-link');
    const headerNav = document.getElementById('header-nav');
    
    // Update state
    state.worldId = worldId;
    state.world = null;
    state.map = null;
    state.events = [];
    state.stats = null;
    
    // Show world detail and tabs, hide world selector
    if (worldSelectorView) worldSelectorView.style.display = 'none';
    if (worldDetailView) worldDetailView.style.display = 'block';
    if (tabsContainer) tabsContainer.style.display = 'block';
    if (backLink) backLink.style.display = 'inline-flex';
    if (headerNav) headerNav.style.display = 'flex';
    
    // Update page title
    document.getElementById('page-title').textContent = 'Loading...';
    
    // Load world data
    await loadWorld();
    
    // If no tab specified, default to overview
    const activeTab = tab || 'overview';
    
    // Switch to the specified tab
    switchTab(activeTab);
}

// Switch to a specific tab
function switchTab(tabId) {
    const tabButtons = document.querySelectorAll('.tab-button');
    const tabPanels = document.querySelectorAll('.tab-panel');
    const dashboardNavLink = document.getElementById('nav-dashboard');
    
    // Update tab buttons
    tabButtons.forEach(btn => {
        const isActive = btn.dataset.tab === tabId;
        btn.classList.toggle('active', isActive);
        btn.setAttribute('aria-selected', isActive ? 'true' : 'false');
    });
    
    // Update tab panels
    tabPanels.forEach(panel => {
        const isActive = panel.id === `panel-${tabId}`;
        panel.classList.toggle('active', isActive);
    });
    
    // Update Dashboard nav link with current world ID
    if (dashboardNavLink && state.worldId) {
        dashboardNavLink.href = `#/worlds/${state.worldId}/dashboard`;
    }
    
    // Load tab-specific content
    loadTabContent(tabId);
    
    // Update URL hash to reflect tab change
    if (state.worldId) {
        const newHash = `#/worlds/${state.worldId}/${tabId}`;
        if (window.location.hash !== newHash) {
            // Use replaceState to avoid adding history entries for tab switches
            history.replaceState(null, '', newHash);
        }
    }
}

// Initialize router
function initRouter() {
    // Handle initial route
    handleRoute();
    
    // Listen for hash changes (browser back/forward)
    window.addEventListener('hashchange', handleRoute);
}

// Expose routing functions globally for onclick handlers
// Note: stopPolling will be defined in World Detail script and accessed via window.stopPolling
