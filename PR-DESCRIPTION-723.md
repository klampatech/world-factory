# WOR-723: Faction Goals + AI Behavior + Primal Beast Integration

## PR Description

Implements faction goals, AI behavior, and primal beast integration per SPEC.md §§5.4, 5.5, 5.6.

## Changes

### 1. API Endpoints (`src/api/v1/factions.rs`)
Added 5 new faction endpoints:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/factions/{id}/goals` | GET | List faction goals |
| `/api/v1/factions/{id}/goals` | POST | Add goal to faction |
| `/api/v1/factions/{id}/goals/{goal_id}` | GET | Get specific goal |
| `/api/v1/factions/{id}/beast-bonds` | GET | List beast bonds |
| `/api/v1/factions/{id}/beast-bonds` | POST | Add beast bond |

### 2. View Models (`src/api/models.rs`)
- `FactionGoalView` - goal id, type, description, progress, target, current, xp_reward, completed
- `BeastBondView` - beast_id, beast_name, bond_type, established_year, bonus_type, bonus_value
- Enhanced `FactionTurnStateView` - added resources_spent, xp, goals[], beast_bonds[]

### 3. AI Priority Scoring (`src/faction_integration.rs`)
Per SPEC.md §5.6:
- `calculate_priority_score()` - `priority_score = goal_progress_rate / turns_remaining`
- `score_actions_for_goal()` - ranks actions by expected progress per turn
- `apply_beast_bond_bonuses()` - applies alignment bonuses

## Testing
- Routes verified in `src/api/v1/factions.rs` lines 49-53
- Handlers implemented at lines 362-590
- AI scoring functions at lines 129-210

## Related Files
- `src/faction.rs` - Core types (FactionGoal, BeastBond, AlignmentBonus)
- `src/beasts/` - Primal beast definitions and slaying system
