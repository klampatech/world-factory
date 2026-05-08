/**
 * World Factory - Timeline Module
 * History event timeline with search and filtering
 */

// ============================================================================
// Timeline State
// ============================================================================

const timelineState = {
    events: [],
    figures: [],
    filteredEvents: [],
    searchQuery: '',
    typeFilter: 'all',
    yearFilter: 'all',
    selectedEvent: null,
    selectedFigure: null
};

// ============================================================================
// Timeline Initialization
// ============================================================================

function initTimeline(containerId, worldId, options = {}) {
    const container = document.getElementById(containerId);
    if (!container) return null;
    
    return {
        container,
        worldId,
        options,
        state: { ...timelineState },
        
        async load() {
            try {
                this.state.events = await api.getSimulationHistory(this.worldId);
                this.state.filteredEvents = [...this.state.events].sort((a, b) => b.tick - a.tick);
                this.render();
            } catch (error) {
                console.error('Failed to load timeline:', error);
                this.showError('Failed to load timeline events');
            }
        },
        
        render() {
            if (this.state.events.length === 0) {
                this.renderEmpty();
                return;
            }
            
            this.container.innerHTML = this.renderTimelineHTML();
            this.setupEventListeners();
        },
        
        renderEmpty() {
            this.container.innerHTML = `
                <div class="empty-state">
                    <div class="empty-state-icon">📜</div>
                    <p>No simulation events yet</p>
                    ${this.options.showSimulateButton !== false ? `
                        <button class="btn btn-primary" id="timeline-simulate-btn" style="margin-top: 16px;">
                            Run Simulation
                        </button>
                    ` : ''}
                </div>
            `;
            
            document.getElementById('timeline-simulate-btn')?.addEventListener('click', () => {
                this.handleSimulate();
            });
        },
        
        renderTimelineHTML() {
            const eventTypes = [...new Set(this.state.events.map(e => e.type))];
            const minTick = this.state.events[this.state.events.length - 1]?.tick || 0;
            const maxTick = this.state.events[0]?.tick || 0;
            
            const filterOptions = ['all', ...eventTypes].map(type => 
                `<option value="${type}">${type === 'all' ? 'All Types' : type.charAt(0).toUpperCase() + type.slice(1)}</option>`
            ).join('');
            
            return `
                <div class="timeline-search">
                    <input type="text" class="timeline-search-input" 
                           id="timeline-search-input" 
                           placeholder="Search events..."
                           value="${this.state.searchQuery}">
                    <select class="timeline-filter" id="timeline-type-filter">
                        ${filterOptions}
                    </select>
                    <select class="timeline-filter" id="timeline-year-filter">
                        <option value="all">All Years</option>
                        <option value="recent">Recent (Last 50)</option>
                        <option value="early">Early History</option>
                    </select>
                </div>
                
                <div class="timeline-stats">
                    <span>Showing <strong id="timeline-showing-count">${this.state.filteredEvents.length}</strong> events</span>
                    <span>Years <strong>${minTick}</strong> to <strong>${maxTick}</strong></span>
                </div>
                
                <div class="timeline" id="timeline-events-list">
                    ${this.state.filteredEvents.slice(0, 100).map(event => this.renderEventHTML(event)).join('')}
                </div>
                
                ${this.state.filteredEvents.length > 100 ? `
                    <div style="text-align: center; padding: 16px;">
                        <p style="color: var(--color-text-muted);">Showing first 100 of ${this.state.filteredEvents.length} events</p>
                    </div>
                ` : ''}
                
                ${this.options.showSimulateButton !== false ? `
                    <div style="margin-top: 24px;">
                        <button class="btn btn-primary" id="timeline-simulate-btn">
                            Run Simulation
                        </button>
                    </div>
                ` : ''}
            `;
        },
        
        renderEventHTML(event) {
            const phaseInfo = getPhaseInfo(event.type);
            const figureLinks = event.figures?.map(f => 
                `<span class="figure-link" data-figure-id="${f.id}">${f.name}</span>`
            ).join(', ') || '';
            
            return `
                <div class="timeline-event" data-event-id="${event.id}">
                    <div class="event-header" onclick="toggleEventExpand(this)">
                        <div>
                            <span class="event-type">${event.type}</span>
                            <span class="expand-hint">click to expand</span>
                        </div>
                        <div style="display: flex; align-items: center; gap: 8px;">
                            <span class="event-tick">Year ${event.tick}</span>
                            <span class="event-expand-icon">▼</span>
                        </div>
                    </div>
                    <div class="event-description">
                        ${event.description}${figureLinks ? ` (${figureLinks})` : ''}
                    </div>
                    <div class="event-expanded-content">
                        <div class="event-details">
                            ${event.location ? `
                                <div class="event-detail-item">
                                    <span class="event-detail-label">Location</span>
                                    <span class="event-detail-value">${event.location}</span>
                                </div>
                            ` : ''}
                            ${event.participants?.length ? `
                                <div class="event-detail-item">
                                    <span class="event-detail-label">Participants</span>
                                    <span class="event-detail-value">${event.participants.join(', ')}</span>
                                </div>
                            ` : ''}
                            ${event.impact ? `
                                <div class="event-detail-item">
                                    <span class="event-detail-label">Impact</span>
                                    <span class="event-detail-value">${event.impact}</span>
                                </div>
                            ` : ''}
                        </div>
                    </div>
                </div>
            `;
        },
        
        setupEventListeners() {
            const searchInput = document.getElementById('timeline-search-input');
            const typeFilter = document.getElementById('timeline-type-filter');
            const yearFilter = document.getElementById('timeline-year-filter');
            
            if (searchInput) {
                searchInput.addEventListener('input', (e) => {
                    this.state.searchQuery = e.target.value;
                    this.applyFilters();
                });
            }
            
            if (typeFilter) {
                typeFilter.value = this.state.typeFilter;
                typeFilter.addEventListener('change', (e) => {
                    this.state.typeFilter = e.target.value;
                    this.applyFilters();
                });
            }
            
            if (yearFilter) {
                yearFilter.addEventListener('change', (e) => {
                    this.state.yearFilter = e.target.value;
                    this.applyFilters();
                });
            }
            
            document.getElementById('timeline-simulate-btn')?.addEventListener('click', () => {
                this.handleSimulate();
            });
            
            // Figure link clicks
            document.querySelectorAll('.figure-link').forEach(link => {
                link.addEventListener('click', (e) => {
                    const figureId = e.target.dataset.figureId;
                    if (figureId) {
                        this.handleFigureClick(figureId);
                    }
                });
            });
        },
        
        applyFilters() {
            let filtered = [...this.state.events];
            
            // Type filter
            if (this.state.typeFilter !== 'all') {
                filtered = filtered.filter(e => e.type === this.state.typeFilter);
            }
            
            // Year filter
            if (this.state.yearFilter === 'recent') {
                const sorted = [...filtered].sort((a, b) => b.tick - a.tick);
                filtered = sorted.slice(0, 50);
            } else if (this.state.yearFilter === 'early') {
                const sorted = [...filtered].sort((a, b) => a.tick - b.tick);
                filtered = sorted.slice(0, 50);
            }
            
            // Search filter
            if (this.state.searchQuery) {
                const query = this.state.searchQuery.toLowerCase();
                filtered = filtered.filter(e => 
                    e.description?.toLowerCase().includes(query) ||
                    e.type?.toLowerCase().includes(query) ||
                    e.location?.toLowerCase().includes(query) ||
                    e.figures?.some(f => f.name?.toLowerCase().includes(query))
                );
            }
            
            // Sort by tick descending
            filtered.sort((a, b) => b.tick - a.tick);
            
            this.state.filteredEvents = filtered;
            this.updateShowingCount();
            this.updateEventList();
        },
        
        updateShowingCount() {
            const countEl = document.getElementById('timeline-showing-count');
            if (countEl) {
                countEl.textContent = this.state.filteredEvents.length;
            }
        },
        
        updateEventList() {
            const listEl = document.getElementById('timeline-events-list');
            if (listEl) {
                listEl.innerHTML = this.state.filteredEvents.slice(0, 100).map(e => this.renderEventHTML(e)).join('');
                
                // Reattach figure link listeners
                document.querySelectorAll('.figure-link').forEach(link => {
                    link.addEventListener('click', (e) => {
                        const figureId = e.target.dataset.figureId;
                        if (figureId) {
                            this.handleFigureClick(figureId);
                        }
                    });
                });
            }
        },
        
        async handleSimulate() {
            try {
                const btn = document.getElementById('timeline-simulate-btn');
                if (btn) {
                    btn.disabled = true;
                    btn.textContent = 'Simulating...';
                }
                
                await api.simulate(this.worldId);
                await this.load();
                
                if (btn) {
                    btn.disabled = false;
                    btn.textContent = 'Run Simulation';
                }
            } catch (error) {
                console.error('Simulation failed:', error);
                alert('Simulation failed. Please try again.');
                const btn = document.getElementById('timeline-simulate-btn');
                if (btn) {
                    btn.disabled = false;
                    btn.textContent = 'Run Simulation';
                }
            }
        },
        
        async handleFigureClick(figureId) {
            if (this.options.onFigureClick) {
                this.options.onFigureClick(figureId);
            }
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
// Global Functions (called from inline onclick)
// ============================================================================

function toggleEventExpand(headerElement) {
    const eventElement = headerElement.closest('.timeline-event');
    if (eventElement) {
        eventElement.classList.toggle('expanded');
    }
}

// ============================================================================
// Figure Biography Modal
// ============================================================================

function showBiographyModal(figure) {
    const modalHtml = `
        <div class="modal-overlay biography-modal active" id="biography-modal">
            <div class="modal">
                <div class="modal-header">
                    <h2>Biography</h2>
                    <button class="modal-close" onclick="closeBiographyModal()">&times;</button>
                </div>
                <div class="modal-body">
                    <div class="biography-header">
                        <div class="biography-avatar">
                            ${figure.name?.charAt(0) || '?'}
                        </div>
                        <div class="biography-info">
                            <h3>${figure.name || 'Unknown Figure'}</h3>
                            <div class="biography-meta">
                                ${figure.species || 'Unknown species'} · ${figure.title || 'No title'}
                            </div>
                        </div>
                    </div>
                    
                    <div class="biography-section">
                        <h4>Statistics</h4>
                        <div class="biography-stats">
                            <div class="biography-stat">
                                <div class="biography-stat-label">Years Active</div>
                                <div class="biography-stat-value">${figure.born || '?'} - ${figure.died || 'Present'}</div>
                            </div>
                            <div class="biography-stat">
                                <div class="biography-stat-label">Influence</div>
                                <div class="biography-stat-value">${figure.influence || 0}</div>
                            </div>
                            <div class="biography-stat">
                                <div class="biography-stat-label">Events Participated</div>
                                <div class="biography-stat-value">${figure.events_count || 0}</div>
                            </div>
                            <div class="biography-stat">
                                <div class="biography-stat-label">Faction</div>
                                <div class="biography-stat-value">${figure.faction || 'None'}</div>
                            </div>
                        </div>
                    </div>
                    
                    <div class="biography-section">
                        <h4>Biography</h4>
                        <p>${figure.biography || 'No biography available.'}</p>
                    </div>
                    
                    ${figure.achievements?.length ? `
                        <div class="biography-section">
                            <h4>Achievements</h4>
                            <div class="biography-achievements">
                                ${figure.achievements.map(a => `
                                    <div class="biography-achievement">
                                        <span class="achievement-icon">${a.icon || '🏆'}</span>
                                        <span>${a.description || a}</span>
                                    </div>
                                `).join('')}
                            </div>
                        </div>
                    ` : ''}
                </div>
            </div>
        </div>
    `;
    
    // Add modal to body
    const existingModal = document.getElementById('biography-modal');
    if (existingModal) {
        existingModal.remove();
    }
    document.body.insertAdjacentHTML('beforeend', modalHtml);
    
    // Close on overlay click
    document.getElementById('biography-modal')?.addEventListener('click', (e) => {
        if (e.target.id === 'biography-modal') {
            closeBiographyModal();
        }
    });
    
    // Close on escape key
    const closeOnEscape = (e) => {
        if (e.key === 'Escape') {
            closeBiographyModal();
            document.removeEventListener('keydown', closeOnEscape);
        }
    };
    document.addEventListener('keydown', closeOnEscape);
}

function closeBiographyModal() {
    const modal = document.getElementById('biography-modal');
    if (modal) {
        modal.classList.remove('active');
        setTimeout(() => modal.remove(), 200);
    }
}
