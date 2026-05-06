"use strict";
/**
 * World Factory - Timeline Component
 *
 * Vertical timeline component for displaying historical events
 * Supports filtering, scrolling, and rich event cards
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.TimelineComponent = TimelineComponent;
const react_1 = require("react");
const TimelineApiClient_1 = require("./TimelineApiClient");
/**
 * Event type display configuration
 */
const EVENT_TYPE_CONFIG = {
    war: { label: 'War & Conflict', color: '#dc2626', icon: '⚔️' },
    discovery: { label: 'Discovery', color: '#2563eb', icon: '🔭' },
    settlement: { label: 'Settlement', color: '#059669', icon: '🏘️' },
    plague: { label: 'Plague & Disease', color: '#7c3aed', icon: '☠️' },
    innovation: { label: 'Innovation', color: '#0891b2', icon: '⚙️' },
    treaty: { label: 'Treaty & Alliance', color: '#4f46e5', icon: '📜' },
    famine: { label: 'Famine', color: '#ea580c', icon: '🌾' },
    revolt: { label: 'Revolt & Revolution', color: '#dc2626', icon: '🔥' },
    coronation: { label: 'Coronation', color: '#ca8a04', icon: '👑' },
    alliance: { label: 'Alliance', color: '#4f46e5', icon: '🤝' },
    betrayal: { label: 'Betrayal', color: '#991b1b', icon: '🗡️' },
    migration: { label: 'Migration', color: '#0891b2', icon: '🚶' },
    construction: { label: 'Construction', color: '#059669', icon: '🏗️' },
    destruction: { label: 'Destruction', color: '#dc2626', icon: '💥' },
    cultural: { label: 'Cultural Event', color: '#db2777', icon: '🎭' },
    natural: { label: 'Natural Event', color: '#65a30d', icon: '🌍' },
};
/**
 * Event card component for individual timeline entries
 */
function EventCard({ event, onClick, isCompact = false, }) {
    const config = EVENT_TYPE_CONFIG[event.eventType] || {
        label: event.eventType,
        color: '#6b7280',
        icon: '📅',
    };
    const significanceBar = Math.round(event.significance * 100);
    return (<article className={`timeline-event ${isCompact ? 'compact' : ''}`} onClick={onClick} role="button" tabIndex={0} onKeyDown={(e) => e.key === 'Enter' && onClick?.()} aria-label={`${event.title}, ${event.position.year}`}>
      <div className="event-indicator" style={{ backgroundColor: config.color }}>
        <span className="event-icon">{config.icon}</span>
      </div>
      
      <div className="event-content">
        <header className="event-header">
          <time className="event-year" dateTime={String(event.position.year)}>
            {event.position.year < 0 ? `${Math.abs(event.position.year)} BCE` : `${event.position.year} CE`}
          </time>
          {event.position.season && (<span className="event-season">{event.position.season}</span>)}
        </header>
        
        <h3 className="event-title">{event.title}</h3>
        
        {!isCompact && (<>
            <p className="event-description">{event.description}</p>
            
            {event.participants.length > 0 && (<div className="event-participants">
                {event.participants.slice(0, 3).map((p, i) => (<span key={p.entityId} className={`participant-badge ${p.role}`}>
                    {p.name}
                    {i < Math.min(event.participants.length, 3) - 1 && ', '}
                  </span>))}
                {event.participants.length > 3 && (<span className="more-participants">
                    +{event.participants.length - 3} more
                  </span>)}
              </div>)}
          </>)}
        
        <div className="event-significance" title={`Significance: ${significanceBar}%`}>
          <div className="significance-bar" style={{ width: `${significanceBar}%`, backgroundColor: config.color }}/>
        </div>
      </div>
    </article>);
}
/**
 * Year marker component for timeline
 */
function YearMarker({ year, isMajor = false }) {
    const displayYear = year < 0 ? `${Math.abs(year)} BCE` : `${year} CE`;
    return (<div className={`year-marker ${isMajor ? 'major' : ''}`}>
      <span className="year-label">{displayYear}</span>
      {isMajor && <div className="year-tick"/>}
    </div>);
}
/**
 * Filter bar component
 */
