/**
 * World Factory - API Client Aliases
 * Module-style re-exports from api-integration.js
 * 
 * api-integration.js must be loaded before this file.
 * This module provides named exports for ES modules style imports.
 */

// Re-export API client methods
const createWorld = (config) => api.createWorld(config);
const getWorld = (worldId) => api.getWorld(worldId);
const fetchWorlds = () => api.fetchWorlds();
const deleteWorld = (worldId) => api.deleteWorld(worldId);
const checkHealth = () => api.checkHealth();
const getWorldMap = (worldId) => api.getWorldMap(worldId);
const getSimulationHistory = (worldId) => api.getSimulationHistory(worldId);
const getHistoryEvents = (worldId, page) => api.getHistoryEvents(worldId, page);
const simulate = (worldId) => api.simulate(worldId);
const getSocieties = (worldId) => api.getSocieties(worldId);
const getFigures = (worldId) => api.getFigures(worldId);
const exportWorld = (worldId) => api.exportWorld(worldId);

// Re-export utility functions
const normalizeWorldId = (worldId) => {
    if (typeof normalizeWorldId === 'function') {
        return normalizeWorldId(worldId);
    }
    return worldId;
};

const formatDate = (dateString) => {
    if (typeof formatDate === 'function') {
        return formatDate(dateString);
    }
    return dateString ? new Date(dateString).toLocaleDateString() : '—';
};

const formatRelativeTime = (dateString) => {
    if (typeof formatRelativeTime === 'function') {
        return formatRelativeTime(dateString);
    }
    if (!dateString) return '—';
    const date = new Date(dateString);
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
};

const formatSeed = (seed) => {
    if (typeof formatSeed === 'function') {
        return formatSeed(seed);
    }
    return seed !== undefined ? String(seed) : '—';
};

const getPhaseInfo = (phase) => {
    if (typeof getPhaseInfo === 'function') {
        return getPhaseInfo(phase);
    }
    const phases = {
        'idle': { name: 'Idle', color: '#6b7280' },
        'generating': { name: 'Generating', color: '#3b82f6' },
        'ready': { name: 'Ready', color: '#22c55e' },
        'simulating': { name: 'Simulating', color: '#8b5cf6' },
        'error': { name: 'Error', color: '#ef4444' }
    };
    return phases[phase] || phases['idle'];
};
