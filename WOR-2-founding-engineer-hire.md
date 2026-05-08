# Founding Engineer — Role Charter & Interview Plan
**Issue:** WOR-2  
**Owner:** CTO  
**Status:** Draft v2 — Updated per CEO feedback (2026-05-05)  
**Date:** 2026-05-05

---

## 1. Role Charter

### Position Summary
The Founding Engineer will be the second technical hire, working directly alongside the founding team to build the core product. This is a high-ownership, high-autonomy role requiring someone comfortable operating in ambiguity while driving execution.

### Responsibilities

**Product Development (60%)**
- Architect and build core product features end-to-end
- Make key technical decisions with appropriate documentation
- Ship production-ready code with minimal supervision

**Technical Leadership (20%)**
- Establish engineering practices and standards
- Mentor future hires as the team grows
- Contribute to product roadmap discussions

**Operations & Culture (20%)**
- Participate in code reviews and design reviews
- Help define our engineering culture and values
- Represent engineering in cross-functional meetings

### Required Experience
- 5+ years of software engineering experience
- **Experience with Rust or demonstrated ability to learn systems languages quickly** (non-negotiable for core work)
- **Comfort with algorithm-heavy code** (our core systems involve Voronoi generation, simulation loops, and geometric computation)
- Full-stack capability (web frontend + backend)
- Track record of taking features from concept to production
- Comfortable with cloud infrastructure (AWS/GCP)

### Preferred Qualifications
- Startup experience (early-stage or growth-stage) — nice-to-have, not required
- Experience with our tech stack (see below)
- Open source contributions
- Experience scaling systems to handle growth

### Tech Stack (to be validated with candidates)
- **Frontend:** React/Next.js or similar
- **Backend:** Rust with Axum (primary) / Go for scripting and integration work
- **Database:** PostgreSQL or similar relational DB
- **Cloud:** AWS or GCP
- **Version Control:** GitHub

### Compensation & Equity
- **Cash:** Competitive with early-stage market (TBD based on location/level)
- **Equity:** 0.5–2% vesting over 4 years with 1-year cliff ✅ (CEO approved)

### Working Style
- Hybrid or remote (with overlap in US Eastern hours)
- Startup tempo — fast iteration, clear ownership
- Collaborative but self-directed

---

## 2. Interview Plan

### Stage 1: Recruiter Screen (30 min)
**Purpose:** Validate basic fit, gauge interest, assess communication

**Questions:**
1. Walk me through your most recent project — what did you build and what was your role?
2. Why are you interested in joining an early-stage company at this stage?
3. What are you looking for in a founding team?

**Red Flags:**
- Vague project descriptions without specifics
- Seeking stability/predictability over impact
- Unable to articulate why this opportunity excites them

**Pass Bar:** Clear communication, genuine interest in early-stage, minimum experience threshold met

---

### Stage 2: Technical Assessment (60 min)
**Purpose:** Validate hands-on technical skills and problem-solving

**Format:** Take-home coding exercise + live review

**Take-home (2–3 hours max):**
- Build a small feature end-to-end (frontend + backend)
- Focus on code quality, not cleverness
- Submit via GitHub PR

**Live Review (30 min):**
- Walk through your solution
- Discuss tradeoffs made
- Q&A on specific decisions

**Red Flags:**
- Unable to explain own code
- Over-engineered without reason
- Poor testing coverage
- Ignoring the simplest solution

**Pass Bar:** Clean, readable code; sensible architecture; can defend decisions

---

### Stage 3: System Design (45 min)
**Purpose:** Assess architectural thinking and scale awareness

**Scenario Options (prioritize algorithm-heavy):**
- **Design a spatial partitioning system for Voronoi generation at scale**
- Architect a real-time simulation loop with deterministic updates
- Design a notification system for 1M users
- Architect a real-time collaboration feature
- Design an API for third-party integrations

**Evaluation Criteria:**
- Can reason about scale and performance
- Considers tradeoffs (consistency vs availability, etc.)
- Asks clarifying questions before diving in
- Communicates clearly under ambiguity

**Pass Bar:** Practical, production-minded thinking; not over-architecting

---

### Stage 4: Culture & Founder Fit (45 min)
**Purpose:** Assess values alignment and working style

**Interviewers:** CEO + 1 additional founder/member

**Conversation Topics:**
1. Tell me about a time you disagreed with a technical decision. How did you handle it?
2. What does "moving fast" mean to you? Where do you draw the line on speed vs. quality?
3. Describe your ideal work environment and management style.
4. What scares you most about joining a company this early?
5. Where do you see yourself in 3 years?

**Values Assessment:**
- **Ownership:** Do they take initiative or wait to be told?
- **Candor:** Can they give and receive honest feedback?
- **Resilience:** How do they handle failure or ambiguity?
- **Curiosity:** Do they want to understand the "why" behind decisions?

**Pass Bar:** Values alignment, authentic conversation, mutual excitement

---

### Stage 5: Reference Checks (2 references)
**Purpose:** Validate track record and behavioral signals

**Questions:**
1. In what context did you work together and for how long?
2. What were their greatest strengths and areas for growth?
3. Tell me about a time they took initiative on a project.
4. How do they handle feedback or criticism?
5. Would you hire them again? Why or why not?

**Pass Bar:** Consistent positive signals across references

---

### Stage 6: Final Decision & Offer

**Decision Criteria (weighted):**
| Criteria | Weight | Notes |
|----------|--------|-------|
| **Rust / systems language competence** | **30%** | Non-negotiable priority |
| **Algorithm & spatial reasoning** | **20%** | Voronoi, pathfinding, simulation |
| Culture & values fit | 20% | |
| Full-stack capability | 15% | |
| Founder rapport | 10% | |
| Reference checks | 5% | |

**Offer Process:**
1. Internal debrief within 48 hours of final interview
2. Consensus decision from interviewers
3. Offer extended within 1 week of final stage
4. **Target start date: June 2, 2026**

---

## 3. Timeline

| Week | Milestone |
|------|-----------|
| Week 1 | Finalize charter, post job, begin outreach |
| Week 2–3 | Recruiter screens |
| Week 3–4 | Technical assessments |
| Week 4–5 | System design + Culture rounds |
| Week 5–6 | Reference checks + offer |

---

## 4. Next Steps

- [ ] Review and approve this charter with CEO
- [ ] Post job description (link to careers page)
- [ ] Set up interview scorecard template
- [ ] Identify first 10 target candidates for outreach

---

## 4. CEO Decisions (2026-05-05)

| Item | Decision |
|------|----------|
| Equity range (0.5–2%) | ✅ Approved |
| Tech stack alignment | ✅ Revised to emphasize Rust |
| Target start date | June 2, 2026 |
| Technical priority | Rust > systems design > full-stack > database |

## 5. Next Steps

- [x] Review and approve this charter with CEO
- [ ] Post job description (link to careers page) — WOR-3
- [ ] Set up interview scorecard template — WOR-5
- [ ] Identify first 10 target candidates for outreach — WOR-4
