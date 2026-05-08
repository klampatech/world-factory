/**
 * World Factory - Dashboard Module
 * World statistics, charts, and summary views
 */

// ============================================================================
// Dashboard State
// ============================================================================

const dashboardState = {
    world: null,
    stats: null,
    planet: null,
    societies: [],
    figures: []
};

// ============================================================================
// Dashboard Initialization
// ============================================================================

function initDashboard(containerId, worldId, options = {}) {
    const container = document.getElementById(containerId);
    if (!container) return null;
    
    return {
        container,
        worldId,
        options,
        state: { ...dashboardState },
        
        async load() {
            try {
                // Load all dashboard data in parallel
                const [world, societies, figures] = await Promise.all([
                    api.getWorld(this.worldId),
                    api.getSocieties(this.worldId).catch(() => []),
                    api.getFigures(this.worldId).catch(() => [])
                ]);
                
                this.state.world = world;
                this.state.societies = societies;
                this.state.figures = figures;
                
                // Calculate stats from world data
                this.calculateStats();
                
                this.render();
            } catch (error) {
                console.error('Failed to load dashboard:', error);
                this.showError('Failed to load dashboard data');
            }
        },
        
        calculateStats() {
            const world = this.state.world;
            
            this.state.stats = {
                totalTiles: (world.width || 64) * (world.height || 64),
                dimensions: `${world.width || 64} × ${world.height || 64}`,
                generationStatus: world.status?.phase || 'unknown',
                seed: world.seed,
                createdAt: world.created_at,
                prehistoryYears: world.config?.prehistory_years || 0,
                eventCount: world.event_count || 0,
                speciesCount: world.config?.species?.length || 0,
                figureCount: this.state.figures.length,
                societyCount: this.state.societies.length
            };
        },
        
        render() {
            if (!this.state.stats) return;
            
            this.container.innerHTML = this.renderDashboardHTML();
            this.renderCharts();
        },
        
        renderDashboardHTML() {
            const stats = this.state.stats;
            const figures = this.state.figures.slice(0, 5); // Top 5 figures
            
            return `
                <div class="stats-grid">
                    <div class="stat-card">
                        <div class="stat-value">${stats.totalTiles.toLocaleString()}</div>
                        <div class="stat-label">Total Tiles</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">${stats.dimensions}</div>
                        <div class="stat-label">Dimensions</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">${stats.eventCount.toLocaleString()}</div>
                        <div class="stat-label">Events</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">${stats.figureCount}</div>
                        <div class="stat-label">Notable Figures</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">${stats.societyCount}</div>
                        <div class="stat-label">Societies</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value">${stats.speciesCount}</div>
                        <div class="stat-label">Species</div>
                    </div>
                </div>
                
                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px;">
                    ${this.renderNotableFiguresSection(figures)}
                    ${this.renderSocietiesSection()}
                </div>
                
                <div style="margin-top: 24px;">
                    ${this.renderWorldInfoSection()}
                </div>
            `;
        },
        
        renderNotableFiguresSection(figures) {
            if (figures.length === 0) {
                return `
                    <div class="chart-container">
                        <div class="chart-title">Notable Figures</div>
                        <p style="color: var(--color-text-muted); text-align: center; padding: 20px;">
                            No notable figures yet
                        </p>
                    </div>
                `;
            }
            
            return `
                <div class="chart-container">
                    <div class="chart-title">Notable Figures</div>
                    <div style="display: flex; flex-direction: column; gap: 12px;">
                        ${figures.map(figure => `
                            <div class="biography-achievement" style="cursor: pointer;" 
                                 onclick="showFigureDetails('${figure.id}')">
                                <div class="biography-avatar" style="width: 40px; height: 40px; font-size: 16px;">
                                    ${figure.name?.charAt(0) || '?'}
                                </div>
                                <div style="flex: 1;">
                                    <div style="font-weight: 600;">${figure.name || 'Unknown'}</div>
                                    <div style="font-size: 12px; color: var(--color-text-muted);">
                                        ${figure.title || 'Notable Figure'} · Influence: ${figure.influence || 0}
                                    </div>
                                </div>
                            </div>
                        `).join('')}
                    </div>
                </div>
            `;
        },
        
        renderSocietiesSection() {
            const societies = this.state.societies;
            
            if (societies.length === 0) {
                return `
                    <div class="chart-container">
                        <div class="chart-title">Societies</div>
                        <p style="color: var(--color-text-muted); text-align: center; padding: 20px;">
                            No societies formed yet
                        </p>
                    </div>
                `;
            }
            
            return `
                <div class="chart-container">
                    <div class="chart-title">Societies</div>
                    <div style="display: flex; flex-direction: column; gap: 8px;">
                        ${societies.map(society => `
                            <div style="display: flex; justify-content: space-between; padding: 8px 12px; 
                                        background: var(--color-bg); border-radius: var(--radius-md);">
                                <span style="font-weight: 500;">${society.name || 'Unknown Society'}</span>
                                <span style="color: var(--color-text-muted);">
                                    Pop: ${society.population?.toLocaleString() || 0}
                                </span>
                            </div>
                        `).join('')}
                    </div>
                </div>
            `;
        },
        
        renderWorldInfoSection() {
            const world = this.state.world;
            
            return `
                <div class="chart-container">
                    <div class="chart-title">World Configuration</div>
                    <div class="config-grid" style="grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));">
                        <div class="config-item">
                            <span class="config-label">World Name</span>
                            <span class="config-value">${world.name || 'Unknown'}</span>
                        </div>
                        <div class="config-item">
                            <span class="config-label">Seed</span>
                            <span class="config-value mono">${world.seed || '—'}</span>
                        </div>
                        <div class="config-item">
                            <span class="config-label">Pre-History Years</span>
                            <span class="config-value">${world.config?.prehistory_years || 0}</span>
                        </div>
                        <div class="config-item">
                            <span class="config-label">Resource Richness</span>
                            <span class="config-value">${world.config?.resource_richness || 'normal'}</span>
                        </div>
                        <div class="config-item">
                            <span class="config-label">Disaster Frequency</span>
                            <span class="config-value">${world.config?.disaster_frequency || 'medium'}</span>
                        </div>
                        <div class="config-item">
                            <span class="config-label">Generation Status</span>
                            <span class="config-value">${world.status?.phase || 'unknown'}</span>
                        </div>
                    </div>
                </div>
            `;
        },
        
        renderCharts() {
            // Charts are rendered via CSS in this simple version
            // Can be extended with Chart.js or similar
        },
        
        showError(message) {
            this.container.innerHTML = `
                <div class="empty-state">
                    <div class="empty-state-icon">⚠️</div>
                    <p>${message}</p>
                </div>
            `;
        }
    };
}

