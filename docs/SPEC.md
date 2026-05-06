# World Factory - Procedural World & History Generation System

## 1. Overview

**Project Name:** World Factory  
**Type:** Headless procedural world generation server with HTML/Canvas visualization layer  
**Core Functionality:** Generate configurable earthlike planets with geological, climatic, and ecological systems; simulate pre-history with configurable time depth; track historical events and notable figures; support configurable "being" species with templated behaviors.  
**Target Users:** Game designers, worldbuilders, TTRPG campaign designers, procedural generation enthusiasts

---

## 2. Architecture

### 2.1 High-Level Components

```
┌─────────────────────────────────────────────────────────────┐
│                      World Factory                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ World Gen   │  │ History     │  │ Faction System      │ │
│  │ Engine      │  │ Simulator   │  │ (Phase 2)           │ │
│  │ (Rust)      │  │ (Rust)      │  │                     │ │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘ │
│         │                │                     │            │
│  ┌──────┴────────────────┴─────────────────────┴──────────┐ │
│  │              Persistence Layer (tarball/JSON)           │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              HTTP API Server (Rust)                     │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              HTML/Canvas Visualization                  │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Tech Stack

- **Core Engine:** Rust (performance, memory safety, excellent for complex simulation)
- **Persistence:** File-based (tarball containing JSON documents)
- **API:** HTTP server (Rust, e.g., Axum or Actix-web)
- **Visualization:** Static HTML/Canvas - single-page application served alongside the API

### 2.3 Data Model Overview

```
World
├── Planet
│   ├── Geography (elevation, tectonic plates, landmasses)
│   ├── Hydrology (oceans, rivers, lakes, groundwater)
│   ├── Atmosphere (weather systems, climate zones)
│   ├── Biomes (temperature, precipitation, flora, fauna)
│   ├── Resources (minerals, materials, organic)
│   └── TimeState (current year, time scale)
├── Beings (species templates with configurable behaviors)
├── Societies (factions, cultures, religions)
├── History (events, figures, artifacts)
└── CurrentState (active factions, settlements, conflicts)
```

---

## 3. World Generation Pipeline

### Phase A: Geography (Voronoi + Noise Hybrid)

Based on Amit Patel's polygonal map generation approach.

**A.1 Polygon Generation**
- Generate N random seed points using quasirandom/dithered distribution (blue noise approximation)
- Apply Lloyd relaxation (1-2 iterations) for even distribution
- Generate Voronoi diagram from seed points
- Result: irregular polygon mesh representing regions

**A.2 Elevation & Tectonics**

Elevation is measured in meters from a defined sea level baseline (0m). The ocean is always the lowest elevation. All other elevations are positive.

**Elevation Bands:**
|| Band | Meters | Description |
|-----|------|--------|-------------|
| 0 | Ocean deep | < -200 | Abyssal plain, ocean trenches |
| 1 | Ocean shelf | -200 to 0 | Continental shelf, shallow seas |
| 2 | Coastal | 0 to 50 | Beaches, marshes, deltas |
| 3 | Lowland | 50 to 300 | Rolling hills, river valleys, farmland |
| 4 | Midland | 300 to 800 | Forested hills, medium mountains |
| 5 | Highland | 800 to 1500 | Alpine meadows, high peaks |
| 6 | Mountain | 1500 to 4000 | Major mountain ranges |
| 7 | Extreme | > 4000 | Highest peaks (snow/capped) |

- Coastal tiles = elevation 0 (sea level)
- Mountains = highest elevation, positioned farthest from coastline
- Ocean elevation is negative (sea level = 0, ocean depths are below)
- Elevation is assigned using domain constraints (not raw noise):
  - Use graph-based approach where elevation follows gameplay constraints
  - Plate interiors trend toward erosion/acculation; plate boundaries produce uplift

**Plate Tectonics Simulation:**
- Group adjacent polygons into 5-10 tectonic plates randomly
- Define plate boundaries: divergent (rifting), convergent (subduction/collision), transform (shearing)
- Calculate crustal movement effects on elevation:
  - Convergent boundaries → uplift (mountains, volcanic arcs)
  - Divergent boundaries → rifting, low-lying rift valleys
  - Transform boundaries → fault lines, earthquakes (no elevation change)
- Apply erosion simulation (fluvial, thermal) over configurable time steps
- Interior plates erode slowly; boundary regions accumulate dramatic terrain

**A.3 Hydrology**
- Identify ocean regions (lowest elevation connected to map edges)
- Generate rivers: paths from mountains to coast using gradient descent
- Calculate drainage basins per polygon
- Groundwater stores based on precipitation and rock permeability

**A.4 Atmosphere & Weather (Simplified)**
- Define climate zones based on latitude and elevation
- Prevailing wind patterns
- Precipitation shadows (mountains block moisture)
- Temperature gradients

**A.5 Biome Assignment**
- Use temperature + precipitation matrix to assign biomes:
  - Tropical rainforest, tropical savanna, desert, grassland
  - Temperate forest, temperate rainforest, taiga, tundra
  - Arctic, alpine, freshwater, saltwater
- Ensure biome transitions are geographically coherent

### Phase B: Resources

Resources spawn based on biome type, geology, elevation, and time. Affinities are strict — resources rarely appear outside their preferred biomes, and never in implausible locations (e.g., fish on mountains).

#### B.1 Elevation Constraint

All resources are constrained by elevation band:

| Elevation Band | Suitable For |
|---------------|--------------|
| Ocean deep (< -200m) | Deep-water fish, oil, gas |
| Ocean shelf (-200m to 0m) | Fish, shellfish, coastal fish |
| Coastal (0m to 50m) | Salt marshes, coastal fish, shellfish |
| Lowland (50m to 300m) | All surface resources: farmland, timber, clay, iron, coal |
| Midland (300m to 800m) | Stone, timber (forested), iron, copper |
| Highland (800m to 1500m) | Stone, precious gems, alpine plants |
| Mountain (1500m to 4000m) | Stone, iron, copper, silver, gold, gems |
| Extreme (> 4000m) | Stone only (quarrying), no organic resources |

#### B.2 Resource Categories and Biome Affinities

Resources are grouped by category. Within each category, specific resources have named biome affinities. **Primary affinity** means 60-80% of deposits spawn there; **secondary affinity** means 15-30%; **rare** means < 5%.

**Organic Resources:**
|| Resource | Primary Affinity | Secondary Affinity | Rare/Absent | Notes |
|---------|----------------|--------------------|---------------------|-----|
| Fish (ocean) | Ocean shelf | Coastal (nearshore) | Inland | Schools in shallow, nutrient-rich waters |
| Shellfish | Coastal, saltwater marsh | Ocean shelf | — | Near shorelines and tidal zones |
| Whales | Ocean deep | Ocean shelf | — | Migratory, deep water |
| Game (deer/elk) | Temperate forest | Taiga, temperate rainforest | Grassland (fewer) | Herds need cover and water |
| Fur-bearing | Taiga | Tundra, temperate forest | — | Cold-climate species |
| Timber | Temperate forest | Tropical rainforest, taiga | — | Hardwood/softwood by biome |
| Wild game | Grassland | Savanna | Forest (hunted out) | Open-range grazing |
| Fertile soil | Grassland | Temperate forest | Desert, tundra | Agricultural potential |
| Herbs/medicinal | Temperate forest | Grassland | Desert | Specific altitude ranges matter |

**Mineral Resources:**
|| Resource | Primary Affinity | Secondary Affinity | Rare/Absent | Notes |
|---------|----------------|--------------------|---------------------|-----|
| Iron ore | Midland mountains, Highland | Lowland hills | Ocean, desert | Veins in metamorphic/sedimentary rock |
| Copper | Midland mountains, Highland | Lowland foothills | — | Often co-located with iron |
| Gold | Mountain (streams) | Midland alluvial | Ocean shelf ( placer) | Primary: quartz veins; placer: river deposits |
| Silver | Mountain, Highland | Midland streams | — | Often with lead/zinc |
| Gems (precious) | Mountain | Highland | — | Form in igneous intrusions |
| Stone/building rock | Any above 300m | Lowland (sedimentary) | Ocean, coastal | Quarry-grade rock |
| Clay | Lowland river valleys | Coastal deltas | Mountains | Alluvial deposits |
| Salt | Coastal (evaporation) | Desert (dry lakebeds) | — | Coastal flats or arid inland seas |

**Energy Resources:**
|| Resource | Primary Affinity | Secondary Affinity | Rare/Absent | Notes |
|---------|----------------|--------------------|---------------------|-----|
| Coal | Lowland (buried) | Midland | Coastal | Formed from ancient forests in sedimentary basins |
| Oil | Lowland sedimentary basins | Coastal (offshore) | Mountains | Trapped in porous sedimentary rock |
| Natural gas | Lowland (with oil) | Coastal (offshore) | — | Associated or non-associated |
| Peat | Tundra, marsh/swamp | Boreal forest | — | Accumulated organic matter |
| Uranium | Midland granitic | Lowland (sedimentary) | — | Associated with specific rock formations |

**Water Resources:**
|| Resource | Primary Affinity | Secondary Affinity | Rare/Absent | Notes |
|---------|----------------|--------------------|---------------------|-----|
| Freshwater rivers | Universal | — | Desert (absent) | Flows from highland to coast |
| Freshwater lakes | Lowland basins | Midland | Desert (absent) | Tectonic or glacial basins |
| Aquifer recharge | Lowland near rivers | Midland valleys | Mountain (poor) | Underground water stores |
| Ice | Arctic, alpine | Tundra | — | glaciers, ice sheets |

#### B.3 Spawning Rules

**Probability by affinity:**
- Primary biome + correct elevation band: 60-80% spawn chance per polygon
- Secondary biome + correct elevation: 15-30% spawn chance
- Outside affinity biome: < 5% (only if biome provides plausible conditions)
- Outside elevation band: 0% (hard constraint — fish don't spawn on mountains)

**Quantity/yield:**
- Base quantity determined by polygon size and geological setting
- Quality modifier: primary affinity → higher yield; secondary → normal; rare → lower yield
- Time modifier: older worlds accumulate more resources in stable geological settings
- Rare resources (gems, gold, oil) have 5-15% of normal spawn frequency

**Spatial clustering:**
- Mineral resources cluster along geological features: veins, faults, sedimentary basins
- Organic resources spread along biome gradients
- Oil/coal form in contiguous deposits (not scattered singletons)

**Biome hard constraints (no exceptions):**
- Fish/shellfish: never spawn on land biomes
- Forest resources (timber): never spawn on ocean, desert, tundra, or arctic
- Desert resources (salt, certain herbs): only spawn in desert biome
- Swamp resources (peat, certain fish): only spawn in freshwater biomes with water access

### Phase C: Time & Natural Events

**C.1 Time Model**
- Time unit: Year (configurable)
- Time scale is configurable per simulation
- Events can happen at sub-year granularity but world state updates yearly

**C.2 Natural Disasters**
| Disaster | Effect | Frequency |
|----------|--------|-----------|
| Flood | Alters river paths, terrain | Common in wet regions |
| Earthquake | Shifts elevation, creates mountains | Tectonic boundaries |
| Volcano | Creates new land, deposits minerals | Volcanic regions |
| Drought | Reduces water, desertification | Arid regions |
| Wildfire | Clears forests | Dry forested regions |
| Meteor impact | Major terrain alteration | Rare |

### Phase D: Natural Wonders

Special landmarks generated based on unique geographic combinations. Wonders must fall within world boundaries (not ocean), and have biome/elevation affinities.

#### D.1 Wonder Categories and Placement

| Wonder Type | Primary Biome Affinity | Elevation Range | Notes |
|-------------|----------------------|-----------------|-------|
| Grand canyon / gorge | Desert, grassland | Midland to Mountain | Requires significant river cutting through soft rock |
| Massive waterfall | Tropical rainforest, temperate rainforest | Highland (river drop) | Large river encountering sudden elevation change |
| Great cave system | Midland, mountain | Below surface | Limestone or volcanic tube regions |
| Sacred grove / ancient forest | Temperate forest, taiga | Lowland to Midland | Old-growth, ecologically significant |
| Volcanic peak | Highland, mountain | Highland+ | Active or dormant volcano |
| Glacier / ice field | Arctic, alpine, tundra | Highland to Extreme | Permanent ice |
| Oasis | Desert | Coastal to Lowland | Underground aquifer surfacing |
| Hot spring / geyser | Highland, mountain | Midland to Highland | Volcanic/geothermal activity |
| Great river | Universal | Flows coast to coast | Major river systems |
| Coastal arch / sea stack | Coastal | Ocean shelf to coastal | Wave erosion on coastline |
| Sandy desert | Desert | Lowland to Midland | Large sand sea erg |
| Mountain range | Tundra, alpine | Highland to Extreme | Multiple connected peaks |
| Island chain | Ocean | Ocean shelf | Volcanic or tectonic origin |
| Coral reef | Coastal (saltwater) | Ocean shelf | Warm, shallow, clear water |

#### D.2 Wonder Placement Rules

- **Within world boundaries**: No wonder may be placed in the open ocean (continental shelf islands are permitted)
- **Biome-specific**: Wonders must match the biome of their location (no glacier in a desert)
- **Elevation constraint**: Wonders that require water must be at elevations where water is plausible (river gorges at elevation > 0)
- **Geological setting**: Mountain wonders require mountainous terrain; coastal wonders require coastline
- **Uniqueness**: Each wonder type may only appear once per world (grand canyon is unique)
- **Scaling**: Very small worlds (≤ 32x32) may skip large-scale wonders (canyon, mountain range)

#### D.3 Artifact Placement

Artifacts are a class of powerful items that arise from specific conditions combining resources, notable figures, and historical events. Artifact placement is not random — it follows causal chains:

**Artifact Prerequisites:**
| Artifact Type | Required Conditions | Typical Location |
|---------------|-------------------|-----------------|
| Legendary weapon | Iron/gold deposit + notable warrior figure | Capital city, battlefield |
| Ancient tome | Civilized biome (not wilderness) + scholar figure | Library, temple, ruin |
| Sacred relic | Religious site + religious figure | Temple, shrine, holy site |
| Magical artifact | Gem deposit + historical event + magical tradition | Museum, vault, ruin |
| Crown/regalia | Gold deposit + centralized government | Capital city |
| Map to treasure | Rare resource + secrecy event | Ruins, hidden cache |
| Ancient artifact | Pre-history civilization + survived ruin | Archaeological site |
| Remnant artifact | Primal beast slain — dropped on death | Location of beast's death |

**Placement rules:**
- Artifacts always exist at a specific location (settlement, landmark, or geographical feature)
- Causal link required: e.g., a crown requires gold mines AND a centralized society capable of producing royalty
- Artifacts may only exist in the past or present — they cannot "spawn" during the current timeline uncaused
- **In-world artifacts**: Discoverable by historical figures, tradeable, stealable
- **Dormant artifacts**: Require activation conditions (specific location + event trigger)
- **Cataclysmic artifacts**: Rare (< 0.1% per year per artifact), world-altering when activated (flood, fire, transformation)

#### D.4 Primal Beasts and Spirits

Ancient elemental beings of immense power, tied to the fundamental forces of the world. Each world has one primal spirit per primary element. These beings predate civilization and are not created by it — they are fundamental expressions of the world's elemental forces.

**Core Properties (all primal beasts share):**
- **Immortal unless slain**: Primal beasts do not die of age or natural causes
- **Unique per world**: Only one fire beast, one water beast, etc. may exist at any time
- **World-bound**: They cannot leave the world's boundaries
- **Power scales with world age**: Older worlds produce more powerful primal beasts
- **Territorial**: Each beast claims and transforms a region around itself

#### D.4.1 The Four Primary Elemental Beasts

| Beast | Element | Biome Affinity | Power Domain |
|-------|---------|---------------|-------------|
| **Pyraxes, the Flame Wyrm** | Fire | Volcanic, desert, highland | Volcanoes, earthquakes, geothermal vents |
| **Tidarth, the Storm Leviathan** | Air | Ocean shelf, coastal, open ocean | Hurricanes, lightning storms, tidal waves |
| **Terros, the Stone Titan** | Earth | Mountain, highland, midland | Earthquakes, mountain formation, landslides |
| **Lumina, the Tide Singer** | Water | Ocean, freshwater lakes, rivers | Droughts, floods, ocean currents, marine life |

#### D.4.2 Elemental Beast Profiles

**Pyraxes, the Flame Wyrm (Fire)**
- *Form*: A massive serpentine dragon of living flame, 50-200m long depending on world age
- *Habitat*: Volcanic peaks, magma chambers, desert deep
- *Territory effect*: Regions near Pyraxes become volcanic or desert. Vegetation dies within 10km. Stone is smelted into pure metals at the surface.
- *Environmental interaction*: Pyraxes moving through a mountain range causes volcanic eruptions along its path. Sleeping near a populated area causes heat waves and crop failure.
- *Power growth*: Absorbs heat from volcanic activity. Older worlds = larger, more destructive Pyraxes.
- *Weakness*: Cannot cross large bodies of ocean. Cold climates suppress its power significantly.

**Tidarth, the Storm Leviathan (Air/Water)**
- *Form*: A colossal whale-like creature 300-500m long, cloaked in perpetual storm clouds that extend 50km in all directions
- *Habitat*: Open ocean, migrates along coastlines
- *Territory effect*: The waters and skies around Tidarth are perpetually stormy. Its "island" is the mass of floating debris and sea life that accumulates on its back — it functions as a moving island. Sailors who land on Tidarth's back find freshwater pools, strange vegetation, and are often lost.
- *Environmental interaction*: Ships caught near Tidarth are battered by storms and often shipwrecked. Coastal cities near its migration path experience seasonal hurricanes.
- *Power growth*: Grows larger with each major storm it weathers. Ocean trade routes shift to avoid its territory.
- *Weakness*: Becomes sluggish in enclosed seas. Freshwater regions repel it.

**Terros, the Stone Titan (Earth)**
- *Form*: A humanoid figure of living rock, 100-300m tall, resembling a mountain given legs
- *Habitat*: Mountain ranges, deep underground caverns
- *Territory effect*: Mountains near Terros grow. Caverns deepen and expand. Precious metals and gems form in its wake. Earthquakes precede its movement.
- *Environmental interaction*: Terros walking through a valley creates a mountain range. Sleeping under a city causes subsidence and foundation cracking.
- *Power growth*: Absorbs minerals and stone. Older worlds = taller, more mineral-rich Terros.
- *Weakness*: Moves extremely slowly (1-5km/year). Cannot cross oceans or large lakes. Rivers divert around it instinctively.

**Lumina, the Tide Singer (Water)**
- *Form*: A translucent serpentine fish-dragon, 100-400m long, surrounded by bioluminescent light
- *Habitat*: Deep ocean, migrates up rivers to freshwater lakes
- *Territory effect*: Marine life flourishes within 200km of Lumina. Coral reefs grow exponentially. Its song is heard in coastal settlements and is considered sacred or ominous depending on the culture.
- *Environmental interaction*: Lumina's death causes immediate marine life collapse within its territory. Droughts spread if Lumina is denied access to freshwater spawning grounds. Its movement causes predictable monsoon patterns.
- *Power growth*: Grows with ocean health and freshwater access. Older worlds = longer, more luminous Lumina.
- *Weakness*: Cannot survive in volcanic or polluted waters. Acidic oceans weaken it severely.

#### D.4.3 Beast-World Interactions

**Environmental effects while alive:**
| Beast | While Active | While Sleeping/Dormant |
|-------|------------|----------------------|
| Pyraxes | Volcanic activity +15%, desert expansion | Normal volcanic baseline |
| Tidarth | Storm frequency +50%, hurricane risk | Calm seas within 100km |
| Terros | Earthquake risk +20%, mountain growth | Normal seismic baseline |
| Lumina | Marine life +30%, fishing yields high | Marine life -40%, fishing collapses |

**Consequences of death:**
| Beast | Immediate Effect | Long-term Effect (1-10 years) | Remnant Drop |
|-------|----------------|---------------------------|-------------|
| Pyraxes | All volcanoes go dormant | Global cooling (-2°C to -5°C), volcanic metals stop surfacing, ore veins deplete 50% faster | **Pyraxes' Heartstone**: A gem of living flame the size of a house. Radiates heat in a 10km radius; placed in a volcano permanently ignites volcanic activity. Can be used as a forge core for fire-elemental weapon/armor crafting with no fuel cost. |
| Tidarth | All storms cease | Ocean currents destabilize, El Niño conditions permanent, hurricane patterns dissolve | **Tidarth's Storm Eye**: A calm, perfectly clear sphere of air 3m in diameter. Inside is perpetually windless. When broken over water, generates a hurricane. When used in construction, allows a building to withstand any storm. |
| Terros | Mountains begin eroding rapidly | Geological instability, earthquakes increase 300%, mineral deposits collapse | **Terros' Primordial Core**: A chunk of pre-continental stone, heavier than it appears. Provides structural integrity to any structure built on it. Can be used to create underground vaults that cannot be collapsed by any earthquake. |
| Lumina | All marine life dies | Ocean becomes dead zone, fisheries collapse globally, freshwater sources dry up | **Lumina's Life Pearl**: A pearl containing the last living essence of the ocean. Keeps a 50km radius of ocean healthy. Can purify any freshwater source and restore fish populations. If consumed, grants water breathing and communion with sea creatures. |

**Slaying mechanics:**
- Slaying a primal beast requires a coordinated effort across multiple factions
- Minimum requirements: 3+ nations cooperating, access to the beast's weakness, legendary artifacts aligned to the element
- The act of slaying itself is a historical event of severity 10 (world-altering)
- The slaying faction inherits a "curse" aligned to the beast's element (e.g., slaying Pyraxes makes the region permanently volcanic)
- **Remnant artifact**: When a primal beast is slain, it drops a physical piece of itself — its **Remnant**. This Remnant becomes an artifact of immense power. It is not merely a trophy — it contains the beast's residual essence and continues to exert a weaker version of the beast's environmental effects in its immediate vicinity. The Remnant can be used in crafting, construction, or rituals to produce world-quality elemental goods.
- **Remnant stability**: Remnants are stable but have a decay rate. Over 100-500 years (depending on world age), a Remnant slowly loses power. It cannot be destroyed — only re-used or sealed.
- **Remnant curse**: The curse from slaying is carried in the Remnant, not the slayer. Whoever possesses the Remnant suffers the curse effects. A faction may slay a beast but give the Remnant to another faction to transfer the curse.

#### D.4.4 Faction Interactions with Primal Beasts

Factions have three possible relationships with each primal beast:

**1. Control (rare, dangerous)**
- Requires a faction to bind the beast through rituals, artifacts, or magical traditions
- Control grants the faction elemental power: volcano blessed by Pyraxes grants fire immunity; Tidarth's blessing calms storms for allied ships
- The beast can break control at any time (loyalty not guaranteed)
- Only one faction can attempt to control a given beast at a time

**2. Alignment (pious relationship)**
- Faction establishes religious or cultural ties to the beast
- Alignment grants minor blessings: Lumina-aligned factions get +20% fishing yield; Terros-aligned get +mineral deposits
- Alignment does not grant control — the beast remains aloof
- Multiple factions may be aligned to the same beast simultaneously

**3. Avoidance (default)**
- Most factions neither control nor align with primal beasts
- Historical events may force interaction (Tidarth migrating through a faction's fishing grounds)
- Treaties may be established to partition migration routes

#### D.4.5 Beast Movement

**Movement patterns:**
- Primal beasts move slowly and predictably (1-10km/year)
- Movement follows elemental gradients: Pyraxes toward volcanic activity, Lumina along ocean currents
- Factions can predict movement based on world geography and elemental map data
- Movement can be temporarily redirected by major events (earthquake diverts Terros; new volcano attracts Pyraxes)

**Territory size:**
- Each beast claims a territory radius of 50-200km depending on world age
- Claimed territory is not "owned" — factions may settle there but face environmental consequences
- Two beasts' territories may overlap: overlap zones experience mixed elemental effects (volcanic coast = Pyraxes + Tidarth territory)

---

## 4. History Generation

### 4.1 Pre-History Generation

Generate configurable years of history before "present day":

**D.1 Being/Species Templates**
```yaml
species_template:
  name: "Human"
  base_traits:
    - name: "Curious"
      effect: "+10% discovery rate"
    - name: "Territorial"
      effect: "Built-in border defense"
  behaviors:
    expansion: 0.7  # tendency to expand territory
    cooperation: 0.5  # tendency to form alliances
    resource_gathering: 0.8
  societies:
    - type: "Tribe"
    - type: "Chiefdom"
    - type: "Nation"