function FilterBar({ filter, onChange, availableEventTypes = [], }) {
    const [showFilters, setShowFilters] = (0, react_1.useState)(false);
    const allEventTypes = [
        'war', 'discovery', 'settlement', 'plague', 'innovation', 'treaty',
        'famine', 'revolt', 'coronation', 'alliance', 'betrayal', 'migration',
        'construction', 'destruction', 'cultural', 'natural'
    ];
    const typesToShow = availableEventTypes.length > 0 ? availableEventTypes : allEventTypes;
    return (<div className="filter-bar">
      <div className="filter-row">
        <select className="filter-select" value={filter.eventTypes?.[0] || ''} onChange={(e) => onChange({
            ...filter,
            eventTypes: e.target.value ? [e.target.value] : [],
        })} aria-label="Filter by event type">
          <option value="">All Event Types</option>
          {typesToShow.map(type => (<option key={type} value={type}>
              {EVENT_TYPE_CONFIG[type]?.label || type}
            </option>))}
        </select>

        <input type="text" className="filter-input" placeholder="Filter by entity..." value={filter.entityId || ''} onChange={(e) => onChange({ ...filter, entityId: e.target.value || undefined })} aria-label="Filter by entity"/>

        <button className={`filter-toggle ${showFilters ? 'active' : ''}`} onClick={() => setShowFilters(!showFilters)} aria-expanded={showFilters} aria-label="Toggle advanced filters">
          Filters {showFilters ? '▲' : '▼'}
        </button>

        {filter.eventTypes?.length || filter.entityId || filter.yearRange ? (<button className="filter-clear" onClick={() => onChange({})} aria-label="Clear all filters">
            Clear
          </button>) : null}
      </div>

      {showFilters && (<div className="filter-advanced">
          <div className="filter-group">
            <label>Year Range</label>
            <div className="year-range-inputs">
              <input type="number" placeholder="Start year" value={filter.yearRange?.start ?? ''} onChange={(e) => onChange({
                ...filter,
                yearRange: {
                    start: e.target.value ? Number(e.target.value) : -5000,
                    end: filter.yearRange?.end ?? 2000,
                },
            })} aria-label="Start year"/>
              <span>to</span>
              <input type="number" placeholder="End year" value={filter.yearRange?.end ?? ''} onChange={(e) => onChange({
                ...filter,
                yearRange: {
                    start: filter.yearRange?.start ?? -5000,
                    end: e.target.value ? Number(e.target.value) : 2000,
                },
            })} aria-label="End year"/>
            </div>
          </div>

          <div className="filter-group">
            <label>Min Significance</label>
            <input type="range" min="0" max="100" value={Math.round((filter.minSignificance ?? 0) * 100)} onChange={(e) => onChange({
                ...filter,
                minSignificance: Number(e.target.value) / 100,
            })} aria-label="Minimum significance"/>
            <span className="significance-value">
              {Math.round((filter.minSignificance ?? 0) * 100)}%
            </span>
          </div>
        </div>)}
    </div>);
}
/**
 * Main Timeline Component
 */
