/**
 * World Factory - Timeline Types
 *
 * Type definitions for timeline and historical event data
 * These interfaces define the event data contract with the backend
 */
/**
 * Represents a point in time on the world timeline
 */
export interface TimelinePosition {
    /** Year on the timeline (can be negative for BCE) */
    year: number;
    /** Optional season within the year */
    season?: 'spring' | 'summer' | 'autumn' | 'winter';
    /** Optional century reference string (e.g., "12th century") */
    century?: string;
}
/**
 * Entity participation in an event
 */
export interface EventParticipant {
    /** Entity ID reference */
    entityId: string;
    /** Entity name for display */
    name: string;
    /** Entity type */
    entityType: 'nation' | 'city' | 'person' | 'organization' | 'region';
    /** Role in the event */
    role: 'initiator' | 'participant' | 'target' | 'witness';
}
/**
 * A historical event on the timeline
 */
export interface HistoricalEvent {
    /** Unique identifier */
    id: string;
    /** Event type classification */
    eventType: EventType;
    /** When the event occurred */
    position: TimelinePosition;
    /** Human-readable title */
    title: string;
    /** Detailed description */
    description: string;
    /** Entities involved in this event */
    participants: EventParticipant[];
    /** IDs of events that preceded and caused this one */
    prerequisites: string[];
    /** Outcomes and effects of this event */
    outcomes: EventOutcome[];
    /** Significance weight (0.0 to 1.0) */
    significance: number;
    /** Optional related entity IDs for filtering */
    relatedEntities?: string[];
    /** Optional tags for filtering */
    tags?: string[];
}
/**
 * Event type classifications
 */
export type EventType = 'war' | 'discovery' | 'settlement' | 'plague' | 'innovation' | 'treaty' | 'famine' | 'revolt' | 'coronation' | 'alliance' | 'betrayal' | 'migration' | 'construction' | 'destruction' | 'cultural' | 'natural';
/**
 * An outcome or effect resulting from an event
 */
export interface EventOutcome {
    /** Outcome type */
    type: 'territory_change' | 'population_change' | 'relationship_change' | 'technology_gain' | 'cultural_change' | 'resource_change';
    /** Description for display */
    description: string;
    /** Affected entity IDs */
    affectedEntities: string[];
    /** Magnitude of the outcome */
    magnitude?: number;
}
/**
 * Timeline data with events
 */
export interface Timeline {
    /** World ID this timeline belongs to */
    worldId: string;
    /** Start year of the timeline */
    startYear: number;
    /** End year of the timeline */
    endYear: number;
    /** All events in chronological order */
    events: HistoricalEvent[];
    /** Total count for pagination */
    totalEvents: number;
}
/**
 * Filter options for timeline queries
 */
export interface TimelineFilter {
    /** Filter by event type */
    eventTypes?: EventType[];
    /** Filter by related entity ID */
    entityId?: string;
    /** Filter by region ID */
    regionId?: string;
    /** Filter by year range */
    yearRange?: {
        start: number;
        end: number;
    };
    /** Filter by minimum significance */
    minSignificance?: number;
    /** Filter by tags */
    tags?: string[];
}
/**
 * API response wrapper for timeline data
 */
export interface TimelineResponse {
    success: boolean;
    data: Timeline;
    error?: string;
}
/**
 * API response wrapper for events list
 */
export interface EventsResponse {
    success: boolean;
    data: {
        events: HistoricalEvent[];
        total: number;
        limit: number;
        offset: number;
    };
    error?: string;
}
/**
 * Request parameters for fetching events
 */
export interface EventsRequest {
    worldId: string;
    /** Filter options */
    filter?: TimelineFilter;
    /** Pagination limit */
    limit?: number;
    /** Pagination offset */
    offset?: number;
    /** Sort order */
    sort?: 'asc' | 'desc';
}
//# sourceMappingURL=TimelineTypes.d.ts.map