```

Users can define custom species via configuration.

**D.2 Civilization Emergence**
- Beings spawn in suitable biomes (not deserts, tundra, ocean)
- Population grows over time based on resources
- Beings form groups: families -> bands -> tribes -> chiefdoms -> nations

**D.3 Historical Events**
Events are generated probabilistically based on world state:

| Event Type | Effects | Triggers |
|------------|---------|----------|
| Settlement Founded | New city, population | Suitable location |
| War Declared | Conflict between societies | Border tension, resources |
| Migration | Population movement | Overpopulation, disaster |
| Plague | Population reduction | Dense settlements |
| Discovery | Technology advance, resources | Proximity to features |
| Cultural Shift | Trait changes | Random or triggered |
| Artifact Creation | Powerful item created | Rare, special conditions |
| Artifact Activation | World-altering effect | Cataclysmic, very rare |

**D.4 Notable Figures**
Each significant event generates a historical figure:
```json
{
  "id": "uuid",
  "name": "Thorin Ironforge",
  "species": "human",
  "birth_year": -450,
  "death_year": -380,
  "titles": ["King", "Conqueror", "Smith"],
  "deeds": [
    {"type": "founded_city", "name": "Ironhold", "year": -420},
    {"type": "won_battle", "name": "Battle of Red Mesa", "year": -400},
    {"type": "discovered_artifact", "name": "Crown of Ages", "year": -390}
  ],
  "impact_score": 85
}
```

**D.5 Historical Artifacts**
Artifacts are rare items with world-altering potential:
- Creation requires specific conditions (rare resources + notable figure + time)
- Activation effects: terrain destruction, population devastation, technology leaps
- History log records all artifact activities
- Cataclysmic events are very rare (probability < 0.1% per year per artifact)

**D.6 Faction Territory and Political Maps**

Political maps do not paint the entire world. Faction territories are clustered, leaving significant gaps between powers unless the world is particularly old (high `pre_history_years`).

**Territory Generation Rules:**

| Rule | Description |
|------|-------------|
| Clustered centers | Factions start with 1-3 connected settlements, then expand outward radially |
| Expansion frontier | Each generation step allows a faction to claim 1-3 adjacent polygons |
| Gap preservation | Minimum 2-3 unclaimed polygons between rival faction borders at all times |
| Contested zones | Wars create temporary "contested" polygon status between opposing borders |
| Ocean exclusion | Factions never claim deep ocean tiles (elevation < -200m) unless holding an island |
| Coastal access | Factions bordering the sea may claim ocean shelf tiles (to -200m) for fishing rights |
| Island claims | Islands off the coast may be claimed if a faction holds the adjacent mainland |
| Strait control | Factions may claim ocean shelf tiles that form natural straits between landmasses |
| Landlocked preference | Starting factions prefer lowland to midland elevations (0-800m) |
| Mountain barriers | Mountain ranges (> 1500m) naturally limit expansion and serve as borders |
| Age scaling | Worlds with `pre_history_years < 200`: 1-2 factions, small territories |
| | Worlds with `pre_history_years 200-500`: 2-4 factions, moderate expansion |
| | Worlds with `pre_history_years > 500`: 4+ factions, full continental reach |

**No Ocean Tile Claiming (default):**
- Deep ocean (> -200m): No faction may claim as core territory
- Exceptions require specific justification in world history:
  - Faction holds a significant island chain
  - Faction controls a critical strait or chokepoint
  - Ancient treaty or magical claim (documented in history log)
- Coastal fishing rights may be claimed on ocean shelf tiles without full territorial claim

**Territory Data Structure:**
```json
{
  "faction_id": "uuid",
  "claimed_polygons": ["poly_id_1", "poly_id_2", ...],
  "core_territory": ["poly_id_main_capital", ...],
  "client_states": ["poly_id_vassal_1", ...],
  "contested": [
    {"polygon_id": "x", "rival_faction_id": "y", "since_year": -50}
  ],
  "strait_controls": [{"polygon_id": "x", "strategic_value": "high"}]
}
```

**Expansion Algorithm:**
1. For each faction, determine "pressure score" = population / territory size (crowding)
2. High-pressure factions attempt expansion to adjacent unclaimed polygons
3. Expansion target must be biome-compatible (no desert factions expanding into tundra without migration event)
4. Expansion into rival territory requires a war event
5. Result: natural-looking borders that leave wilderness between civilizations

---

## 5. Persistence

### 5.1 Storage Format

Worlds stored as tarball archives (.wfw extension):

```
my_world.wfw/
├── world.json          # Top-level world metadata
├── planet/
│   ├── geography.json  # Polygon mesh, elevation, hydrology
│   ├── biomes.json     # Biome assignments per polygon
│   ├── resources.json  # Resource locations and quantities
│   └── weather.json    # Climate and weather patterns
├── history/
│   ├── events.json     # All historical events chronologically
│   ├── figures.json    # Notable people
│   └── artifacts.json  # Historical artifacts
├── societies/
│   ├── factions.json   # Current factions/societies
│   └── settlements.json # Cities, towns, villages
└── time/
    └── state.json      # Current year, time scale, active disasters
