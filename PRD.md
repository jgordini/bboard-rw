# Product Requirements Document (PRD)
## UAB IT Idea Board

**Version:** 2.0  
**Date:** 2024  
**Status:** Production  
**Owner:** UAB IT

---

## 1. Executive Summary

The UAB IT Idea Board is an anonymous idea submission and voting platform designed to facilitate open communication and idea sharing within the UAB IT community. The application allows users to submit ideas anonymously, vote on submissions they find valuable, and enables administrators to moderate content. Built with a focus on simplicity, security, and real-time collaboration, the platform encourages engagement without requiring user authentication.

### 1.1 Product Vision
To create an accessible, anonymous platform where UAB IT community members can freely share ideas, provide feedback, and collectively identify the most valuable improvements and solutions.

### 1.2 Key Success Metrics
- Number of ideas submitted per month
- Average votes per idea
- User engagement rate (submissions + votes)
- Time to first submission
- Admin moderation efficiency

---

## 2. Product Overview

### 2.1 Problem Statement
UAB IT needs a simple, frictionless way for community members to:
- Share improvement ideas and suggestions
- Vote on ideas they support
- Identify the most popular/valuable ideas through democratic voting
- Maintain anonymity to encourage honest feedback

### 2.2 Target Users

**Primary Users:**
- **Anonymous Contributors**: UAB IT community members who want to submit ideas and vote on others' ideas
  - No authentication required
  - Browser-based access
  - Mobile and desktop users

**Secondary Users:**
- **Administrators**: UAB IT staff who moderate content and manage the platform
  - Password-protected admin panel
  - Content moderation capabilities
  - Analytics and statistics

### 2.3 Use Cases

**Use Case 1: Submit an Idea**
1. User visits the Idea Board
2. User types their idea in the submission form (max 500 characters)
3. User completes CAPTCHA verification
4. User clicks "Post"
5. System validates content (profanity filter, length)
6. Idea is submitted and appears in the list
7. User sees success confirmation

**Use Case 2: Vote on an Idea**
1. User browses the list of ideas (sorted by vote count)
2. User clicks the upvote button on an idea they support
3. System checks if user has already voted (localStorage + database)
4. Vote is recorded
5. Vote count increments and idea re-sorts in real-time
6. Button changes to "voted" state (disabled, green)

**Use Case 3: Moderate Content (Admin)**
1. Admin navigates to admin panel
2. Admin enters password
3. Admin views all submissions with statistics
4. Admin can delete individual ideas or bulk delete
5. Admin can view total ideas and votes

---

## 3. Functional Requirements

### 3.1 Core Features

#### 3.1.1 Idea Submission
- **FR-001**: Users must be able to submit ideas without authentication
- **FR-002**: Submission form must include:
  - Text area (max 500 characters)
  - Character counter (0/500)
  - CAPTCHA verification
  - Submit button
- **FR-003**: Character counter must:
  - Update in real-time as user types
  - Show warning state at 90% capacity (450/500)
  - Show error state at 100% capacity (500/500)
- **FR-004**: System must validate submissions:
  - Content is not empty
  - Content does not exceed 500 characters
  - Content passes profanity filter
  - CAPTCHA token is valid
- **FR-005**: On successful submission:
  - Form clears
  - CAPTCHA resets
  - Success message displays
  - Idea appears in list (via real-time update)
- **FR-006**: On failed submission:
  - Error message displays
  - Form retains user input
  - CAPTCHA resets

#### 3.1.2 Voting System
- **FR-007**: Users must be able to upvote ideas
- **FR-008**: Each user can vote only once per idea
- **FR-009**: Vote prevention must use:
  - Browser fingerprint (localStorage voter ID)
  - Database constraint (unique voter_fingerprint + idea_id)
- **FR-010**: Vote button must:
  - Show as outline circle when unvoted
  - Show as filled green circle when voted
  - Disable after voting
  - Display vote count below button
- **FR-011**: Vote count must:
  - Increment immediately (optimistic update)
  - Sync with database
  - Update in real-time across all users
- **FR-012**: Ideas must sort by vote count (descending)
- **FR-013**: When vote count changes, ideas must re-sort with smooth animation

#### 3.1.3 Idea Display
- **FR-014**: Ideas must display in a card layout with:
  - Upvote button and count (left side)
  - Idea content (center)
  - Relative timestamp (e.g., "2 hours ago")
- **FR-015**: Ideas must be sorted by vote count (highest first)
- **FR-016**: Cards must have hover effects
- **FR-017**: Empty state must display when no ideas exist
- **FR-018**: Loading state must display during initial load

