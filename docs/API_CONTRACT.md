/**
 * World Factory - API Contract Documentation
 * 
 * Expected API endpoints and response shapes for frontend-backend integration
 * This document serves as the contract for backend implementation
 * 
 * @see WOR-37: Map Data Endpoint
 * @see WOR-41: History Timeline UI
 */

/**
 * Base URL: /api
 * All endpoints return JSON with application/json content type
 * All endpoints require authentication (session cookie or Bearer token)
 */

/**
 * =============================================================================
 * ENDPOINT: GET /api/worlds/:id/map
 * =============================================================================
 * 
 * Purpose: Retrieve render-ready map data for a specific world
 * 
 * URL Parameters:
 *   - id (string, required): World UUID
 * 
 * Query Parameters:
 *   - minX (number, optional): Viewport bound - minimum X coordinate
 *   - minY (number, optional): Viewport bound - minimum Y coordinate
 *   - maxX (number, optional): Viewport bound - maximum X coordinate
 *   - maxY (number, optional): Viewport bound - maximum Y coordinate
 *   - lod (number, optional): Level of detail (0=low, 1=medium, 2=high)
 *     Default: 1
 * 
 * Response (200 OK):
 * {
 *   "success": true,
 *   "data": {
 *     "worldId": "uuid-string",
 *     "dimensions": { "width": 1000, "height": 1000 },
 *     "scale": 1.0,
 *     "polygons": [
 *       {
 *         "id": "poly-uuid",
 *         "type": "territory" | "biome" | "region" | "resource",
 *         "vertices": [{ "x": 0, "y": 0 }, ...],
 *         "holes": [[{ "x": 50, "y": 50 }, ...], ...] // optional
 *       }
 *     ],
 *     "biomes": [
 *       {
 *         "id": "biome-uuid",
 *         "type": "forest",
 *         "color": [34, 139, 34], // [R, G, B] integers 0-255
 *         "name": "Dense Forest"
 *       }
 *     ],
 *     "resources": [
 *       {
 *         "id": "res-uuid",
 *         "type": "iron",
 *         "position": { "x": 150, "y": 200 },
 *         "magnitude": 3, // 1-5 scale
 *         "name": "Iron Deposit"
 *       }
 *     ],
 *     "entities": [
 *       {
 *         "id": "city-uuid",
 *         "type": "city" | "settlement" | "landmark" | "fortress",
 *         "position": { "x": 500, "y": 500 },
 *         "name": "New Avalon",
 *         "significance": 5
 *       }
 *     ],
 *     "elevationGrid": [[0, 1, 2], [1, 2, 3], ...], // optional, height values
 *     "metadata": {
 *       "generatedAt": "2026-04-30T12:00:00Z",
 *       "version": "1.0.0"
 *     }
 *   }
 * }
 * 
 * Error Responses:
 *   - 400 Bad Request: Invalid parameters
 *   - 401 Unauthorized: Missing authentication
 *   - 404 Not Found: World does not exist
 *   - 500 Internal Server Error: Server error
 * 
 * Performance Requirements:
 *   - Response time < 500ms for full map
 *   - Response time < 200ms for viewport-bounded request
 *   - Map data should be cacheable (ETag support recommended)
 */

/**
 * =============================================================================
 * ENDPOINT: GET /api/worlds/:id
 * =============================================================================
 * 
 * Purpose: Retrieve world metadata (name, generation status, etc.)
 * 
 * URL Parameters:
 *   - id (string, required): World UUID
 * 
 * Response (200 OK):
 * {
 *   "success": true,
 *   "data": {
 *     "id": "uuid-string",
 *     "name": "Thornvald",
 *     "status": "generating" | "ready" | "failed",
 *     "progress": 0.75, // 0-1 for generating status
 *     "createdAt": "2026-04-30T10:00:00Z",
 *     "parameters": {
 *       "seed": 12345,
 *       "size": "large"
 *     }
 *   }
 * }
 */