```

### 5.2 World Metadata
```json
{
  "id": "uuid",
  "name": "Middle Earth Clone",
  "created_at": "2024-01-15T10:30:00Z",
  "config": {
    "width": 64,
    "height": 64,
    "pre_history_years": 500,
    "time_scale": "years",
    "seed": 12345
  },
  "current_year": 1247,
  "planet_type": "earthlike"
}
```

### 5.3 Operations
- **Save:** Pack world state into tarball
- **Load:** Unpack tarball into memory
- **List:** Show all saved worlds in storage directory
- **Delete:** Remove world tarball

---

## 6. Visualization (HTML/Canvas)

### 6.0 Landing Page (World Selector)

When the server starts, `GET /` serves a **World Selector** landing page:

**URL:** `GET /` (root)

**Purpose:** List all available worlds with quick access to their visualizations, and create new worlds.

**Response:** HTML page with:
- Header: "World Factory" with server status
- **Generate New World form** (modal or inline):
  - Name field (required)
  - Width/Height sliders (default: 64x64, max: 128)
  - Pre-history years input (default: 100, range: 0-1000)
  - Seed input (optional, auto-generated if empty)
  - Species selection (multi-select: Human, Elf, Dwarf, Orc, Halfling)
  - Resource richness dropdown (Poor / Normal / Rich / Abundant)
  - Disaster frequency dropdown (Low / Medium / High)
  - "Generate" button → calls `POST /api/worlds`
  - On success → redirects to `GET /worlds/:id`
  - On error → shows error message in form
- World list cards showing:
  - World name
  - World ID (for debugging)
  - Generation status badge (generating / ready / failed)
  - Progress bar for worlds still generating
  - "View Map" button → navigates to `/worlds/:id/map`
  - "View Timeline" button → navigates to `/worlds/:id/timeline`
  - "View Dashboard" button → navigates to `/worlds/:id/dashboard`
- Footer with server info

**Page Layout:**
```
┌─────────────────────────────────────────────────────────────┐
│  🌍 World Factory                      [Server: Running]   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [+ Generate New World]                                      │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Middle Earth Clone                                   │   │
│  │ ID: a1b2c3d4-...  |  Status: ✅ Ready               │   │
│  │ 64×64 | 500 pre-history years | 342 events          │   │
│  │                                                      │   │
│  │ [View Map] [Timeline] [Dashboard]                    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Thorin Test World                                    │   │
│  │ ID: e5f6g7h8-...  |  Status: 🔄 Generating... 45%   │   │
│  │ 32×32 | 100 pre-history years                        │   │
│  │ [View Map] [Timeline] [Dashboard]                    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Generate New World Modal:**
```
┌─────────────────────────────────────────────────────────────┐
│  Generate New World                                    [X]  │
├─────────────────────────────────────────────────────────────┤
│  Name:        [________________________]                     │
│                                                             │
│  Dimensions:  Width: [====|====] 64                         │
│              Height: [====|====] 64                         │
│                                                             │
│  Pre-history:  [____100____] years                          │
│  Seed:         [__________] (optional, auto-generated)      │
│                                                             │
│  Species:      ☑ Human  ☑ Elf  ☐ Dwarf  ☐ Orc  ☐ Halfling │
│                                                             │
│  Resources:    ( ) Poor  (•) Normal  ( ) Rich  ( ) Abundant│
│  Disasters:    ( ) Low  (•) Medium  ( ) High               │
│                                                             │
│                    [Cancel]  [Generate World]               │
└─────────────────────────────────────────────────────────────┘
```

