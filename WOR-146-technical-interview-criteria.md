# Technical Interview Criteria & Coding Exercise
**Issue:** WOR-146  
**Owner:** CTO  
**Status:** Draft v1  
**Date:** 2026-05-06

---

## 1. Overview

This document defines the technical interview evaluation criteria and coding exercise for the Founding Engineer role. The assessment is designed to validate:

1. **Rust competency** (non-negotiable priority)
2. **Algorithm & spatial reasoning** (core to our Voronoi/simulation work)
3. **Full-stack execution** (end-to-end problem solving)
4. **Code quality & testing** (production-minded engineering)

---

## 2. Evaluation Rubric

Each category is scored 1–4:
- **4 = Exceptional** — Exceeds expectations, demonstrates mastery
- **3 = Strong** — Meets expectations, solid execution
- **2 = Adequate** — Some gaps, but workable
- **1 = Below Bar** — Significant issues, does not pass

### 2.1 Rust Competency (Weight: 30%)

| Criterion | What We're Looking For |
|-----------|------------------------|
| **Ownership & Borrowing** | Correct usage of lifetimes; no data races; understands ownership transfer |
| **Error Handling** | Uses `Result`/`Option` appropriately; no `.unwrap()` in production code |
| **Async Programming** | Basic async/await; understanding of tokio runtime |
| **Standard Library** | Familiarity with collections, iterators, string handling |
| **Memory Management** | No leaks; appropriate use of `Box`, `Rc`, `Arc` when needed |
| **Testing** | Unit tests with `#[test]`; integration test awareness |

**Red Flags:**
- Excessive `.clone()` without justification
- Panics for error handling (`.unwrap()`, `.expect()`)
- Fighting the borrow checker instead of understanding it
- Unnecessary heap allocations in hot paths

---

### 2.2 Algorithm & Spatial Reasoning (Weight: 25%)

| Criterion | What We're Looking For |
|-----------|------------------------|
| **Data Structure Selection** | Appropriate use of trees, graphs, hashmaps, sets, grids |
| **Computational Complexity** | Awareness of O(n), O(n log n), O(n²) tradeoffs |
| **Geometric Intuition** | Comfort with coordinates, distances, regions, boundaries |
| **Problem Decomposition** | Breaks complex problems into manageable pieces |
| **Edge Cases** | Handles degenerate cases (empty input, extreme values) |

**Relevant Domains for Paperclip:**
- Voronoi diagrams / Delaunay triangulation
- Pathfinding (A*, Dijkstra)
- Spatial partitioning (quadtrees, k-d trees, grid hashing)
- Simulation loops with deterministic updates

---

### 2.3 Full-Stack Execution (Weight: 20%)

| Criterion | What We're Looking For |
|-----------|------------------------|
| **API Design** | Clean REST/HTTP API; sensible endpoints; proper HTTP methods |
| **Data Handling** | Serialization/deserialization; validation; error responses |
| **Frontend Logic** | Clean UI state management; responsive interactions |
| **Integration** | End-to-end functionality from API to UI |

---

### 2.4 Code Quality (Weight: 15%)

| Criterion | What We're Looking For |
|-----------|------------------------|
| **Readability** | Clear naming; logical structure; minimal cognitive load |
| **Modularity** | Functions with single responsibility; DRY principles |
| **Documentation** | Comments where needed; no unnecessary comments |
| **Testing** | Tests cover happy path and edge cases |

---

### 2.5 Problem Solving Under Pressure (Weight: 10%)

| Criterion | What We're Looking For |
|-----------|------------------------|
| **Clarifying Questions** | Asks questions before diving in |
| **Tradeoff Discussion** | Explains time/space/clarity tradeoffs |
| **Adaptability** | Adjusts approach based on feedback |

---

## 3. System Design Interview

### 3.1 Interview Format
- **Duration:** 45 minutes
- **Format:** Structured discussion with live whiteboard collaboration
- **Focus:** Validate ability to design systems relevant to Paperclip's spatial/simulation work