/**
 * =============================================================================
 * ENDPOINT: GET /api/worlds
 * =============================================================================
 * 
 * Purpose: List all worlds for current user
 * 
 * Query Parameters:
 *   - limit (number, optional): Max results (default: 20)
 *   - offset (number, optional): Pagination offset
 * 
 * Response (200 OK):
 * {
 *   "success": true,
 *   "data": {
 *     "worlds": [...],
 *     "total": 42,
 *     "limit": 20,
 *     "offset": 0
 *   }
 * }
 */

/**
 * =============================================================================
 * ENDPOINT: GET /api/worlds/:id/timeline
 * =============================================================================
 * 
 * Purpose: Retrieve timeline with historical events for a specific world
 * 
 * URL Parameters:
 *   - id (string, required): World UUID
 * 
 * Query Parameters:
 *   - limit (number, optional): Max events to return (default: 100)
 *   - offset (number, optional): Pagination offset (default: 0)
 *   - sort (string, optional): 'asc' for oldest first, 'desc' for newest first
 *   - eventTypes (string, optional): Comma-separated list of event types to filter
 *   - entityId (string, optional): Filter events involving this entity
 *   - regionId (string, optional): Filter events in this region
 *   - startYear (number, optional): Filter events from this year onwards
 *   - endYear (number, optional): Filter events up to this year
 *   - minSignificance (number, optional): Minimum significance 0-1
 *   - tags (string, optional): Comma-separated tags to filter
 * 
 * Response (200 OK):
 * {
 *   "success": true,
 *   "data": {
 *     "worldId": "uuid-string",
 *     "startYear": -3000,
 *     "endYear": 1500,
 *     "events": [
 *       {
 *         "id": "event-uuid",
 *         "eventType": "war",
 *         "position": {
 *           "year": 1347,
 *           "season": "autumn",
 *           "century": "14th century"
 *         },
 *         "title": "The Great War of Valdoria",
 *         "description": "A conflict between...",
 *         "participants": [
 *           {
 *             "entityId": "nation-uuid",
 *             "name": "Kingdom of Valdoria",
 *             "entityType": "nation",
 *             "role": "initiator"
 *           }
 *         ],
 *         "prerequisites": ["event-uuid-1", "event-uuid-2"],
 *         "outcomes": [
 *           {
 *             "type": "territory_change",
 *             "description": "Kingdom lost northern provinces",
 *             "affectedEntities": ["region-uuid"],
 *             "magnitude": 0.8
 *           }
 *         ],
 *         "significance": 0.85,
 *         "relatedEntities": ["nation-uuid", "city-uuid"],
 *         "tags": ["military", "political", "north"]
 *       }
 *     ],
 *     "totalEvents": 342
 *   }
 * }
 * 
 * Event Type Values:
 *   - war, discovery, settlement, plague, innovation, treaty
 *   - famine, revolt, coronation, alliance, betrayal, migration
 *   - construction, destruction, cultural, natural
 * 
 * Error Responses:
 *   - 400 Bad Request: Invalid parameters
 *   - 401 Unauthorized: Missing authentication
 *   - 404 Not Found: World does not exist
 *   - 500 Internal Server Error: Server error
 * 
 * Performance Requirements:
 *   - Response time < 300ms for filtered queries
 *   - Pagination recommended for > 500 events
 */

/**
 * =============================================================================
 * ENDPOINT: GET /api/worlds/:id/events
 * =============================================================================
 * 
 * Purpose: Retrieve paginated historical events with filtering
 * Alias for timeline with different default pagination
 * 
 * Query Parameters: Same as /api/worlds/:id/timeline
 * 
 * Response (200 OK):
 * {
 *   "success": true,
 *   "data": {
 *     "events": [...],
 *     "total": 342,
 *     "limit": 50,
 *     "offset": 0
 *   }
 * }
 */

/**
 * =============================================================================
 * ENDPOINT: GET /api/worlds/:id/events
 * =============================================================================
 * 
 * Purpose: Retrieve paginated historical events with filtering
 * Alias for timeline with different default pagination
 * 
 * Query Parameters: Same as /api/worlds/:id/timeline
 * 
 * Response (200 OK):
 * {
 *   "success": true,
 *   "data": {
 *     "events": [...],
 *     "total": 342,
 *     "limit": 50,
 *     "offset": 0
 *   }
 * }
 */

