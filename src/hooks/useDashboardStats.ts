/**
 * useDashboardStats Hook
 * 
 * React hook for fetching and managing world dashboard statistics.
 * Handles loading states, error handling, and automatic refetching.
 * 
 * @see WOR-1324: Phase 4: Dashboard stats — population, societies, resources
 */

import { useState, useEffect, useCallback } from 'react';
import { fetchWorldStats, transformToWorldStateMetrics, generateMockStats, getDefaultWorldMetrics } from '../services/dashboardService';
import type { WorldStateMetrics } from '../components/Dashboard';

export interface UseDashboardStatsOptions {
  /** World ID to fetch stats for */
  worldId?: string;
  /** Enable mock data fallback (default: true) */
  useMockFallback?: boolean;
  /** Auto-refresh interval in ms (0 = disabled) */
  refreshInterval?: number;
  /** Callback when stats are successfully fetched */
  onStatsLoaded?: (stats: WorldStateMetrics) => void;
  /** Callback on fetch error */
  onError?: (error: Error) => void;
}

export interface UseDashboardStatsResult {
  /** Current world state metrics */
  metrics: WorldStateMetrics;
  /** Loading state */
  loading: boolean;
  /** Error if fetch failed */
  error: Error | null;
  /** Manual refresh function */
  refresh: () => Promise<void>;
  /** Whether using mock data */
  isMockData: boolean;
}

/**
 * Hook for fetching and managing world dashboard statistics
 */
export function useDashboardStats(options: UseDashboardStatsOptions = {}): UseDashboardStatsResult {
  const {
    worldId,
    useMockFallback = true,
    refreshInterval = 0,
    onStatsLoaded,
    onError,
  } = options;

  const [metrics, setMetrics] = useState<WorldStateMetrics>(getDefaultWorldMetrics);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<Error | null>(null);
  const [isMockData, setIsMockData] = useState<boolean>(false);

  const fetchStats = useCallback(async () => {
    if (!worldId) {
      // No world ID provided - use mock data
      if (useMockFallback) {
        const mockStats = generateMockStats();
        setMetrics(transformToWorldStateMetrics(mockStats));
        setIsMockData(true);
        setLoading(false);
      }
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const stats = await fetchWorldStats(worldId);
      setMetrics(transformToWorldStateMetrics(stats));
      setIsMockData(false);
      onStatsLoaded?.(transformToWorldStateMetrics(stats));
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to fetch stats');
      setError(error);
      onError?.(error);

      // Fall back to mock data if enabled
      if (useMockFallback) {
        const mockStats = generateMockStats();
        setMetrics(transformToWorldStateMetrics(mockStats));
        setIsMockData(true);
      }
    } finally {
      setLoading(false);
    }
  }, [worldId, useMockFallback, onStatsLoaded, onError]);

  // Initial fetch
  useEffect(() => {
    fetchStats();
  }, [fetchStats]);

  // Auto-refresh if interval is set
  useEffect(() => {
    if (refreshInterval <= 0 || !worldId) return;

    const intervalId = setInterval(fetchStats, refreshInterval);
    return () => clearInterval(intervalId);
  }, [fetchStats, refreshInterval, worldId]);

  return {
    metrics,
    loading,
    error,
    refresh: fetchStats,
    isMockData,
  };
}

export default useDashboardStats;
