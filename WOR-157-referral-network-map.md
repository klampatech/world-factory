# Referral Network Map — Founding Engineer Sourcing
**Issue:** WOR-157  
**Owner:** CTO  
**Status:** Draft — 2026-05-06  
**Related:** WOR-2 (founding engineer charter)

---

## 1. Candidate Profile Recap

Target founding engineer must have:
- **Non-negotiable:** Rust experience or demonstrated ability to learn systems languages
- **Critical:** Algorithm-heavy background (Voronoi, spatial computation, simulation)
- **Important:** Full-stack capability, startup experience
- **Compensation:** 0.5–2% equity + competitive cash
- **Start date:** June 2, 2026

---

## 2. Referral Network Tiers

### Tier 1: First-Degree Warm Connections
*People who know us well and have direct access to engineers*

| Node | Relationship | Referral Potential | Notes |
|------|--------------|-------------------|-------|
| [CEO Name] | Co-founder | Primary | Direct network of startup alumni, previous companies |
| [CTO - me] | Co-founder | Primary | Tech community, Rust contributors, algorithm researchers |
| Investors/Advisors | Capital partners | High | Often have founder network access |
| [Angel Investors] | Early backers | High | May know Rust/startup engineers |

**Action Items:**
- [ ] CEO: Export LinkedIn connections list, identify Rust engineers
- [ ] CTO: Pull GitHub followers/contacts who work in Rust
- [ ] Ask advisors directly: "Do you know any Rust engineers looking for early-stage roles?"

---

### Tier 2: Technical Communities
*Spaces where Rust engineers congregate*

| Community | Platform | Engagement Strategy |
|-----------|----------|---------------------|
| Rust Language | zulip.rs/internals forum | Community members, RFC contributors |
| Rust Gaming/Graphics | gamedev.rs, rend3 | Algorithm-heavy, spatial computing |
| Algorithm Competitions | Codeforces, LeetCode | Top performers often Rust users |
| Rust OSS Contributors | GitHub | Identify prolific contributors in relevant domains |
| Rust @Work | Rust forum "jobs" board | Active job seekers |

**Target Profiles to Identify:**
1. Contributors to `geo`, `voronator`, `kdtree`, `ncollide` crates
2. Active Codeforces/LeetCode users with Rust solutions
3. Game engine developers using Rust (amethyst, bevy, rust-gpu)

---

### Tier 3: Academic/Research Pipeline
*Algorithm experts who may be interested in industry*

| Institution/Group | Focus | Outreach Method |
|-------------------|-------|-----------------|
| Computational Geometry labs | Voronoi, spatial algorithms | Cold outreach, professor referrals |
| Graphics research groups | Real-time rendering, simulation | Conference connections |
| Robotics/Autonomy | Pathfinding, optimization | Industry transfer candidates |

**Target Professors/Research Groups:**
- [ ] Identify 5-10 computational geometry researchers
- [ ] Ask about students graduating soon (May/June cohort)

---

### Tier 4: Company Alumni Networks
*Engineers from companies known for algorithm work*

| Company | Why Relevant | Access Method |
|---------|--------------|---------------|
| Figma | Real-time collaboration, spatial ops | Alumni list, LinkedIn |
| Embark Trucks | Rust + autonomous driving | Layoffs, career pages |
| Oxide Computer | Systems/Rust focus | Small team, direct outreach |
| Starflyer (Temporal) | Rust + algorithms | Small network |
| Mozilla ( Servo) | Rust original research | Alumni |

---

## 3. Warm Introduction Paths

### Path A: Through Investors/Advisors
```
Investor → Portfolio founders → Their engineers → Referral
```
**Ask:** "Do you have any portfolio founders who might know Rust engineers?"

### Path B: Through Open Source
```
Maintainer/contributor → OSS community → Their network → Referral
```
**Ask:** "Who else has contributed to this codebase who might be interested in early-stage work?"

### Path C: Through Technical Events
```
Conference/meetup → Personal connection → Follow-up → Referral
```
**Events to target:**
- RustConf 2026 (if scheduled)
- Rust London/Bay Area meetups
- SIGGRAPH (graphics/algorithms)
- GDC (game developers with algorithm focus)

---

## 4. Cold Outreach Targets by Role

### Primary Targets (Warm → Cold Gradient)

| # | Target Type | First Contact | Warm Path |
|---|-------------|---------------|-----------|
| 1 | Portfolio company founders | Investor intro | High warmth |
| 2 | Rust OSS maintainers | GitHub interaction | Medium warmth |
| 3 | Algorithm competition top 100 | Conference/meetup | Medium warmth |
| 4 | Academic researchers | Professor referral | Medium warmth |
| 5 | Company alumni | LinkedIn | Low warmth |

### Company Alumni to Target First

| Company | Rust Signals | Notes |
|---------|--------------|-------|
| Oxide Computer | Core team uses Rust | Small, likely not hiring |
| Aleo | ZK/crypto, Rust | May have overhired |
| Figma | WebGL, collaboration | Recently acquired |
| Embark | Autonomous trucks | Layoffs announced |
| Deterministic | Rust + simulation | Early stage, competitive |

---

## 5. Outreach Sequencing (Week 1-2)

### Week 1: Network Mapping
- [ ] CEO: Export investor/advisor contacts with Rust network signals
- [ ] CTO: Pull Rust OSS contributors from relevant crates
- [ ] Identify 20 warm intro paths
- [ ] Shortlist 10 academic/research contacts

### Week 2: Active Sourcing
- [ ] Send personalized outreach to Tier 1 warm connections
- [ ] Identify 5 candidate targets with clear referral paths
- [ ] Schedule intro calls with promising referrals

### Week 3-4: Pipeline Building
- [ ] Move promising referrals to WOR-4 candidate list
- [ ] Track referral source in candidate tracking (see `hiring-templates/03-candidate-tracking.md`)
- [ ] Document which referral sources yield best candidates

---

## 6. Referral Ask Templates

### For Investors/Advisors
> "We're looking for a founding engineer with strong Rust experience and algorithm background for our core spatial computing work. Do you have any portfolio founders or contacts who might know someone like that? Happy to share the role charter."

### For Technical Community
> "I'm CTO at [Company] — we're building spatial computing infrastructure in Rust. Looking for a founding engineer who enjoys algorithm-heavy work. Know anyone who'd thrive in that environment?"

### For OSS Contributors
> "We admire your work on [crate]. We're using similar techniques for [our problem]. Would you be open to a conversation about what we're building and whether there might be a fit?"

---

## 7. Success Metrics

| Metric | Target | Tracking |
|--------|--------|----------|
| Warm introductions | 10+ by end of Week 2 | Candidate tracking sheet |
| Referral-sourced candidates | 5+ in pipeline | Source field in tracking |
| Referral → Screen conversion | 50%+ | Track conversion rates |
| Referral → Hire | 1 (target) | Source attribution on hire |

---

## 8. Next Steps

- [ ] CEO: Schedule 30-min network audit session
- [ ] CTO: Create Rust OSS contributor shortlist (50 candidates)
- [ ] Draft personalized outreach sequence
- [ ] Set up referral tracking in candidate system
- [ ] Identify 3-5 "unicorn" targets (ideal fits) for concentrated effort

---

*Last updated: 2026-05-06*
