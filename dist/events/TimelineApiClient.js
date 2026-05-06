"use strict";
/**
 * World Factory - Timeline API Client
 *
 * Frontend service for fetching historical events and timeline data
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.timelineApi = exports.TimelineApiClient = void 0;
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api';
/**
 * Timeline API client for fetching historical event data
 */
class TimelineApiClient {
    constructor(baseUrl = API_BASE_URL) {
        this.baseUrl = baseUrl;
    }
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
    async getTimeline(worldId, filter, limit = 100, offset = 0, sort = 'asc') {
        const params = new URLSearchParams();
        params.set('limit', String(limit));
        params.set('offset', String(offset));
        params.set('sort', sort);
        if (filter) {
            if (filter.eventTypes?.length) {
                params.set('eventTypes', filter.eventTypes.join(','));
            }
            if (filter.entityId) {
                params.set('entityId', filter.entityId);
            }
            if (filter.regionId) {
                params.set('regionId', filter.regionId);
            }
            if (filter.yearRange) {
                params.set('startYear', String(filter.yearRange.start));
                params.set('endYear', String(filter.yearRange.end));
            }
            if (filter.minSignificance !== undefined) {
                params.set('minSignificance', String(filter.minSignificance));
            }
            if (filter.tags?.length) {
                params.set('tags', filter.tags.join(','));
            }
        }
        const response = await fetch(`${this.baseUrl}/worlds/${worldId}/timeline?${params.toString()}`, {
            method: 'GET',
            headers: {
                'Accept': 'application/json',
            },
            credentials: 'include',
        });
        if (!response.ok) {
            const errorData = await response.json().catch(() => ({}));
            throw new Error(errorData.error || `Timeline fetch failed: ${response.status}`);
        }
        const result = await response.json();
        if (!result.success) {
            throw new Error(result.error || 'Timeline fetch failed');
        }
        return result.data;
    }
    /**
     * Fetch events with filtering and pagination
     * GET /api/worlds/:id/events
     */
    async getEvents(request) {
        const params = new URLSearchParams();
        params.set('limit', String(request.limit || 50));
        params.set('offset', String(request.offset || 0));
        if (request.sort) {
            params.set('sort', request.sort);
        }
        if (request.filter) {
            const f = request.filter;
            if (f.eventTypes?.length)
                params.set('eventTypes', f.eventTypes.join(','));
            if (f.entityId)
                params.set('entityId', f.entityId);
            if (f.regionId)
                params.set('regionId', f.regionId);
            if (f.yearRange) {
                params.set('startYear', String(f.yearRange.start));
                params.set('endYear', String(f.yearRange.end));
            }
            if (f.minSignificance !== undefined) {
                params.set('minSignificance', String(f.minSignificance));
            }
            if (f.tags?.length)
                params.set('tags', f.tags.join(','));
        }
        const response = await fetch(`${this.baseUrl}/worlds/${request.worldId}/events?${params.toString()}`, {
            method: 'GET',
            headers: {
                'Accept': 'application/json',
            },
            credentials: 'include',
        });
        if (!response.ok) {
            throw new Error(`Events fetch failed: ${response.status}`);
        }
        const result = await response.json();
        if (!result.success) {
            throw new Error(result.error || 'Events fetch failed');
        }
        return result.data;
    }
    /**
     * Fetch a single event by ID
     * GET /api/events/:id
     */
    async getEvent(eventId) {
        const response = await fetch(`${this.baseUrl}/events/${eventId}`, {
            method: 'GET',
            headers: {
                'Accept': 'application/json',
            },
            credentials: 'include',
        });
        if (!response.ok) {
            throw new Error(`Event fetch failed: ${response.status}`);
        }
        const result = await response.json();
        if (!result.success) {
            throw new Error(result.error || 'Event fetch failed');
        }
        return result.data;
    }
}
exports.TimelineApiClient = TimelineApiClient;
// Singleton instance for convenience
exports.timelineApi = new TimelineApiClient();
//# sourceMappingURL=TimelineApiClient.js.map