### 3.2 Primary Scenario: Distributed Simulation Coordinator

**The Problem:**
Design a service that coordinates real-time simulations across multiple worker nodes.

**Key Considerations to Explore:**
- How do workers register and advertise their capabilities?
- How does the coordinator balance load across workers?
- How do you handle worker failure mid-simulation?
- What data structures would you use for spatial partitioning of work?
- How would you handle simulation state checkpoints and recovery?

**Evaluation Criteria:**

| Dimension | Score 3 (Pass) | Score 4 (Exceptional) |
|-----------|----------------|----------------------|
| **Architecture** | Clear component separation; appropriate abstractions | Anticipates scaling; considers failure modes |
| **Spatial Awareness** | Mentions spatial partitioning strategies | Proposes quadtree/grid hashing with reasoning |
| **Distributed Systems** | Understands consensus/failure detection basics | Discusses CAP tradeoffs; has opinions on consistency |
| **Communication** | API design is coherent | API is ergonomic; considers client needs |
| **Performance** | Identifies bottlenecks | Quantifies tradeoffs (latency vs throughput) |

### 3.3 Alternative Scenarios (Rotate)

1. **Spatial Index Service** — Design an API for storing/querying 10M+ geometric shapes
2. **Collaborative Editor** — Real-time multi-user document synchronization
3. **Event Sourcing Backend** — Design an event store for simulation snapshots

### 3.4 System Design Red Flags

- No consideration for failure modes
- Ignores spatial partitioning requirements
- Over-engineering for the stated scale
- Unable to defend design choices
- Poor API ergonomics

---

## 4. Pass Bar

| Category | Minimum Score | Notes |
|----------|---------------|-------|
| Rust Competency | **3.0 average** | Non-negotiable |
| Algorithm & Spatial | **2.5 average** | Can improve with mentorship |
| Full-Stack Execution | **2.5 average** | Framework familiarity less important |
| Code Quality | **2.5 average** | Fundamentals matter most |
| Problem Solving | **2.0 average** | Cultural signal |

**Overall:** Must average ≥ **2.7** across all categories with Rust ≥ 3.0

---

## 4. Coding Exercise: Spatial Query Service

### 4.1 Exercise Overview

**Name:** Point-in-Regions Service  
**Time Estimate:** 2–3 hours  
**Purpose:** Validate Rust competency, spatial reasoning, and full-stack capability

### 4.2 Problem Statement

Build a web service that:
1. Accepts a list of axis-aligned rectangular regions (defined by x1, y1, x2, y2)
2. Accepts a list of query points
3. Returns, for each point, which regions contain that point

### 4.3 Requirements

**Backend (Rust with Axum):**
- POST `/regions` — Upload regions (returns region IDs)
- POST `/points` — Query points against stored regions
- GET `/regions/{id}` — Retrieve a single region
- DELETE `/regions/{id}` — Remove a region
- In-memory storage is acceptable (no database required)

**Frontend (React/Next.js or plain HTML/JS):**
- Form to add regions (with visual preview)
- Form to submit query points
- Display results showing which regions contain each point

**Algorithm Requirement:**
- Use an appropriate spatial data structure (quadtree, R-tree, grid) for efficient point-in-region queries
- O(n) brute force is acceptable for ≤100 regions, but structure must support scaling

### 4.4 API Specification

```json
// POST /regions
Request:
{
  "regions": [
    { "x1": 0, "y1": 0, "x2": 10, "y2": 10 },
    { "x1": 5, "y1": 5, "x2": 15, "y2": 15 }
  ]
}
Response:
{
  "regions": [
    { "id": "r1", "x1": 0, "y1": 0, "x2": 10, "y2": 10 },
    { "id": "r2", "x1": 5, "y1": 5, "x2": 15, "y2": 15 }
  ]
}

// POST /points
Request:
{
  "points": [
    { "x": 3, "y": 3 },
    { "x": 7, "y": 7 },
    { "x": 20, "y": 20 }
  ]
}
Response:
{
  "results": [
    { "x": 3, "y": 3, "regions": ["r1"] },
    { "x": 7, "y": 7, "regions": ["r1", "r2"] },
    { "x": 20, "y": 20, "regions": [] }
  ]
}
```

