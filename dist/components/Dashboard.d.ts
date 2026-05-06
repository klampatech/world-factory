/**
 * World Factory - Dashboard Component
 *
 * Displays world state summary with status, progress, and quick actions
 *
 * @see WOR-42: Dashboard - World State Summary
 */
export interface WorldSummary {
    id: string;
    name: string;
    status: 'generating' | 'ready' | 'failed';
    progress: number;
    createdAt: string;
    parameters?: {
        seed?: number;
        size?: string;
    };
}
/**
 * World-level state summary metrics (per-world statistics)
 */
export interface WorldStateMetrics {
    /** Current in-world year being displayed */
    currentYear: number;
    /** Total population across all species */
    totalPopulation: number;
    /** Population breakdown by species type */
    populationBySpecies: {
        species: string;
        population: number;
        percentage: number;
    }[];
    /** Number of active societies/civilizations */
    activeSocieties: number;
    /** Resource summary statistics */
    resources: {
        type: string;
        total: number;
        scarcity: 'abundant' | 'common' | 'rare' | 'critical';
    }[];
}
export interface DashboardState {
    loading: boolean;
    error: Error | null;
    worlds: WorldSummary[];
    total: number;
    /** Current selected world's metrics (null if no world selected) */
    selectedWorldMetrics: WorldStateMetrics | null;
}
export interface DashboardProps {
    /** Initial worlds to display (skip fetch if provided) */
    initialWorlds?: WorldSummary[];
    /** Maximum worlds to display */
    limit?: number;
    /** CSS class name */
    className?: string;
    /** Called when a world is selected */
    onWorldSelect?: (worldId: string) => void;
    /** Called when create new world is clicked */
    onCreateWorld?: () => void;
    /** Initial world state metrics to display */
    initialMetrics?: WorldStateMetrics | null;
}
/**
 * Dashboard component displaying world state summary
 *
 * Features:
 * - World list with status indicators
 * - World state summary (year, population, societies, resources)
 * - Progress bars for generating worlds
 * - Responsive grid layout
 * - Empty state with CTA
 * - Error state with retry
 */
export declare function Dashboard({ initialWorlds, limit, className, onWorldSelect, onCreateWorld, initialMetrics, }: DashboardProps): any;
export default Dashboard;
//# sourceMappingURL=Dashboard.d.ts.map