### 6.1 View Modes

#### World Map View
**URL:** `GET /worlds/:id/map`

- Canvas rendering of world map (full viewport)
- Color-coded by biome type
- Overlay toggle buttons: Resources | Political | Elevation
- Zoom controls: Fit | 50% | 100% | 200%
- Pan via drag
- Mini-map in corner for orientation
- Click polygon for details panel (right sidebar)

#### History Timeline View
**URL:** `GET /worlds/:id/timeline`

- Vertical timeline of events
- Filter bar: All | War | Settlement | Discovery | Plague | etc.
- Search by figure or place name
- Click event to expand details (year, description, participants)
- Click figure to see biography popup

#### World Dashboard View
**URL:** `GET /worlds/:id/dashboard`

- Current year display (large, prominent)
- Active disasters count with icons
- Population totals by species (pie chart)
- Resource summary (bar chart)
- Notable figures spotlight (3-5 top impact)
- Recent events list (last 10)

#### World Detail Page
**URL:** `GET /worlds/:id`

- World metadata (name, seed, dimensions, creation date)
- Generation configuration
- Status with progress bar (if still generating)
- Tabs: Overview | Map | Timeline | Dashboard
- Navigation between views

### 6.2 Navigation

**Global Navigation:**
```
┌─────────────────────────────────────────────────────────────┐
│  🌍 WF  │ World Selector │ Map: [World] │ Timeline │ Dash   │
└─────────────────────────────────────────────────────────────┘
```

- Clicking "World Selector" returns to `GET /`
- World name in header links to `GET /worlds/:id`
- Tab navigation for current world's views

**URL Structure:**
```
GET  /                           # Landing page (world list)
GET  /worlds/:id                 # World detail/overview
GET  /worlds/:id/map             # Map visualization
GET  /worlds/:id/timeline        # History timeline
GET  /worlds/:id/dashboard       # World dashboard
```

### 6.3 Interaction
- Read-only viewing (default)
- Click regions to see details
- Hover for tooltips
- Overlay toggles (resources, elevation, political)
- Export map as PNG (via canvas export button)

---

## 7. API Design

### 7.1 Endpoints