### 4.5 Evaluation Criteria

| Checkpoint | What We'll Review |
|------------|-------------------|
| **Core Functionality** | Regions are stored; queries return correct results |
| **Edge Cases** | Handles empty input, overlapping regions, points on boundaries |
| **Spatial Structure** | Meaningful attempt at spatial indexing (or justification for brute force) |
| **Code Organization** | Clean separation: models, handlers, spatial logic |
| **Error Handling** | Proper validation; meaningful error messages |
| **Testing** | Unit tests for spatial logic; API tests for endpoints |
| **Frontend** | Functional UI; clear results display |

### 4.6 Submission Instructions

1. Create a public GitHub repository
2. Include `README.md` with:
   - How to build and run locally
   - API documentation
   - Brief explanation of spatial data structure choice
3. Open a PR against a `submission` branch (or submit link directly)
4. Include a self-assessment using the rubric above

### 4.7 Live Review (30 minutes)

During the follow-up call, be prepared to discuss:
1. Walk through your spatial data structure choice
2. Explain any tradeoffs you considered
3. What would you change to support 1M regions?
4. How would you handle concurrent writes?

---

## 5. Alternative Coding Exercises

### 5.1 Exercise B: Simulation Loop

For candidates more interested in simulation/game engine work:

**Problem:** Implement a simple cellular automaton with:
- Grid of cells with state (0 or 1)
- Update function with configurable rules (e.g., Conway's Game of Life)
- Web UI to visualize and step through generations
- Export generation history as JSON

**Focus Areas:** Performance, deterministic behavior, clean state management

### 5.2 Exercise C: Pathfinding API

For candidates more interested in game/infrastructure work:

**Problem:** Build a pathfinding service that:
- Stores a navigable graph (nodes + weighted edges)
- Accepts start/end node queries
- Returns optimal path using A* or Dijkstra
- Web UI to visualize graph and highlight path

**Focus Areas:** Graph algorithms, priority queues, API design

---

## 6. Interview Scorecard Template

### Candidate: _________________ Date: _____________

| Category | Score (1-4) | Notes |
|----------|-------------|-------|
| Rust Ownership & Borrowing | | |
| Rust Error Handling | | |
| Rust Async/Stdlib | | |
| Algorithm Correctness | | |
| Algorithm Efficiency | | |
| Spatial Reasoning | | |
| API Design | | |
| Full-Stack Integration | | |
| Code Readability | | |
| Testing | | |
| Problem Solving | | |
| Communication | | |

**Overall Score:** _____ / 4.0  
**Recommendation:** ☐ Strong Yes  ☐ Yes  ☐ No  ☐ Strong No  
**Notes:**

---

## 7. Next Steps

- [x] Draft technical interview criteria (this document)
- [x] System Design Interview section (Section 3) — added 45-min distributed simulation scenario with evaluation rubric
- [ ] CEO review and approval
- [ ] Integrate with WOR-5 scorecard templates
- [ ] Publish final coding exercise to candidate-facing repo

---

## 8. Appendix: Spatial Data Structure Reference

### Grid Hashing
- **Best for:** Uniform distributions; simple boundary queries
- **Complexity:** O(1) average for point lookup
- **Tradeoff:** Poor for sparse or varying-density data

### Quadtree
- **Best for:** 2D spatial partitioning; variable density
- **Complexity:** O(log n) average for insertion/query
- **Tradeoff:** More complex to implement correctly

### R-Tree
- **Best for:** Rectangle queries; GIS applications
- **Complexity:** O(log n) for range queries
- **Tradeoff:** Requires library (e.g., `rstar` crate)

### Brute Force
- **Best for:** ≤100 regions (our threshold)
- **Complexity:** O(n) per query
- **Tradeoff:** Simplicity; acceptable for small scale
