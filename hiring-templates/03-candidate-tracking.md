# Candidate Tracking System

## Overview

This document provides a simple tracking system for managing candidates through the interview pipeline.

---

## Candidate Tracking Table (Markdown)

Use this template for each candidate:

```markdown
## [Candidate Name]

| Field | Value |
|-------|-------|
| Position | [Job Title] |
| Applied Date | [Date] |
| Source | [LinkedIn/Referral/Job Board/Other] |
| Current Stage | [Applied → Phone Screen → Technical → Final → Offer → Hired/Rejected] |
| Stage Updated | [Date] |
| Recruiter | [Name] |
| Assigned Interviewers | [Names] |

### Interview Schedule
| Stage | Interviewer | Date/Time | Status |
|-------|-------------|-----------|--------|
| Phone Screen | [Name] | [DateTime] | [Scheduled/Completed/No Show] |
| Technical | [Name] | [DateTime] | [Scheduled/Completed/No Show] |
| Final Round | [Name(s)] | [DateTime] | [Scheduled/Completed/No Show] |

### Scorecards
| Stage | Score (1-5) | Notes |
|-------|-------------|-------|
| Phone Screen | [#] | [Summary] |
| Technical | [#] | [Summary] |
| Final Round | [#] | [Summary] |

### Communication Log
| Date | Type | Summary |
|------|------|---------|
| [Date] | [Email/Call/Note] | [Brief summary] |

### Decision
| Field | Value |
|-------|-------|
| Outcome | [Hired / Rejected / Withdrew] |
| Decision Date | [Date] |
| Reason | [Brief explanation] |
| Salary Offered | [$ if applicable] |
| Feedback Sent | [Yes/No/Date] |
```

---

## Stage Workflow

```
┌─────────┐    ┌─────────────┐    ┌───────────┐    ┌─────────┐    ┌────────┐
│ APPLIED │───▶│ PHONE SCREEN│───▶│ TECHNICAL │───▶│  FINAL  │───▶│ OFFER  │
└─────────┘    └─────────────┘    └───────────┘    └─────────┘    └────────┘
     │                │                  │             │              │
     ▼                ▼                  ▼             ▼              ▼
  REJECTED        REJECTED          REJECTED       REJECTED       HIRED
                                  (post-tech)    (post-final)   (accepted)
                                                        │
                                                        ▼
                                                    DECLINED
                                                    (candidate)
```

### Stage Definitions

| Stage | Description | Typical Duration | Owner |
|-------|-------------|------------------|-------|
| Applied | Resume/application received | - | Recruiter |
| Phone Screen | Initial 30-min screening call | 1-3 days after apply | Recruiter/Hiring Manager |
| Technical | Skills assessment / coding | 1-2 weeks after phone | Engineer(s) |
| Final Round | Culture fit / leadership | 1-2 weeks after technical | Senior leadership |
| Offer | Formal offer extended | Target 1 week | Recruiter/HM |

### Decision Criteria by Stage

| Stage | Advance Criteria | Reject Criteria |
|-------|------------------|-----------------|
| Phone Screen | Communication skills, basic role fit, availability | Poor communication, clearly underqualified, compensation mismatch |
| Technical | Solves problems adequately, writes clean code, explains thinking | Can't complete basics, poor problem-solving, culture red flags |
| Final | Aligns with values, strong collaboration, growth mindset | Misalignment with values, red flags in judgment |

---

## Weekly Pipeline Report

```markdown
## Week of [Date]

### Active Pipeline
| Stage | Count | Avg Time in Stage |
|-------|-------|-------------------|
| Phone Screen | [#] | [# days] |
| Technical | [#] | [# days] |
| Final Round | [#] | [# days] |
| Offer | [#] | [# days] |

### This Week's Activity
- Interviews Scheduled: [#]
- Interviews Completed: [#]
- Offers Extended: [#]
- Offers Accepted: [#]
- Rejections Sent: [#]

### Bottlenecks / Blockers
- [Issue if any]

### Follow-up Actions
- [ ] [Action item 1]
- [ ] [Action item 2]
```

---

## Setup Instructions

1. **Spreadsheet Setup**: Copy the table template into a spreadsheet (Google Sheets, Excel, Notion, Airtable)
2. **Columns for Board View**: Name, Position, Stage, Applied Date, Last Activity, Next Step
3. **Color Coding by Stage**:
   - 🟢 Green: Active in stage (last activity < 3 days)
   - 🟡 Yellow: Needs attention (no activity 3-7 days)
   - 🔴 Red: Stalled (> 7 days without movement)

4. **Automated Reminders**: Set calendar reminders for:
   - 1 week after phone screen without technical scheduled
   - 2 days before any scheduled interview
   - 24hr after interview to complete scorecard
   - 1 week after final round for decision