function TimelineComponent({ worldId, initialFilter = {}, onEventSelect, className, height = '600px', }) {
    const containerRef = (0, react_1.useRef)(null);
    const [timeline, setTimeline] = (0, react_1.useState)(null);
    const [loading, setLoading] = (0, react_1.useState)(true);
    const [error, setError] = (0, react_1.useState)(null);
    const [filter, setFilter] = (0, react_1.useState)(initialFilter);
    const [visibleRange, setVisibleRange] = (0, react_1.useState)({ start: 0, end: 50 });
    const [selectedEventId, setSelectedEventId] = (0, react_1.useState)(null);
    // Group events by year for efficient rendering
    const eventsByYear = (0, react_1.useMemo)(() => {
        if (!timeline)
            return new Map();
        const groups = new Map();
        for (const event of timeline.events) {
            const year = event.position.year;
            if (!groups.has(year)) {
                groups.set(year, []);
            }
            groups.get(year).push(event);
        }
        return groups;
    }, [timeline]);
    // Get sorted years
    const sortedYears = (0, react_1.useMemo)(() => {
        return Array.from(eventsByYear.keys()).sort((a, b) => a - b);
    }, [eventsByYear]);
    // Determine if a year is a "major" marker (century marks)
    const isMajorYear = (0, react_1.useCallback)((year) => {
        return year % 100 === 0;
    }, []);
    // Load timeline data
    (0, react_1.useEffect)(() => {
        let cancelled = false;
        async function loadTimeline() {
            setLoading(true);
            setError(null);
            try {
                const data = await TimelineApiClient_1.timelineApi.getTimeline(worldId, filter, 200, 0, 'asc');
                if (!cancelled) {
                    setTimeline(data);
                }
            }
            catch (err) {
                if (!cancelled) {
                    setError(err instanceof Error ? err : new Error('Failed to load timeline'));
                }
            }
            finally {
                if (!cancelled) {
                    setLoading(false);
                }
            }
        }
        loadTimeline();
        return () => {
            cancelled = true;
        };
    }, [worldId, filter]);
    // Handle scroll for virtual scrolling
    const handleScroll = (0, react_1.useCallback)(() => {
        if (!containerRef.current)
            return;
        const { scrollTop, scrollHeight, clientHeight } = containerRef.current;
        const buffer = 5; // Render extra items above/below viewport
        // Calculate which years are visible
        // This is a simplified approach - production would use a virtual list
        const scrollProgress = scrollTop / (scrollHeight - clientHeight);
        const totalYears = sortedYears.length;
        const visibleCount = Math.ceil((clientHeight / 80) + buffer * 2);
        const startIndex = Math.max(0, Math.floor(scrollProgress * totalYears) - buffer);
        const endIndex = Math.min(totalYears, startIndex + visibleCount);
        setVisibleRange({ start: startIndex, end: endIndex });
    }, [sortedYears.length]);
    // Handle event selection
    const handleEventClick = (0, react_1.useCallback)((event) => {
        setSelectedEventId(event.id);
        onEventSelect?.(event);
    }, [onEventSelect]);
    // Loading state with skeleton
    if (loading) {
        return (<div className={`timeline-loading ${className || ''}`} style={{ height }} aria-label="Loading timeline">
        <div className="timeline-skeleton">
          {Array.from({ length: 8 }).map((_, i) => (<div key={i} className="skeleton-event">
              <div className="skeleton-marker"/>
              <div className="skeleton-content">
                <div className="skeleton-year"/>
                <div className="skeleton-title"/>
                <div className="skeleton-description"/>
              </div>
            </div>))}
        </div>
        <style>{`
          .timeline-loading {
            background: var(--bg-primary, #0f0f1a);
            border-radius: 8px;
            overflow: hidden;
            padding: 16px;
          }
          .timeline-skeleton {
            display: flex;
            flex-direction: column;
            gap: 24px;
          }
          .skeleton-event {
            display: flex;
            gap: 16px;
            align-items: flex-start;
          }
          .skeleton-marker {
            width: 32px;
            height: 32px;
            border-radius: 50%;
            background: linear-gradient(90deg, #1a1a2e 25%, #2a2a4e 50%, #1a1a2e 75%);
            background-size: 200% 100%;
            animation: shimmer 1.5s infinite;
            flex-shrink: 0;
          }
          .skeleton-content {
            flex: 1;
            display: flex;
            flex-direction: column;
            gap: 8px;
          }
          .skeleton-year {
            width: 80px;
            height: 14px;
            background: linear-gradient(90deg, #1a1a2e 25%, #2a2a4e 50%, #1a1a2e 75%);
            background-size: 200% 100%;
            animation: shimmer 1.5s infinite;
            border-radius: 4px;
          }
          .skeleton-title {
            width: 60%;
            height: 18px;
            background: linear-gradient(90deg, #1a1a2e 25%, #2a2a4e 50%, #1a1a2e 75%);
            background-size: 200% 100%;
            animation: shimmer 1.5s infinite;
            border-radius: 4px;
          }
          .skeleton-description {
            width: 90%;
            height: 12px;
            background: linear-gradient(90deg, #1a1a2e 25%, #2a2a4e 50%, #1a1a2e 75%);
            background-size: 200% 100%;
            animation: shimmer 1.5s infinite;
            border-radius: 4px;
          }
          @keyframes shimmer {
            0% { background-position: 200% 0; }
            100% { background-position: -200% 0; }
          }
        `}</style>
      </div>);
    }
    // Error state
    if (error) {
        return (<div className={`timeline-error ${className || ''}`} style={{ height }} role="alert">
        <div className="error-content">
          <h3>Failed to load timeline</h3>
          <p>{error.message}</p>
          <button onClick={() => window.location.reload()}>
            Retry
          </button>
        </div>
        <style>{`
          .timeline-error {
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100%;
            background: var(--bg-primary, #0f0f1a);
            border-radius: 8px;
            color: #ff6b6b;
          }
          .error-content {
            text-align: center;
          }
          .error-content h3 {
            margin: 0 0 8px 0;
            color: #ff6b6b;
          }
          .error-content p {
            margin: 0 0 16px 0;
            color: #888;
          }
          .error-content button {
            padding: 8px 16px;
            background: #3a3a6e;
            color: #fff;
            border: none;
            border-radius: 4px;
            cursor: pointer;
          }
        `}</style>
      </div>);
    }
    // Empty state
    if (!timeline || timeline.events.length === 0) {
        return (<div className={`timeline-empty ${className || ''}`} style={{ height }} role="status">
        <div className="empty-content">
          <span className="empty-icon">📜</span>
          <h3>No events found</h3>
          <p>Try adjusting your filters or generate more history.</p>
        </div>
        <style>{`
          .timeline-empty {
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100%;
            background: var(--bg-primary, #0f0f1a);
            border-radius: 8px;
            color: #888;
          }
          .empty-content {
            text-align: center;
          }
          .empty-icon {
            font-size: 48px;
            display: block;
            margin-bottom: 16px;
          }
          .empty-content h3 {
            margin: 0 0 8px 0;
            color: #ccc;
          }
          .empty-content p {
            margin: 0;
            color: #666;
          }
        `}</style>
      </div>);
    }
    // Visible years for current viewport
    const visibleYears = sortedYears.slice(visibleRange.start, visibleRange.end);
    return (<div className={`timeline-container ${className || ''}`} style={{ height }} ref={containerRef} onScroll={handleScroll} role="region" aria-label="Historical timeline" tabIndex={0}>
      <FilterBar filter={filter} onChange={setFilter} availableEventTypes={Array.from(new Set(timeline.events.map(e => e.eventType)))}/>

      <div className="timeline-scroll-info">
        Showing {visibleRange.start + 1}-{Math.min(visibleRange.end, sortedYears.length)} of {sortedYears.length} years
        • {timeline.totalEvents} events
      </div>

      <div className="timeline-content">
        <div className="timeline-line"/>
        
        {visibleYears.map(year => {
            const events = eventsByYear.get(year) || [];
            return (<div key={year} className="year-group">
              <YearMarker year={year} isMajor={isMajorYear(year)}/>
              
              <div className="year-events">
                {events.map(event => (<EventCard key={event.id} event={event} onClick={() => handleEventClick(event)} isCompact={selectedEventId !== event.id}/>))}
              </div>
            </div>);
        })}
      </div>

      <style>{`
        .timeline-container {
          display: flex;
          flex-direction: column;
          background: var(--bg-primary, #0f0f1a);
          border-radius: 8px;
          overflow: hidden;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }
        
        .filter-bar {
          padding: 12px 16px;
          background: var(--bg-secondary, #1a1a2e);
          border-bottom: 1px solid rgba(255,255,255,0.1);
          position: sticky;
          top: 0;
          z-index: 10;
        }
        
        .filter-row {
          display: flex;
          gap: 8px;
          align-items: center;
          flex-wrap: wrap;
        }
        
        .filter-select,
        .filter-input {
          padding: 8px 12px;
          background: rgba(0,0,0,0.3);
          border: 1px solid rgba(255,255,255,0.1);
          border-radius: 6px;
          color: #e0e0e0;
          font-size: 13px;
          min-width: 150px;
        }
        
        .filter-select:focus,
        .filter-input:focus {
          outline: none;
          border-color: #6366f1;
        }
        
        .filter-toggle,
        .filter-clear {
          padding: 8px 12px;
          background: rgba(255,255,255,0.05);
          border: 1px solid rgba(255,255,255,0.1);
          border-radius: 6px;
          color: #888;
          font-size: 12px;
          cursor: pointer;
          transition: all 0.2s;
        }
        
        .filter-toggle:hover,
        .filter-clear:hover {
          background: rgba(255,255,255,0.1);
          color: #ccc;
        }
        
        .filter-toggle.active {
          background: #6366f1;
          border-color: #6366f1;
          color: white;
        }
        
        .filter-advanced {
          margin-top: 12px;
          padding-top: 12px;
          border-top: 1px solid rgba(255,255,255,0.05);
          display: flex;
          gap: 24px;
        }
        
        .filter-group {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }
        
        .filter-group label {
          font-size: 11px;
          color: #666;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }
        
        .year-range-inputs {
          display: flex;
          align-items: center;
          gap: 8px;
        }
        
        .year-range-inputs input {
          width: 100px;
          padding: 6px 8px;
          background: rgba(0,0,0,0.3);
          border: 1px solid rgba(255,255,255,0.1);
          border-radius: 4px;
          color: #e0e0e0;
          font-size: 13px;
        }
        
        .year-range-inputs span {
          color: #666;
        }
        
        input[type="range"] {
          width: 120px;
        }
        
        .significance-value {
          font-size: 12px;
          color: #888;
          margin-left: 8px;
        }
        
        .timeline-scroll-info {
          padding: 8px 16px;
          font-size: 12px;
          color: #666;
          background: rgba(0,0,0,0.2);
        }
        
        .timeline-content {
          flex: 1;
          overflow-y: auto;
          padding: 16px;
          position: relative;
        }
        
        .timeline-line {
          position: absolute;
          left: 31px;
          top: 0;
          bottom: 0;
          width: 2px;
          background: linear-gradient(to bottom, 
            transparent 0%,
            rgba(99, 102, 241, 0.3) 5%,
            rgba(99, 102, 241, 0.3) 95%,
            transparent 100%
          );
        }
        
        .year-group {
          margin-bottom: 24px;
          position: relative;
        }
        
        .year-marker {
          display: flex;
          align-items: center;
          gap: 12px;
          margin-bottom: 12px;
          position: relative;
          z-index: 1;
        }
        
        .year-marker::before {
          content: '';
          position: absolute;
          left: 16px;
          width: 30px;
          height: 2px;
          background: var(--bg-secondary, #1a1a2e);
        }
        
        .year-label {
          background: var(--bg-secondary, #1a1a2e);
          padding: 4px 8px;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 600;
          color: #888;
          white-space: nowrap;
          border: 1px solid rgba(255,255,255,0.05);
        }
        
        .year-marker.major .year-label {
          background: #6366f1;
          color: white;
          font-size: 13px;
        }
        
        .year-events {
          display: flex;
          flex-direction: column;
          gap: 12px;
          padding-left: 48px;
        }
        
        .timeline-event {
          display: flex;
          gap: 12px;
          background: var(--bg-secondary, #1a1a2e);
          border-radius: 8px;
          padding: 12px;
          cursor: pointer;
          transition: all 0.2s;
          border: 1px solid transparent;
        }
        
        .timeline-event:hover {
          border-color: rgba(255,255,255,0.1);
          transform: translateX(4px);
        }
        
        .timeline-event:focus {
          outline: none;
          border-color: #6366f1;
        }
        
        .timeline-event.selected {
          border-color: #6366f1;
          box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.2);
        }
        
        .timeline-event.compact {
          padding: 8px 12px;
        }
        
        .timeline-event.compact .event-description,
        .timeline-event.compact .event-participants {
          display: none;
        }
        
        .event-indicator {
          width: 32px;
          height: 32px;
          border-radius: 50%;
          display: flex;
          align-items: center;
          justify-content: center;
          flex-shrink: 0;
          box-shadow: 0 2px 8px rgba(0,0,0,0.3);
        }
        
        .event-icon {
          font-size: 14px;
        }
        
        .event-content {
          flex: 1;
          min-width: 0;
        }
        
        .event-header {
          display: flex;
          align-items: center;
          gap: 8px;
          margin-bottom: 4px;
        }
        
        .event-year {
          font-size: 12px;
          font-weight: 600;
          color: #6366f1;
        }
        
        .event-season {
          font-size: 11px;
          color: #666;
          text-transform: capitalize;
        }
        
        .event-title {
          font-size: 15px;
          font-weight: 600;
          color: #e0e0e0;
          margin: 0 0 6px 0;
          line-height: 1.3;
        }
        
        .event-description {
          font-size: 13px;
          color: #888;
          line-height: 1.5;
          margin: 0 0 8px 0;
          display: -webkit-box;
          -webkit-line-clamp: 2;
          -webkit-box-orient: vertical;
          overflow: hidden;
        }
        
        .event-participants {
          display: flex;
          flex-wrap: wrap;
          gap: 4px;
          margin-bottom: 8px;
        }
        
        .participant-badge {
          font-size: 11px;
          padding: 2px 6px;
          border-radius: 4px;
          background: rgba(255,255,255,0.05);
          color: #aaa;
        }
        
        .participant-badge.initiator {
          background: rgba(99, 102, 241, 0.2);
          color: #a5b4fc;
        }
        
        .participant-badge.target {
          background: rgba(220, 38, 38, 0.2);
          color: #fca5a5;
        }
        
        .more-participants {
          font-size: 11px;
          color: #666;
        }
        
        .event-significance {
          height: 3px;
          background: rgba(255,255,255,0.1);
          border-radius: 2px;
          overflow: hidden;
        }
        
        .significance-bar {
          height: 100%;
          border-radius: 2px;
          transition: width 0.3s ease;
        }
        
        /* Responsive */
        @media (max-width: 640px) {
          .filter-row {
            flex-direction: column;
            align-items: stretch;
          }
          
          .filter-select,
          .filter-input {
            width: 100%;
          }
          
          .filter-advanced {
            flex-direction: column;
          }
        }
      `}</style>
    </div>);
}
exports.default = TimelineComponent;
//# sourceMappingURL=TimelineComponent.js.map