```
GET    /                            # Landing page (World Selector HTML)
POST   /api/worlds                  # Generate new world
GET    /api/worlds                  # List all worlds
GET    /api/worlds/:id             # Get world metadata
GET    /api/worlds/:id/planet      # Get planet data
GET    /api/worlds/:id/history      # Get history
GET    /api/worlds/:id/history/events # Get events (paginated)
GET    /api/worlds/:id/societies    # Get societies/factions
GET    /api/worlds/:id/figures     # Get notable figures
DELETE /api/worlds/:id              # Delete world
GET    /api/worlds/:id/export       # Download as tarball

POST   /api/worlds/:id/simulate    # Advance time (Phase 2)
GET    /api/worlds/:id/map         # Get map render data

GET    /worlds/:id                 # World overview page (HTML)
GET    /worlds/:id/map             # Map view page (HTML)
GET    /worlds/:id/timeline        # Timeline view page (HTML)
GET    /worlds/:id/dashboard        # Dashboard view page (HTML)
```

### 7.2 World Generation Request

**Create a new world:**

```json
POST /api/v1/worlds
{
  "name": "My Fantasy World",
  "config": {
    "width": 64,
    "height": 64,
    "pre_history_years": 500,
    "seed": 12345,
    "species_templates": ["human"],
    "disaster_frequency": "medium",
    "resource_richness": "normal"
  }
}
```

**Response (202 Accepted):**
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "My Fantasy World",
    "status": "generating",
    "progress": 0.0,
    "created_at": "2026-05-05T18:00:00Z",
    "parameters": {
      "seed": 12345,
      "width": 64,
      "height": 64,
      "pre_history_years": 500
    }
  }
}
```

**Query world status:**
```json
GET /api/v1/worlds/550e8400-e29b-41d4-a716-446655440000

{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "My Fantasy World",
    "status": "ready",
    "progress": 1.0,
    "created_at": "2026-05-05T18:00:00Z",
    "parameters": { ... }
  }
}
```

**Status values:** `generating` | `ready` | `failed`

**Configuration options:**
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `width` | u32 | 64 | World width (max: 128) |
| `height` | u32 | 64 | World height (max: 128) |
| `pre_history_years` | u32 | 100 | Years of history to simulate |
| `seed` | u64 | auto | RNG seed for deterministic generation |
| `species_templates` | string[] | ["human"] | Species to generate |
| `disaster_frequency` | string | "medium" | low / medium / high |
| `resource_richness` | string | "normal" | poor / normal / rich / abundant |

---

## 7.5 Docker & Deployment

### 7.5.1 Docker Configuration

World Factory can be run as a persistent HTTP server in Docker for local development and API testing.

**Dockerfile**
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release
EXPOSE 8080
CMD ["target/release/world_factory", "serve", "--port", "8080"]
```

**Multi-stage build (smaller image)**
```dockerfile
FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y pkg-config && \
    cargo build --release && \
    apt-get clean && rm -rf /var/lib/apt/lists/*
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && \
    apt-get clean && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/world_factory /usr/local/bin/
EXPOSE 8080
CMD ["world_factory", "serve", "--port", "8080", "--host", "0.0.0.0"]
```

### 7.5.2 Docker Compose

```yaml
version: '3.8'
services:
  world-factory:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - world_factory_data:/root/.local/share/world-factory
    environment:
      - RUST_LOG=info
      - WORLD_FACTORY_PORT=8080
      - WORLD_FACTORY_HOST=0.0.0.0
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s

volumes:
  world_factory_data:
```

### 7.5.3 Persistent Server Mode

The server starts in persistent mode with:
```bash
world_factory serve --port 8080 --host 0.0.0.0
```

This runs indefinitely, maintaining world state in memory. All generated worlds are persisted to the data volume.

### 7.5.4 API Testing Workflow

1. **Start the server**
   ```bash
   docker compose up -d world-factory
   ```

2. **Create a test world**
   ```bash
   curl -X POST http://localhost:8080/api/worlds \
     -H "Content-Type: application/json" \
     -d '{
       "name": "Test World",
       "config": {
         "width": 32,
         "height": 32,
         "pre_history_years": 100,
         "seed": 42
       }
     }'
   ```

3. **Poll for generation status**
   ```bash
   curl http://localhost:8080/api/worlds/<id>
   # Returns: { "status": "generating" | "ready" | "failed" }
   ```

4. **Fetch generated data**
   ```bash
   curl http://localhost:8080/api/worlds/<id>/map
   curl http://localhost:8080/api/worlds/<id>/timeline
   curl http://localhost:8080/api/worlds/<id>/figures
   ```

5. **Run multiple configurations rapidly**
   ```bash
   # Stress test with various configs
   for size in 16 32 64; do
     curl -X POST http://localhost:8080/api/worlds \
       -d "{\"name\":\"Size${size}\",\"config\":{\"width\":${size},\"height\":${size}}}"
   done
   ```

### 7.4 CLI-to-Server World Persistence

CLI-generated worlds are **not automatically visible to the API server**. To see CLI-generated worlds in the server's web UI, the world must be exported to the server's configured `WORLD_FACTORY_DATA_DIR`.

#### How It Works

1. **CLI generation** writes output to the CLI process's own `WORLD_FACTORY_DATA_DIR` (or its default: `~/.local/share/world-factory/generated/<world_id>/`)
2. **Server** reads worlds from its own `WORLD_FACTORY_DATA_DIR` at startup and via `GET /api/v1/worlds`
3. **Disconnected by default** — CLI and server have independent storage unless configured to share the same directory

#### Using a Shared Data Directory

Configure both CLI and server to use the same data directory via the `WORLD_FACTORY_DIR` environment variable:

```bash
# Generate a world with shared storage
WORLD_FACTORY_DIR=/tmp/worlds cargo run -- generate --width 32 --height 32 --seed 42

# Start the server with the same directory
WORLD_FACTORY_DIR=/tmp/worlds cargo run --features api -- --server --port 8080

# The world is now visible at GET /api/v1/worlds
```

#### Exporting a CLI World to a Specific Directory

The CLI should support an explicit `--export-to` flag to save a generated world to a target directory:

```bash
world-factory generate --width 64 --height 64 --seed 12345 --export-to ~/.local/share/world-factory
```

If `--export-to` is provided, after successful generation the world is saved as a `.wfw` tarball in `<export_dir>/generated/<world_id>/world.wfw`, including all metadata needed for the server to list and serve it.

#### Automatic Export on Generation

When the CLI generates a world, it should automatically persist it to `WORLD_FACTORY_DIR` (not just print output). This requires the generation pipeline to call `packaging::save_world_package()` with the correct storage path after successful generation.

#### Acceptance Criteria

| # | Criterion | Verification |
|---|-----------|-------------|
| 1 | `cargo run -- generate --width 32 --height 32 --seed 42` saves a `.wfw` file to `WORLD_FACTORY_DIR/generated/` | Check file exists after generation |
| 2 | Starting the server with the same `WORLD_FACTORY_DIR` lists the CLI-generated world at `GET /api/v1/worlds` | Server startup log shows world loaded |
| 3 | The exported world has valid `world.json` metadata with id, name, created_at, config fields | Inspect `.wfw` tarball contents |
| 4 | `--export-to <path>` saves to the specified directory instead of default | Compare storage paths |
| 5 | Running `generate` twice with same seed produces same world id (deterministic) | Compare exported world ids |

### 7.5.5 Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `WORLD_FACTORY_PORT` | `8080` | HTTP server port |
| `WORLD_FACTORY_HOST` | `127.0.0.1` | Bind address |
| `WORLD_FACTORY_DATA_DIR` | `~/.local/share/world-factory` | Data storage path |
| `RUST_LOG` | `info` | Logging level |

---

## 8. Implementation Phases

### Phase 1: Core World Generation (4-6 weeks)

**1.1 Project Setup**
- Rust project with Cargo workspace
- Basic HTTP server (Axum or Actix-web)
- Storage directory management
- Configuration system

**1.2 Geography Engine**
- Voronoi generation with Lloyd relaxation
- Elevation assignment with domain constraints
- Basic plate tectonics simulation
- River generation (gradient descent paths)
- Erosion simulation

**1.3 Climate & Biomes**
- Climate zone calculation
- Biome assignment matrix
- Basic weather patterns

**1.4 Resource System**
- Resource category definitions
- Biome-based spawning rules
- Resource quantity calculation

**Deliverable:** Generate a complete earthlike planet with geography, biomes, and resources.

### Phase 2: History Generation (4-6 weeks)

**2.1 Being/Species System**
- Species template definition format (YAML/JSON)
- Basic human template with default behaviors
- Species injection architecture

**2.2 Civilization Emergence**
- Settlement spawning algorithm
- Population growth model
- Group hierarchy (bands -> tribes -> nations)

**2.3 Event Generation**
- Event probability system
- Event effect application
- Event types: settlement, war, migration, plague, discovery
- Historical record keeping

**2.4 Notable Figures**
- Figure generation from events
- Biography system
- Impact scoring