#### 3.1.4 Real-time Updates
- **FR-019**: New ideas must appear automatically without page refresh
- **FR-020**: Real-time updates must use WebSocket or similar real-time technology
- **FR-021**: When new idea is inserted, list must reload to show correct sort order

#### 3.1.5 Content Moderation
- **FR-022**: Profanity filter must block:
  - The 7 dirty words and variations
  - Common racial slurs
  - Obfuscated versions (using special characters)
- **FR-023**: Filter must check content before submission
- **FR-024**: Filter must show error message if profanity detected

#### 3.1.6 Security
- **FR-025**: CAPTCHA must be required for all submissions
- **FR-026**: CAPTCHA service must be configurable
- **FR-027**: All database access must be secured with appropriate access controls
- **FR-028**: Client-side code must only use public API keys
- **FR-029**: No sensitive data must be stored in localStorage

### 3.2 Admin Features

#### 3.2.1 Authentication
- **FR-030**: Admin panel must require password authentication
- **FR-031**: Password must be configurable
- **FR-032**: Session must persist in browser (localStorage)
- **FR-033**: Logout must clear session

#### 3.2.2 Dashboard
- **FR-034**: Admin panel must display:
  - Total ideas count
  - Total votes count
  - Logout button
- **FR-035**: Statistics must update when content changes

#### 3.2.3 Content Management
- **FR-036**: Admin must be able to view all ideas with:
  - Full content
  - Vote count
  - Timestamp
- **FR-037**: Admin must be able to delete individual ideas
- **FR-038**: Admin must be able to delete all ideas (with confirmation)
- **FR-039**: Admin must be able to delete ideas older than 30 days
- **FR-040**: Bulk actions must require double confirmation for destructive operations

#### 3.2.4 List Management
- **FR-041**: Ideas list must display in reverse chronological order (newest first)
- **FR-042**: Each idea must have a delete button
- **FR-043**: Refresh button must reload all ideas

### 3.3 User Experience

#### 3.3.1 Responsive Design
- **FR-044**: Application must work on:
  - Desktop (1920px+)
  - Tablet (768px - 1919px)
  - Mobile (320px - 767px)
- **FR-045**: Layout must adapt to screen size
- **FR-046**: Touch targets must be at least 44x44px on mobile