// ============================================================================
// Global Functions
// ============================================================================

function showFigureDetails(figureId) {
    const figure = dashboardState.figures.find(f => f.id === figureId);
    if (figure) {
        showBiographyModal(figure);
    }
}

// ============================================================================
// Simple Charts (Pure CSS)
// ============================================================================

function renderSimpleBarChart(containerId, data, options = {}) {
    const container = document.getElementById(containerId);
    if (!container) return;
    
    const maxValue = Math.max(...data.map(d => d.value));
    
    container.innerHTML = data.map(item => {
        const percentage = (item.value / maxValue) * 100;
        return `
            <div style="display: flex; align-items: center; gap: 8px;">
                <div style="width: 80px; font-size: 12px; color: var(--color-text-muted);">
                    ${item.label}
                </div>
                <div style="flex: 1; background: var(--color-border); border-radius: 4px; height: 16px;">
                    <div style="width: ${percentage}%; background: ${options.color || 'var(--color-primary)'}; 
                                height: 100%; border-radius: 4px;"></div>
                </div>
                <div style="width: 50px; font-size: 12px; text-align: right;">
                    ${item.value}
                </div>
            </div>
        `;
    }).join('');
}

function renderSimplePieChart(containerId, data, options = {}) {
    const container = document.getElementById(containerId);
    if (!container) return;
    
    const total = data.reduce((sum, item) => sum + item.value, 0);
    const colors = options.colors || ['#3b82f6', '#22c55e', '#f59e0b', '#ef4444', '#8b5cf6'];
    
    // Create HTML with legend
    let html = `<div style="display: flex; flex-wrap: wrap; gap: 16px;">`;
    
    data.forEach((item, index) => {
        const percentage = total > 0 ? (item.value / total * 100).toFixed(1) : 0;
        const color = colors[index % colors.length];
        
        html += `
            <div style="display: flex; align-items: center; gap: 8px;">
                <div style="width: 16px; height: 16px; background: ${color}; border-radius: 4px;"></div>
                <span style="font-size: 13px;">${item.label}: ${percentage}%</span>
            </div>
        `;
    });
    
    html += `</div>`;
    
    container.innerHTML = html;
}