**2.5 Historical Artifacts**
- Artifact creation conditions
- Artifact effects
- Cataclysmic event system (rare)

**Deliverable:** Generate configurable years of history with events, figures, and artifacts.

### Phase 3: Persistence & API (2-3 weeks)

**3.1 Storage Layer**
- Tarball creation/extraction
- JSON serialization of all entities
- World listing, loading, saving, deletion

**3.2 HTTP API**
- All endpoints from Section 7
- Error handling
- Request validation

**Deliverable:** Worlds can be saved, loaded, and listed via API.

### Phase 4: Visualization (3-4 weeks)

**4.1 HTML/Canvas Map**
- Biome color mapping
- Zoom and pan controls
- Region highlighting

**4.2 History Viewer**
- Timeline rendering
- Event filtering
- Figure profiles

**4.3 Dashboard**
- World state summary
- Current year display
- Active events

**Deliverable:** Browser-based read-only visualization of generated worlds.

### Phase 5: Faction Turn System

> **Reference:** SWN faction system implementation in [MichaelBlackwell/SWN3](https://github.com/MichaelBlackwell/SWN3) — specifically `src/store/slices/turnSlice.ts`, `src/services/turnManager.ts`, and `src/types/faction.ts`.

Factions exist as persistent political actors on the world map after Phase 4 history generation completes. This phase activates a turn-based strategy layer where one faction is player-controlled and the rest are AI-managed. The system is designed to run without a DM — all rules are deterministic and encoded.

---

#### 5.0 World State → Faction Turn Bridge

Before the first turn, the world state is frozen from history generation and translated into faction resources:

**Faction Attributes** (three primary stats):

| Attribute | Represents | Modifiers |
|-----------|-----------|-----------|
| **Force** (f) | Military strength | Territory count, standing armies, martial culture |
| **Cunning** (c) | Political/espionage power | Settlements with intelligence traditions, trade volume |
| **Wealth** (w) | Economic resources | Resource richness, trade routes, commercial settlements |
| **HP / MaxHP** | Faction stability | Base 10 + sum of (f + c + w) / 3 |

**From History Phase (D.6) into Faction State:**

```
territory_polygons.json → faction.territories[] (list of tile IDs)
societies/factions.json → faction.type, faction.tags[], faction.name
societies/settlements.json → faction.assets[] (each settlement = 1 base asset)
global_factions_resource_pool → faction.xp, initial fac_creds
```

**Faction Tags** — 5 tags max per faction, assigned at generation from history context:

| Tag | Effect |
|-----|--------|
| `Colonists` | +1 to settlement establishment actions |
| `Martial` | +1 to Attack action effectiveness |
| `Merchant` | +1 to income phase FacCred generation |
| `Devout` | +1 to alignment actions with primal beasts |
| `Nomadic` | Assets can move 2× normal range per Move action |
| `Ancient` | Starts with 1d4 relic artifacts from history |
| `Barbarian` | Cannot purchase assets; must capture them |
| `Technical` | Asset upgrade costs reduced 25% |
| `Pirate` | Can raid trade routes for FacCred income |
| `Imperialist` | Territory expansion pressure +50% |

*Additional tags drawn from SWN repertoire as needed: Deep Rooted, Fanatical, Machiavellian, Mercenary Group, Plutocratic, Preceptor Archive, Psychic Academy, Savage, Scavengers, Secretive, Theocratic.*

**Faction Goal** — one active goal at a time:

| Goal Type | Win Condition | XP Reward |
|-----------|--------------|-----------|
| Military Conquest | Control 60% of world territory | 4 |
| Commercial Expansion | FacCred income ≥ 50/turn for 3 consecutive turns | 3 |
| Cultural Dominance | 80% of settlements have faction's culture | 3 |
| Primal Allegiance | Control or align with 2 primal beasts | 4 |
| Destroy the Foe | Target faction reduced to 0 HP | 3 |
| Expand Influence | Control 50% more settlements than turn 1 | 2 |
| Peaceable Kingdom | No wars declared for 5 consecutive turns | 2 |
| Inside Enemy Territory | Settlements established in rival territory ≥ 5 | 3 |
| Wealth of Worlds | Accumulate 500 FacCred total | 2 |
| Blood the Enemy | Deal 100+ total HP damage to named rival | 2 |

XP is spent to recruit new hero units, upgrade assets, or reroll failed conflict dice.

---

#### 5.1 Turn Structure

Each **turn** represents 1 year of in-world time. Turns are divided into **4 sequential phases** resolved in strict order. All factions resolve the same phase simultaneously before moving to the next.

```
Year N, Phase 1: INCOME
Year N, Phase 2: MAINTENANCE
Year N, Phase 3: ACTION
Year N, Phase 4: NEWS
Year N+1 → Phase 1: INCOME (loop)
```

**Phase 1 — Income**

All factions simultaneously receive FacCreds from their assets:

```
income = sum(asset.income_per_turn for each asset)
```

Asset income is derived from settlement type and territory resources (recalculated each turn). Primal beast alignment also grants income: each beast aligned adds +1d6 FacCred/turn as its territory generates offerings.

If a faction's total income exceeds its **maintenance burden** (see Phase 2), the surplus is added to `faction.fac_creds`. If income is negative (rare — e.g., a beast aligned against a faction), the deficit is subtracted.

**Phase 2 — Maintenance**

All factions simultaneously pay upkeep for their assets:

```
upkeep = sum(asset.upkeep_cost for each asset)
```

Assets that cannot pay upkeep follow this cascade per turn of failure:

| Consecutive Failures | Effect |
|---------------------|--------|
| 1 | Asset takes 1 HP damage |
| 2 | Asset takes 2 HP damage |
| 3+ | Asset destroyed; territory reverts to neutral; remnant placed |

If an asset is destroyed, a **Remnant** is dropped at its location (see D.4.3). The faction's claim on that territory reverts to neutral unless another faction's asset occupies it.

Factions that cannot pay full upkeep for 3 consecutive turns **disband** — all their assets are destroyed, territories go neutral, remaining FacCreds are lost.

**Phase 3 — Action**

Factions take strategic actions. This is the only phase with player input (for the player-controlled faction) and AI decision-making (for all others).

Key SWN rules enforced:

- A faction may perform **one action type** per turn, but may apply it to **multiple targets**
- Each **asset can act only once** per turn (attack, move, or use an ability)
- Only **one asset can be purchased** per turn
- The chosen action type is **committed** — once a faction announces Attack, all its attacks resolve before the phase ends

Action types:

| Action Type | Description | Stat Used |
|-------------|-------------|-----------|
| **Attack** | Conflict against enemy assets at a location | Force |
| **Move** | Relocate assets between settlements/territories | — |
| **Purchase** | Acquire a new asset (1 per turn hard cap) | Wealth |
| **Diplomacy** | Declare/break alliances, propose treaties | Cunning |
| **Expand** | Extend territory into neutral zones | Force |
| **Special** | Context actions (beast binding, relic activation) | Cunning |

**Attack Resolution** (deterministic, no DM needed):

```
attacker_score = attacker_force + sum(asset.force_bonus) + tag_modifiers + terrain_modifier
defender_score = defender_force + sum(asset.defense_bonus) + tag_modifiers + terrain_modifier

roll = 2d6
if attacker_score + roll >= defender_score + 10: attacker deals damage = (attacker_force - defender_force) / 2 + roll
if defender_score + roll >= attacker_score + 10: defender repels; attacker takes 1 HP damage to attacking asset
else: stalemate; both withdraw to adjacent neutral territory
```

Damage is applied to defender's HP first, then attacker. At 0 HP, an asset is destroyed.

**Move Resolution**:

Assets move along connected territory (settlements linked by claimed tiles). Moving through rival territory requires an Attack action first. Asset arrives at destination and can act again if it has not yet acted this turn.

**Expand Resolution**:

A faction can extend its territory by 1 tile per turn per settlement on a frontier. The tile must be adjacent to an existing claim. If a rival faction has a settlement on that tile, Expand fails on that tile. Mountains, ocean, and primal beast territory blocks expansion.

**Diplomacy Resolution**:

- Declare alliance: both factions must agree; creates shared territory access
- Declare hostility: triggers Attack actions at will against that faction next turn
- Propose treaty: Cunning vs Cunning roll; on success, terms accepted

**Phase 4 — News**

All action results from Phase 3 are compiled into a narrative log. Events are broadcast to all factions:

- Conflicts won/lost with casualties
- Assets destroyed and Remnants dropped
- Territory changes
- Primal beast movements and their effects
- Goal progress updates
- AI faction diplomatic proposals

The player faction receives this as a turn-end report. AI factions auto-process this for their next decision cycle.

---

#### 5.2 Faction Asset System

Assets represent military units, buildings, ships, and installations. Each asset has:

```typescript
interface FactionAsset {
  id: string;
  definition_id: string;    // references asset_library entry
  location: string;          // settlement_id or territory_tile_id
  hp: number;
  max_hp: number;
  stealthed: boolean;       // hidden from enemy intel
  can_act: boolean;         // false after asset has acted this turn
  purchased_turn: number;    // for cooldown tracking
}
```

**Asset Categories** (from asset_library):

| Category | Required Stat | Example Types |
|----------|--------------|---------------|
| Force | Force ≥ rating | Militia, Infantry, Cavalry, Siege Engine, Warship |
| Cunning | Cunning ≥ rating | Spy Network, Assassins, Diplomat, Temple |
| Wealth | Wealth ≥ rating | Market, Bank, Trade Post, Harbor, Caravan |

**Asset Limits** — a faction can own up to **N assets per category** where N = that stat rating. Assets beyond the limit cost +1 FacCred maintenance each per turn.

**Upgrades** — assets can be upgraded using XP. Upgrading costs `current_upgrade_cost` FacCreds and increases the asset's HP and abilities. A refitted asset cannot act for 1 turn.

---

#### 5.3 Multi-Turn Campaigns

Certain goals require multiple turns to complete. These suspend normal single-turn action rules:

**Homeworld Transition** — A faction may relocate its capital. During transition (3 turns), the faction cannot take Attack or Expand actions. Movement of other assets is allowed.

**Planet Seizure** — To claim a rival's settlement, a faction must:
1. Attack and reduce the settlement's asset HP to 0 (clearing phase)
2. Hold the settlement for 3 consecutive turns (holding phase)
3. On success, the settlement's asset transfers to the seizing faction

**Primal Beast Binding** — Requires 3+ factions with legendary artifacts to jointly attempt binding. Each faction contributes 1 artifact to the ritual. Binding contested via Cunning vs beast level roll. On success, the beast is Controlled for 10 turns. On failure, the contributed artifacts are destroyed.

---

#### 5.4 Primal Beast Integration

Beasts remain on the world map and continue their passive effects during the faction turn system. In addition:

**Beast Movement** — Each turn, each beast moves up to its speed value in tiles. Movement is deterministic based on world state: Pyraxes seeks volcanic tiles, Lumina seeks deep ocean, etc. Movement does not follow faction territory rules.

**Beast Alignment** — Factions can spend a Diplomacy action to offer tribute to a beast. Tribute costs FacCreds equal to the faction's current income. On acceptance, alignment lasts 3 turns and grants:

| Beast | Alignment Bonus |
|-------|----------------|
| Pyraxes | +2 Force to all Force assets; volcanic tiles in faction territory become mineral-rich |
| Tidarth | +1d6 FacCred/turn; ships in faction territory cannot be attacked by pirates |
| Terros | +1 to all defensive rolls; earthquakes deal -50% damage to faction settlements |
| Lumina | +1d6 FacCred/turn; marine resources × 2 in faction territory; all faction members can breathe underwater |

**Beast Control** — If a faction holds 3+ legendary artifacts, they may attempt to Control a beast (see 5.3). A Controlled beast moves at the faction's direction and its effect zone can be targeted.

**Beast Death** — If a primal beast is killed (extremely rare — requires overwhelming force), the triggering faction absorbs the curse but also inherits the Remnant. See D.4.3 for death consequences.

---

#### 5.5 Victory Conditions

A faction wins when its goal is achieved (see goal types in 5.0). The first faction to achieve its goal triggers an **epoch end** — the turn resolves normally, then:

1. All factions receive the victory notification with the winning faction's name
2. World state is frozen and archived as an epoch snapshot
3. New epoch may begin with world state reset to post-Phase 4 baseline, or the campaign ends

If multiple factions complete goals simultaneously, the faction with the most XP breaks the tie.

**Soft Failure** — A faction reduced to 0 HP is not eliminated but becomes a client state of the faction that dealt the final blow. Client states contribute half their income to their patron and cannot take independent actions.

---

#### 5.6 AI Faction Behavior

AI factions use a simplified goal-seeking algorithm:

```
priority_score = goal_progress_rate / turns_remaining
```

Each turn, each AI faction:
1. Evaluates all valid actions for its current goal
2. Scores each action by `expected_goal_progress_per_turn`
3. Picks the highest-scoring action type (or a random valid action if no goal progress)
4. Commits all available assets to that action type

AI factions do not plan more than 1 turn ahead. This produces emergent, believable behavior without perfect information.

**Difficulty scaling** — AI factions have their stats modified by world age:

| World Age | AI Force | AI Cunning | AI Wealth |
|----------|----------|------------|-----------|
| < 100 years | −2 | +1 | +0 |
| 100–500 years | +0 | +0 | +0 |
| 500–1000 years | +1 | −1 | +0 |
| > 1000 years | +2 | −1 | +1 |

---

#### 5.7 Data Model

New stored files (added to `world_name/world.json`):

```
world/
  factions/
    faction_turn_state.json    # Current turn, phase, action history
    faction_assets.json        # All faction asset instances
    faction_relationships.json # Alliance/hostility state matrix
  campaigns/
    homeworld_transitions.json
    planet_seizures.json
    beast_bindings.json
```

**faction_turn_state.json:**

```json
{
  "turn": 1,
  "phase": "Income",
  "turn_history": [
    { "turn": 1, "phase": "News", "events": [...] }
  ],
  "player_controlled_faction_id": "faction_uuid"
}
```

**faction_relationships.json:**

```json
{
  "relations": {
    "faction_a_id": {
      "faction_b_id": "allied",
      "faction_c_id": "neutral"
    }
  }
}
```

---

#### 5.8 API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/v1/worlds/:id/turn` | Current turn/phase state |
| `POST` | `/api/v1/worlds/:id/turn/action` | Commit a faction action |
| `GET` | `/api/v1/worlds/:id/factions` | All factions with stats |
| `GET` | `/api/v1/worlds/:id/factions/:fid` | Single faction detail |
| `GET` | `/api/v1/worlds/:id/factions/:fid/assets` | Faction assets |
| `POST` | `/api/v1/worlds/:id/factions/:fid/assets` | Purchase asset |
| `GET` | `/api/v1/worlds/:id/factions/:fid/goals` | Current goal and progress |
| `GET` | `/api/v1/worlds/:id/turn/events` | News log for current turn |
| `POST` | `/api/v1/worlds/:id/turn/advance` | Advance to next phase |
| `GET` | `/api/v1/worlds/:id/beasts` | Primal beast locations/status |
| `POST` | `/api/v1/worlds/:id/beasts/:bid/align` | Attempt alignment with beast |

**Action request body:**

```json
POST /api/v1/worlds/:id/turn/action
{
  "action_type": "Attack",
  "actor_asset_ids": ["asset_1", "asset_2"],
  "target_location": "settlement_uuid",
  "target_faction_id": "enemy_faction_uuid"
}
```

---

**Deliverable:** A turn-based faction strategy layer where one faction is player-controlled, AI factions pursue goals autonomously, primal beasts are political actors, and the first faction to achieve its goal triggers an epoch victory.

---

## 9. Configuration Reference

### 9.1 World Generation Config

```yaml
world:
  name: "World Name"
  seed: 12345  # Optional, random if omitted
  
  dimensions:
    width: 64   # Max 64 per requirements
    height: 64  # Max 64 per requirements
    
  time:
    pre_history_years: 500  # 0 for none, configurable
    time_scale: "years"
    
  geography:
    erosion_iterations: 100
    mountain_roughness: 0.5
    sea_level: 0.3  # Percentage of map as ocean
    
  climate:
    axial_tilt: 23.5  # Degrees, earthlike
    disaster_frequency: "medium"  # low, medium, high
    year_length_days: 365
    
  resources:
    richness: "normal"  # poor, normal, rich, abundant
    mineral_spawn_rate: 0.15
    organic_spawn_rate: 0.25
    
  species:
    - type: "human"
      population_factor: 1.0
      expansion_rate: 0.7
```

### 9.2 Species Template Example

```yaml
species:
  name: "Human"
  plural: "Humans"
  
  traits:
    - name: "Adaptable"
      description: "Can settle in varied biomes"
      biome_compatibility: 0.8  # 80% of biomes
    - name: "Curious"
      description: "Higher discovery rates"
      discovery_bonus: 0.15
      
  base_stats:
    reproduction_rate: 0.02
    life_expectancy: 70
    food_requirement: 1.0
    
  society_types:
    - name: "Tribe"
      min_population: 50
      max_population: 500
    - name: "Chiefdom"
      min_population: 500
      max_population: 10000
    - name: "Nation"
      min_population: 10000
```

---

## 10. File Structure

```
world-factory/
├── Cargo.toml
├── Dockerfile              # Multi-stage Docker build
├── docker-compose.yml      # Persistent server deployment
├── .dockerignore
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config/
│   │   └── mod.rs
│   ├── world/
│   │   ├── mod.rs
│   │   ├── generation/
│   │   │   ├── mod.rs
│   │   │   ├── geography.rs    # Voronoi, elevation, plates
│   │   │   ├── hydrology.rs     # Rivers, groundwater
│   │   │   ├── climate.rs       # Weather, biomes
│   │   │   └── resources.rs     # Resource spawning
│   │   ├── entities/
│   │   │   ├── mod.rs
│   │   │   ├── planet.rs
│   │   │   ├── biome.rs
│   │   │   ├── resource.rs
│   │   │   └── polygon.rs
│   │   └── time/
│   │       ├── mod.rs
│   │       ├── events.rs
│   │       ├── figures.rs
│   │       └── disasters.rs
│   ├── history/
│   │   ├── mod.rs
│   │   ├── beings.rs           # Species templates
│   │   ├── societies.rs        # Factions, cultures
│   │   └── civilization.rs    # Settlement, growth
│   ├── persistence/
│   │   ├── mod.rs
│   │   ├── storage.rs
│   │   └── serializer.rs
│   └── api/
│       ├── mod.rs
│       ├── routes.rs
│       └── handlers.rs
├── web/
│   ├── index.html
│   ├── css/
│   │   └── styles.css
│   └── js/
│       ├── app.js
│       ├── map-view.js
│       ├── timeline.js
│       └── dashboard.js
├── tests/
│   ├── api_history_figures_test.rs   # API history + figures endpoint
│   ├── api_world_generation.rs       # API world generation workflow
│   ├── cli_*                         # CLI acceptance tests (Section 14)
│   ├── elevation_assignment_test.rs  # Elevation + biome assignment
│   ├── export_endpoint_test.rs      # Export PNG/JSON endpoints
│   ├── integration_world_generation.rs  # Full world gen pipeline
│   ├── phase1_integration_test.rs   # Phase 1 combined tests
│   ├── phase2_integration_test.rs   # Phase 2 combined tests
│   ├── species_template_test.rs      # Species template loading
│   ├── world_generation_tests.rs     # World generation unit tests
│   └── history_tests.rs
├── examples/
│   └── basic_world.yaml
├── docs/
│   ├── SPEC.md               # Full specification
│   ├── CURRENT_STATUS.md     # Implementation status
│   └── API_CONTRACT.md       # API documentation
└── README.md
```

---

## 11. Algorithm Notes

### 11.1 Voronoi + Lloyd Relaxation

Unlike fractal noise approaches, this graph-based method:
- Produces recognizable regions (not just noise)
- Allows constraint-based elevation (gameplay-directed)
- Rivers flow logically from mountains to coast
- Coastlines are guaranteed to form closed islands

Reference: http://www-cs-students.stanford.edu/~amitp/game-programming/polygon-map-generation/

### 11.2 Tectonic Simulation (Simplified)

- Divide polygons into 5-10 plates randomly
- Identify plate edges (polygons adjacent to different plates)
- Edge polygons get +elevation (mountain building)
- Some edges get volcanic activity
- Interior plates have slower erosion

### 11.3 Erosion Model

Simple fluvial erosion:
- For each river, calculate water volume based on drainage area
- Erode terrain proportional to water volume and gradient
- Apply over multiple iterations for realistic canyon formation

### 11.4 Event Probability

Events trigger based on:
```
P(event) = base_rate * modifier(world_state) * modifier(location)
```

Example: War probability increases with:
- Shared border length
- Resource competition
- Historical grievances (prior wars)
- Population pressure

---

## 12. Future Considerations (Out of Scope for Phase 1-4)

- Multiplayer/world sharing
- Custom being species via UI
- Faction control and turn resolution (Phase 5)
- Save/load simulation state mid-run
- Plugin system for custom event types
- 3D globe visualization
- Sound/ambient audio
- Procedural language generation (Dwarf Fortress style)

---

## 13. Glossary

| Term | Definition |
|------|------------|
| Polygon | A single region in the Voronoi mesh, the fundamental unit of world geography |
| Biome | Regional climate and ecology (desert, forest, tundra, etc.) |
| Tectonic Plate | Large section of crust that moves and interacts with others |
| Being | A sapient species (human, elf, or custom-defined) |
| Society | A group of beings organized under shared governance |
| Faction | A society with goals and resources (SWN-inspired) |
| Artifact | A rare, powerful item that can affect the world |
| Pre-history | The simulated past before "present day" |

---

## 14. CLI Testing Acceptance Criteria

This section defines the acceptance criteria that must be satisfied before any CLI command is considered complete. Tests should live in `tests/cli_*` and be run with `cargo test`.

### 14.1 Flag and Argument Testing

All CLI flags must satisfy these criteria:

| # | Criterion | Description |
|---|-----------|-------------|
| CLI-1 | `--help` works without flags conflicting | No flag may use `-h` as a short flag because clap reserves it for `--help` |
| CLI-2 | `--help` and `--version` are discoverable | Every command must expose these via clap's automatic help generation |
| CLI-3 | Short flags are unique per command | Two flags may not share the same short form (e.g., `-h` for both height and help) |
| CLI-4 | Required vs optional flags are explicit | Flags with defaults must not be marked required |
| CLI-5 | Flag types are validated | `u32` flags reject negative numbers and non-numeric input with a clear error message |

**Test example (CLI-1 — height flag conflict):**
```rust
#[test]
fn test_generate_help_does_not_panic() {
    let result = Command::new("world-factory")
        .args(["generate", "--help"])
        .output();
    // Must not panic; result should be Ok
    assert!(result.is_ok());
    // stdout should contain help text
    let output = result.unwrap();
    assert!(output.status.success());
}
```

### 14.2 Generate Command Testing

| # | Criterion | Description |
|---|-----------|-------------|
| CLI-10 | `generate` completes successfully with default parameters | Exit code 0, no panic |
| CLI-11 | `generate --width 32 --height 32 --seed 42` produces reproducible output | Running twice with same seed produces identical terrain |
| CLI-12 | `generate --width 0` is rejected with a validation error | Width must be > 0 and ≤ 128 |
| CLI-13 | `generate --width 256` is rejected (exceeds maximum) | Width max is 128 |
| CLI-14 | `generate --seed 0` works (seed 0 is valid) | Zero is a valid u64 seed |
| CLI-15 | Unknown flags produce a clear error message | e.g., `generate --unknown` → "unrecognized argument `--unknown`" |
| CLI-16 | Generate output is written to storage, not just stdout | A `.wfw` file exists in `WORLD_FACTORY_DIR/generated/` after completion |

### 14.3 Server Command Testing

| # | Criterion | Description |
|---|-----------|-------------|
| CLI-20 | `--server --port 8080` starts an HTTP server on port 8080 | `curl http://localhost:8080/health` returns 200 |
| CLI-21 | `--server --port 0` picks an available port | The selected port is printed to stdout |
| CLI-22 | `--server` fails gracefully if port is already in use | Clear error message, not a panic |
| CLI-23 | Without `--features api`, `server` mode prints a clear error | "API support not compiled in. Rebuild with --features api" |

### 14.4 Shared Storage Testing

| # | Criterion | Description |
|---|-----------|-------------|
| CLI-30 | `WORLD_FACTORY_DIR` environment variable overrides default storage path | Verify with `std::env::var("WORLD_FACTORY_DIR")` |
| CLI-31 | CLI and server both use `WORLD_FACTORY_DIR` when set | Both read/write to the same directory |
| CLI-32 | Server lists worlds generated by the CLI when using the same `WORLD_FACTORY_DIR` | Integration test: generate CLI world, start server, call `GET /api/v1/worlds` |

### 14.5 Regression Test for the `-h` Flag Bug

The following bug must not recur:

> **Bug:** The `height` parameter used `#[arg(short, long)]` which auto-generated `-h` as the short flag. Clap reserves `-h` for `--help`, causing `debug_assert!` to panic when `--help` was invoked.

**Regression test:**
```rust
#[test]
fn test_height_flag_does_not_conflict_with_help() {
    // This previously panicked in debug builds due to -h conflict
    let result = Command::new("world-factory")
        .args(["generate", "--height", "64", "--help"])
        .output();
    assert!(result.is_ok(), "Should not panic on --help with height flag");
    let output = result.unwrap();
    assert!(output.status.success());
    // Verify help text is present
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--height"));
    assert!(stdout.contains("--help"));
}
```

### 14.6 Running Tests

```bash
# Run all CLI tests
cargo test --test cli_*

# Run specific CLI test
cargo test --test cli_* test_height_flag_does_not_conflict

# Run with api feature for integration tests
cargo test --features api --test cli_*
```

---

## 15. References

1. Amit Patel's Polygon Map Generation: http://www-cs-students.stanford.edu/~amitp/game-programming/polygon-map-generation/
2. Dwarf Fortress World Generation: https://dwarffortresswiki.org/index.php/World_generation
3. Stars Without Number Faction System: https://d0ngiovanni.github.io/swn-faction-spreadsheet/
4. Voronoi Diagrams: https://en.wikipedia.org/wiki/Voronoi_diagram
5. Lloyd's Algorithm: https://en.wikipedia.org/wiki/Lloyd%27s_algorithm