#### 3.3.2 UAB Branding
- **FR-047**: Application must use UAB brand colors:
  - UAB Green (#1A5632)
  - Dragons Lair Green (#033319)
  - Campus Green (#90D408)
  - Loyal Yellow (#FDB913)
  - Smoke Gray (#808285)
- **FR-048**: UAB logo must display in header
- **FR-049**: Typography must use UAB fonts (Kulturista, Aktiv Grotesk) with fallbacks
- **FR-050**: Design must match UAB design system

#### 3.3.3 Performance
- **FR-051**: Initial page load must be under 2 seconds
- **FR-052**: Vote submission must provide immediate feedback (< 100ms)
- **FR-053**: Real-time updates must appear within 1 second
- **FR-054**: Re-sort animations must be smooth (60fps)

#### 3.3.4 Accessibility
- **FR-055**: All interactive elements must have ARIA labels
- **FR-056**: Color contrast must meet WCAG AA standards
- **FR-057**: Keyboard navigation must be supported
- **FR-058**: Screen reader compatibility required

---

## 4. Non-Functional Requirements

### 4.1 Technical Requirements

#### 4.1.1 Technology Stack
- **NFR-001**: Frontend: Vanilla JavaScript (no frameworks)
- **NFR-002**: Styling: CSS framework (CDN-based)
- **NFR-003**: Database: Relational database with real-time capabilities
- **NFR-004**: Icons: Icon library (CDN)
- **NFR-005**: CAPTCHA: Third-party CAPTCHA service
- **NFR-006**: Hosting: Static file hosting

#### 4.1.2 Browser Support
- **NFR-007**: Chrome/Edge (latest 2 versions)
- **NFR-008**: Firefox (latest 2 versions)
- **NFR-009**: Safari (latest 2 versions)
- **NFR-010**: Mobile browsers (iOS Safari, Chrome Mobile)

#### 4.1.3 Database Schema
- **NFR-011**: `ideas` table:
  - `id` (Unique identifier, primary key)
  - `content` (TEXT, required, max 500 characters)
  - `created_at` (Timestamp, default current time)
  - `vote_count` (INTEGER, default 0)
- **NFR-012**: `votes` table:
  - `id` (Unique identifier, primary key)
  - `idea_id` (Foreign key to ideas, cascade delete)
  - `voter_fingerprint` (TEXT, required)
  - `created_at` (Timestamp, default current time)
  - Unique constraint on (idea_id, voter_fingerprint)
- **NFR-013**: Database trigger or application logic must auto-increment vote_count on vote insert
- **NFR-014**: Indexes must exist on:
  - `ideas.vote_count` (for sorting)
  - `votes.idea_id` (for lookups)

#### 4.1.4 Security
- **NFR-015**: Database access must be controlled via access policies or application-level security
- **NFR-016**: Access policies must allow:
  - Public SELECT on ideas
  - Public INSERT on ideas
  - Public SELECT on votes (for duplicate checking)
  - Public INSERT on votes
- **NFR-017**: No authentication required for public access
- **NFR-018**: Admin password must be stored securely (not in plain text)
- **NFR-019**: All user input must be sanitized (XSS prevention)
- **NFR-020**: Rate limiting should be considered for production

### 4.2 Performance Requirements
- **NFR-021**: Page load time: < 2 seconds
- **NFR-022**: Time to interactive: < 3 seconds
- **NFR-023**: Vote submission latency: < 500ms
- **NFR-024**: Real-time update latency: < 1 second
- **NFR-025**: Support at least 1000 concurrent users

### 4.3 Reliability Requirements
- **NFR-026**: Uptime: 99.5% availability
- **NFR-027**: Graceful error handling for all API calls
- **NFR-028**: Offline detection and user notification
- **NFR-029**: Database connection retry logic

### 4.4 Maintainability Requirements
- **NFR-030**: Code must be well-commented
- **NFR-031**: Functions must be modular and reusable
- **NFR-032**: Configuration must be centralized
- **NFR-033**: No external build process required

---

## 5. User Stories

### 5.1 Anonymous User Stories

**US-001**: As an anonymous user, I want to submit an idea without creating an account, so that I can quickly share my thoughts.

**US-002**: As an anonymous user, I want to see all ideas sorted by popularity, so that I can quickly identify the most valuable suggestions.

**US-003**: As an anonymous user, I want to vote on ideas I support, so that I can help prioritize valuable suggestions.

**US-004**: As an anonymous user, I want to see new ideas appear automatically, so that I don't need to refresh the page.

**US-005**: As an anonymous user, I want to see when I've already voted on an idea, so that I don't accidentally vote twice.

**US-006**: As an anonymous user, I want the platform to work on my mobile device, so that I can participate from anywhere.

### 5.2 Admin User Stories

**US-007**: As an admin, I want to view all submitted ideas, so that I can monitor community engagement.

**US-008**: As an admin, I want to delete inappropriate content, so that I can maintain a professional environment.

**US-009**: As an admin, I want to see statistics about submissions and votes, so that I can understand platform usage.

**US-010**: As an admin, I want to bulk delete old ideas, so that I can manage database size.

---

## 6. Data Model

### 6.1 Ideas Table
```
ideas
├── id: Unique Identifier (Primary Key)
├── content: TEXT (Required, Max 500 chars)
├── created_at: Timestamp (Default: Current Time)
└── vote_count: INTEGER (Default: 0)
```

### 6.2 Votes Table
```
votes
├── id: Unique Identifier (Primary Key)
├── idea_id: Foreign Key → ideas.id (CASCADE DELETE)
├── voter_fingerprint: TEXT (Required)
├── created_at: Timestamp (Default: Current Time)
└── UNIQUE(idea_id, voter_fingerprint)
```

### 6.3 LocalStorage
```
voter_id: string (Unique voter identifier)
voted_ideas: JSON array (List of voted idea IDs)
```

---

## 7. API Specifications

### 7.1 Backend API Requirements
- **API-001**: RESTful or GraphQL API for database operations
- **API-002**: Real-time subscription mechanism (WebSocket, Server-Sent Events, or polling)
- **API-003**: Public API endpoints for:
  - GET ideas (list all, sorted by vote_count)
  - POST ideas (create new)
  - POST votes (create new)
  - GET votes (check for duplicates)
- **API-004**: Admin API endpoints (password-protected):
  - GET ideas (all with metadata)
  - DELETE ideas (single or bulk)
  - GET statistics

### 7.2 Required Operations

**Insert Idea**
- Method: POST
- Endpoint: `/api/ideas`
- Body: `{ content: string }`
- Response: Created idea object

**Select Ideas**
- Method: GET
- Endpoint: `/api/ideas`
- Query: `?sort=vote_count&order=desc`
- Response: Array of idea objects

**Insert Vote**
- Method: POST
- Endpoint: `/api/votes`
- Body: `{ idea_id: string, voter_fingerprint: string }`
- Response: Created vote object

**Delete Idea (Admin)**
- Method: DELETE
- Endpoint: `/api/ideas/:id`
- Auth: Admin password required
- Response: Success confirmation

### 7.3 Real-time Requirements
- **API-005**: Real-time mechanism must notify clients of:
  - New idea insertions
  - Vote count changes
- **API-006**: Real-time updates must be efficient (low latency, minimal bandwidth)

---

## 8. User Interface Specifications

### 8.1 Main Page Layout

```
┌─────────────────────────────────────┐
│  UAB Logo Header (Green)            │
├─────────────────────────────────────┤
│  Title: "UAB IT Idea Board"         │
│  Subtitle: "Share improvements..."  │
├─────────────────────────────────────┤
│  Submission Form                     │
│  ┌───────────────────────────────┐ │
│  │ Textarea (500 char max)       │ │
│  │ CAPTCHA Widget                │ │
│  │ [0/500]          [Post Button]│ │
│  └───────────────────────────────┘ │
├─────────────────────────────────────┤
│  Ideas List (sorted by votes)       │
│  ┌───┬───────────────────────────┐ │
│  │ ↑ │ Idea content text...      │ │
│  │ 5 │ 2 hours ago               │ │
│  └───┴───────────────────────────┘ │
│  ┌───┬───────────────────────────┐ │
│  │ ↑ │ Another idea...           │ │
│  │ 3 │ 1 day ago                 │ │
│  └───┴───────────────────────────┘ │
└─────────────────────────────────────┘
```

### 8.2 Admin Panel Layout

```
┌─────────────────────────────────────┐
│  Admin Panel Header                 │
├─────────────────────────────────────┤
│  Stats Cards                        │
│  [Total Ideas] [Total Votes] [Logout]│
├─────────────────────────────────────┤
│  Bulk Actions                       │
│  [Delete All] [Delete Old] [Refresh]│
├─────────────────────────────────────┤
│  All Submissions List               │
│  ┌───────────────────────────────┐ │
│  │ Idea content... [Delete]      │ │
│  │ 5 votes | 2024-01-15 10:30   │ │
│  └───────────────────────────────┘ │
└─────────────────────────────────────┘
```

### 8.3 Color Palette
- **UAB Green**: #1A5632 (Primary actions, headers)
- **Dragons Lair Green**: #033319 (Hover states)
- **Campus Green**: #90D408 (Votes, success states)
- **Loyal Yellow**: #FDB913 (Warnings)
- **Smoke Gray**: #808285 (Text, borders)
- **Smoke Gray 7**: #F5F5F5 (Backgrounds)
- **Smoke Gray 15**: #DADADA (Borders)

### 8.4 Typography
- **Headings**: Kulturista (serif), fallback to Georgia
- **Body**: Aktiv Grotesk, fallback to system sans-serif
- **Base Size**: 18px
- **Small Text**: 14px
- **Tiny Text**: 12px

---

## 9. Security Requirements

### 9.1 Content Security
- **SEC-001**: All user input must be sanitized before display
- **SEC-002**: XSS prevention via HTML escaping
- **SEC-003**: Profanity filtering on submission
- **SEC-004**: CAPTCHA verification required

### 9.2 Data Security
- **SEC-005**: No personal information collected
- **SEC-006**: Voter fingerprints stored in localStorage (not personally identifiable)
- **SEC-007**: Database access controlled via access policies or application security
- **SEC-008**: Admin password stored securely (hashed or in secure environment variable)

### 9.3 Access Control
- **SEC-009**: Public read access to ideas
- **SEC-010**: Public write access to ideas and votes
- **SEC-011**: Admin panel password-protected
- **SEC-012**: No user authentication system

---

## 10. Deployment Requirements

### 10.1 Hosting
- **DEP-001**: Static file hosting required for frontend
- **DEP-002**: Backend API hosting required
- **DEP-003**: CDN support recommended
- **DEP-004**: HTTPS required

### 10.2 Configuration
- **DEP-005**: API endpoint URLs must be configurable
- **DEP-006**: CAPTCHA service credentials must be configurable
- **DEP-007**: Admin password must be configurable

### 10.3 Database Setup
- **DEP-008**: Database must be provisioned
- **DEP-009**: Database tables must be created
- **DEP-010**: Access policies or security rules must be configured
- **DEP-011**: Real-time capabilities must be enabled
- **DEP-012**: Database trigger or application logic must be set up for vote counting

---

## 11. Testing Requirements

### 11.1 Functional Testing
- **TEST-001**: Test idea submission with valid content
- **TEST-002**: Test idea submission with profanity (should fail)
- **TEST-003**: Test idea submission without CAPTCHA (should fail)
- **TEST-004**: Test voting on an idea
- **TEST-005**: Test duplicate vote prevention
- **TEST-006**: Test real-time updates
- **TEST-007**: Test admin login and logout
- **TEST-008**: Test admin delete operations

### 11.2 Browser Testing
- **TEST-009**: Test on Chrome, Firefox, Safari
- **TEST-010**: Test on mobile browsers (iOS, Android)
- **TEST-011**: Test responsive layout at various screen sizes

### 11.3 Performance Testing
- **TEST-012**: Test page load time
- **TEST-013**: Test vote submission latency
- **TEST-014**: Test with 100+ ideas in database

### 11.4 Security Testing
- **TEST-015**: Test XSS prevention
- **TEST-016**: Test SQL injection prevention
- **TEST-017**: Test access control enforcement

---

## 12. Future Enhancements (Out of Scope)

### 12.1 Potential Features
- User authentication and profiles
- Idea categories/tags
- Comments/replies on ideas
- Idea status tracking (under review, implemented, etc.)
- Email notifications
- Export functionality (CSV, PDF)
- Advanced analytics dashboard
- Idea search and filtering
- Rich text formatting
- Image attachments
- Downvoting capability
- Idea editing
- Moderation queue
- Spam detection algorithms

### 12.2 Technical Improvements
- Progressive Web App (PWA) support
- Offline mode with sync
- Service worker for caching
- Unit and integration tests
- CI/CD pipeline
- Error logging service
- Performance monitoring
- A/B testing framework

---

## 13. Success Criteria

### 13.1 Launch Criteria
- ✅ All core features implemented and tested
- ✅ UAB branding applied correctly
- ✅ Responsive design verified on all target devices
- ✅ Security measures in place
- ✅ Admin panel functional
- ✅ Documentation complete

### 13.2 Post-Launch Metrics
- Number of ideas submitted in first month
- Average votes per idea
- User retention rate
- Admin moderation frequency
- Error rate and uptime

---

## 14. Dependencies

### 14.1 External Services
- **CAPTCHA Service**: Third-party CAPTCHA provider
- **CSS Framework CDN**: Styling framework
- **Icon Library CDN**: Icon library
- **Database Service**: Backend database with real-time capabilities
- **API Hosting**: Backend API hosting service

### 14.2 Internal Dependencies
- UAB brand guidelines
- UAB logo assets

---

## 15. Risks and Mitigations

### 15.1 Technical Risks
- **Risk**: Database service outage
  - **Mitigation**: Error handling, user notifications, fallback messaging

- **Risk**: CAPTCHA service failure
  - **Mitigation**: Graceful degradation, allow submission if widget fails to load

- **Risk**: Spam submissions
  - **Mitigation**: CAPTCHA, profanity filter, admin moderation, potential rate limiting

### 15.2 Security Risks
- **Risk**: XSS attacks
  - **Mitigation**: HTML escaping, input sanitization

- **Risk**: Database abuse
  - **Mitigation**: Access policies, rate limiting (future), admin monitoring

### 15.3 Operational Risks
- **Risk**: Inappropriate content
  - **Mitigation**: Profanity filter, admin moderation tools

- **Risk**: High database growth
  - **Mitigation**: Bulk delete tools, 30-day cleanup feature

---

## 16. Glossary

- **Voter Fingerprint**: Unique identifier stored in localStorage to prevent duplicate votes
- **Optimistic Update**: UI update before server confirmation
- **FLIP Animation**: First, Last, Invert, Play - animation technique for smooth reordering
- **CAPTCHA**: Completely Automated Public Turing test to tell Computers and Humans Apart
- **Access Policies**: Security rules that control database access

---

## 17. Appendices

### 17.1 File Structure
```
redclone/
├── index.html              # Main application page
├── admin.html              # Admin panel
├── css/
│   └── style.css          # Custom styles
├── js/
│   ├── config.js          # API configuration
│   ├── ui.js              # UI rendering functions
│   └── app.js             # Main application logic
├── uablogo.svg            # UAB logo
├── README.md              # Setup and deployment guide
└── PRD.md                 # This document
```

### 17.2 Configuration Checklist
- [ ] Database provisioned
- [ ] Database tables created
- [ ] Access policies or security rules configured
- [ ] Real-time capabilities enabled
- [ ] Database trigger or application logic created for vote counting
- [ ] CAPTCHA service configured
- [ ] CAPTCHA credentials configured in application
- [ ] API endpoints configured
- [ ] Admin password set securely
- [ ] Application deployed to hosting service

---

**Document End**
