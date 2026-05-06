/**
 * World Factory - Timeline Component
 *
 * Vertical timeline component for displaying historical events
 * Supports filtering, scrolling, and rich event cards
 */
import type { HistoricalEvent, TimelineFilter } from './TimelineTypes';
export interface TimelineComponentProps {
    /** World ID to load timeline for */
    worldId: string;
    /** Initial filter settings */
    initialFilter?: TimelineFilter;
    /** Callback when an event is selected */
    onEventSelect?: (event: HistoricalEvent) => void;
    /** CSS class name */
    className?: string;
    /** Height constraint */
    height?: string | number;
}
/**
 * Main Timeline Component
 */
export declare function TimelineComponent({ worldId, initialFilter, onEventSelect, className, height, }: TimelineComponentProps): any;
export default TimelineComponent;
//# sourceMappingURL=TimelineComponent.d.ts.map