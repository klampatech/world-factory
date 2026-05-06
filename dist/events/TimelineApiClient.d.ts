/**
 * World Factory - Timeline API Client
 *
 * Frontend service for fetching historical events and timeline data
 */
import type { Timeline, EventsRequest, EventsResponse, TimelineFilter } from './TimelineTypes';
/**
 * Timeline API client for fetching historical event data
 */
export declare class TimelineApiClient {
    private baseUrl;
    constructor(baseUrl?: string);
    /**
     * Fetch timeline with events for a world
     * GET /api/worlds/:id/timeline
     *
     * @param worldId - World UUID
     * @param filter - Optional filter criteria
     * @param limit - Max events to return (default: 100)
     * @param offset - Pagination offset
     * @param sort - Sort order (asc = oldest first, desc = newest first)
     */
    getTimeline(worldId: string, filter?: TimelineFilter, limit?: number, offset?: number, sort?: 'asc' | 'desc'): Promise<Timeline>;
    /**
     * Fetch events with filtering and pagination
     * GET /api/worlds/:id/events
     */
    getEvents(request: EventsRequest): Promise<EventsResponse['data']>;
    /**
     * Fetch a single event by ID
     * GET /api/events/:id
     */
    getEvent(eventId: string): Promise<import('./TimelineTypes').HistoricalEvent>;
}
export declare const timelineApi: TimelineApiClient;
//# sourceMappingURL=TimelineApiClient.d.ts.map