/**
 * =============================================================================
 * ENDPOINT: GET /api/worlds/:id/figures
 * =============================================================================
 * 
 * Purpose: Retrieve historical figures/persons for a specific world
 * 
 * URL Parameters:
 *   - id (string, required): World UUID
 * 
 * Query Parameters:
 *   - limit (number, optional): Max results (default: 50, max: 200)
 *   - offset (number, optional): Pagination offset
 *   - speciesId (string, optional): Filter by species ID
 *   - regionId (string, optional): Filter by home region
 *   - minSignificance (number, optional): Minimum significance 0-1
 * 
 * Response (200 OK):
 * {
 *   "success": true,
 *   "data": {
 *     "worldId": "uuid-string",
 *     "figures": [
 *       {
 *         "id": "figure-uuid",
 *         "name": {
 *           "title": "King",
 *           "given": "Aragorn",
 *           "family": "II"
 *         },
 *         "entityType": "person",
 *         "birthYear": 2931,
 *         "deathYear": 3081,
 *         "birthplaceId": "region-uuid",
 *         "culture": "Dúnedain",
 *         "titles": ["King", "Ranger"],
 *         "description": "The last king of Arnor...",
 *         "significance": 0.95,
 *         "speciesId": "species-uuid"
 *       }
 *     ],
 *     "total": 156,
 *     "limit": 50,
 *     "offset": 0
 *   }
 * }
 * 
 * Error Responses:
 *   - 400 Bad Request: Invalid world ID format
 *   - 401 Unauthorized: Missing authentication
 *   - 404 Not Found: World does not exist
 *   - 500 Internal Server Error: Server error
 * 
 * Performance Requirements:
 *   - Response time < 200ms for typical queries
 *   - Significance filter should use database index
 */

/**
 * =============================================================================
 * ENDPOINT: GET /api/events/:id
 * =============================================================================
 * 
 * Purpose: Retrieve a single event by ID
 * 
 * URL Parameters:
 *   - id (string, required): Event UUID
 * 
 * Response (200 OK):
 * {
 *   "success": true,
 *   "data": { /* HistoricalEvent object */ }
 * }
 */

/**
 * =============================================================================
 * CACHING STRATEGY
 * =============================================================================
 * 
 * - Map data should include ETag header for client-side caching
 * - Client should send If-None-Match header with cached ETag
 * - 304 Not Modified response when cache is valid
 * - Version field in metadata for cache invalidation
 * 
 * Client Caching Logic:
 * 1. First request: fetch full map, store data + ETag
 * 2. Subsequent requests: send If-None-Match with ETag
 * 3. If 304: use cached data
 * 4. If 200: update cache with new data + new ETag
 */

/**
 * =============================================================================
 * ERROR HANDLING
 * =============================================================================
 * 
 * All error responses follow this shape:
 * {
 *   "success": false,
 *   "error": "Human-readable error message",
 *   "code": "ERROR_CODE" // optional machine-readable code
 * }
 * 
 * Frontend Error States:
 * - Show user-friendly message from error field
 * - Log detailed error to console for debugging
 * - Provide retry option for transient errors
 * - Show "world not found" UI for 404 responses
 */

/**
 * =============================================================================
 * REAL-TIME UPDATES (Future)
 * =============================================================================
 * 
 * For generating worlds, consider WebSocket updates:
 * 
 * WebSocket: ws://host/api/worlds/:id/live
 * 
 * Messages:
 * {
 *   "type": "progress",
 *   "progress": 0.5,
 *   "phase": "terrain_generation"
 * }
 * 
 * {
 *   "type": "map_update",
 *   "data": { /* partial map data */ }
 * }
 * 
 * {
 *   "type": "complete",
 *   "mapData": { /* full map data */ }
 * }
 */
