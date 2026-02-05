# User Authentication System Design

**Date:** 2026-02-05
**Status:** Approved
**Migration Strategy:** Fresh start (delete existing data)

## Overview

Transform the UAB IT Idea Board from an anonymous submission system to a full-featured authenticated platform with user accounts, role-based permissions, moderation tools, and flagging capabilities. This design implements the database schema specified in `database-structure.md` with local email/password authentication (SSO deferred to future phase).

## Key Decisions

1. **Fresh Start**: Delete all existing anonymous data - clean migration
2. **Local Auth First**: Build email/password authentication, defer UAB CAS SSO integration
3. **Auto-Publish Model**: Ideas are public immediately, moderators moderate after-the-fact
4. **Login Required**: All voting and commenting requires user authentication
5. **Edit-Only Permissions**: Users can edit but not delete their own content
6. **Keep Profanity Filter**: First line of defense despite moderation system
7. **Fixed Stage Lifecycle**: "Ideate" → "Review" → "In Progress" → "Completed"
8. **Simple Flagging**: Users flag content without required reasons
9. **Soft Moderation**: `is_off_topic` hides content but keeps it for moderator review
10. **Deferred Email**: Create table structure but don't implement SMTP yet

## Database Schema Changes

### New Tables

#### users
Replaces the `admin_users` table entirely.
```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(200) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role SMALLINT NOT NULL DEFAULT 0,  -- 0: User, 1: Moderator, 2: Admin
    created_on TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### flags
Tracks user reports of inappropriate content.
```sql
CREATE TABLE flags (
    id SERIAL PRIMARY KEY,
    target_type VARCHAR(20) NOT NULL,  -- 'idea' or 'comment'
    target_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, target_type, target_id)  -- One flag per user per item
);

CREATE INDEX idx_flags_target ON flags(target_type, target_id);
CREATE INDEX idx_flags_user_id ON flags(user_id);
```

#### email_notifications
Infrastructure for future email notifications (no SMTP implementation yet).
```sql
CREATE TABLE email_notifications (
    id SERIAL PRIMARY KEY,
    recipient_email VARCHAR(200) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 0,  -- 0: Pending, 1: Sent, 2: Failed
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_email_notifications_status ON email_notifications(status);
```

### Modified Tables

#### ideas
```sql
ALTER TABLE ideas
    ADD COLUMN user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ADD COLUMN stage VARCHAR(50) NOT NULL DEFAULT 'Ideate',
    ADD COLUMN is_public BOOLEAN NOT NULL DEFAULT true,
    ADD COLUMN is_off_topic BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN pinned_at TIMESTAMPTZ NULL,
    ALTER COLUMN title TYPE VARCHAR(100);

CREATE INDEX idx_ideas_user_id ON ideas(user_id);
CREATE INDEX idx_ideas_stage ON ideas(stage);
CREATE INDEX idx_ideas_is_public ON ideas(is_public);
CREATE INDEX idx_ideas_pinned_at ON ideas(pinned_at DESC NULLS LAST);
```

**Stage Values (Fixed):**
- `Ideate` - Default for new ideas
- `Review` - Under moderator review
- `In Progress` - Being implemented
- `Completed` - Finished implementation

#### votes
```sql
ALTER TABLE votes
    DROP COLUMN voter_fingerprint,
    ADD COLUMN user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE;

-- Update unique constraint
DROP INDEX IF EXISTS votes_idea_id_voter_fingerprint_key;
ALTER TABLE votes ADD CONSTRAINT votes_user_id_idea_id_unique UNIQUE(user_id, idea_id);

CREATE INDEX idx_votes_user_id ON votes(user_id);
```

#### comments
```sql
ALTER TABLE comments
    ADD COLUMN user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ADD COLUMN is_pinned BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN is_deleted BOOLEAN NOT NULL DEFAULT false;

CREATE INDEX idx_comments_user_id ON comments(user_id);
CREATE INDEX idx_comments_is_pinned ON comments(is_pinned);
```

### Removed Tables
- `admin_users` - Replaced by `users` table with role field

### Migration Strategy
```sql
-- Order of operations in migration file:
1. Create users table
2. Create flags table
3. Create email_notifications table
4. ALTER ideas, votes, comments (add new columns)
5. DROP admin_users table
6. TRUNCATE ideas, votes, comments (fresh start)
```

## Authentication & User Management

### Registration Flow
- **Route**: `/register`
- **Form Fields**: email, name, password (min 8 chars)
- **Validation**:
  - Email format validation (no domain restrictions - open registration)
  - Name profanity filter
  - Password strength requirements
- **Process**: Hash password with bcrypt, create user with role=0 (User)
- **Post-Registration**: Auto-login and redirect to main board

### Login Flow
- **Route**: `/login`
- **Authentication**: Email + password against `users` table
- **Session**: Store `user_id` and `role` in session
- **Redirect**: Main board after successful login

### Initial Admin Bootstrap
- **Trigger**: On server startup, check if any admin exists (`role=2`)
- **Environment Variables**:
  - `INITIAL_ADMIN_EMAIL` (default: "admin")
  - `INITIAL_ADMIN_PASSWORD` (default: "admin")
- **Process**: If no admin exists, create one from env vars
- **Security**: Log warning if using default credentials

### User Profile
- **Route**: `/profile`
- **Features**:
  - View and edit name
  - Change password
  - View own submission history (ideas and comments)

### Role-Based Access Control
- **User (role=0)**: Can submit ideas, vote, comment, edit own content
- **Moderator (role=1)**: All User permissions + moderation actions
- **Admin (role=2)**: All Moderator permissions + user management

## Idea Submission & Moderation

### Idea Submission
- **Requirements**: Must be logged in
- **Form Fields**:
  - Title (max 100 chars)
  - Description (max 500 chars)
- **Validation**: Profanity filter on both fields
- **Defaults**: `is_public: true`, `stage: 'Ideate'`, `is_off_topic: false`
- **Visibility**: Immediately visible on main board (auto-publish)

### Idea Display
- **Main Board Query**: `is_public=true AND is_off_topic=false`
- **Display Fields**: Author name, creation date, vote count, comment count, stage badge
- **Stage Badge Colors**:
  - Ideate: Blue
  - Review: Yellow
  - In Progress: Orange
  - Completed: Green
- **Pinned Ideas**: Show at top of list with pin icon

### Author Editing
- **Permissions**: Edit own ideas only
- **Editable Fields**: Title and description
- **Validation**: Re-apply profanity filter
- **Restrictions**: Cannot change stage, cannot delete
- **History**: No edit history tracking (simple overwrite)

### Moderator Actions (role >= 1)
- **Change Stage**: Dropdown to move through lifecycle
- **Pin/Unpin**: Toggle `pinned_at` timestamp
- **Mark Off-Topic**: Set `is_off_topic=true` (soft hide)
- **Delete**: Hard delete from database (permanent)

### Moderator UI
- **Route**: `/admin/moderation`
- **Views**: Flagged content queue, off-topic items list
- **Actions**: Bulk operations available

## Voting & Commenting System

### Voting
- **Requirements**: Must be logged in (anonymous voting removed)
- **Database**: `votes(user_id, idea_id)` with unique constraint
- **Interaction**: Click to vote, click again to unvote
- **Vote Count**: Maintained by existing database triggers
- **UI**: Show vote count + visual indicator if user has voted

### Commenting
- **Requirements**: Must be logged in
- **Location**: Idea detail page
- **Validation**: Max 500 chars, profanity filter applied
- **Display**: Author name, timestamp, content
- **Sort Order**:
  - Pinned comments first
  - Then by creation date (oldest first)

### Comment Editing
- **Permissions**: Edit own comments only
- **Editable**: Content field only
- **Validation**: Re-apply profanity filter
- **Restrictions**: Cannot delete own comments
- **History**: No edit history (simple overwrite)

### Comment Moderation (role >= 1)
- **Pin**: Set `is_pinned=true` (shows at top)
- **Soft Delete**: Set `is_deleted=true` (shows "[deleted by moderator]")
- **Hard Delete**: Permanent removal from database
- **Display Logic**: Hide soft-deleted from public, show to moderators with label
- **Author Indicator**: Visual badge if commenter is the idea author

## Flagging & Moderation Features

### Flagging System
- **Trigger**: "Flag" button on ideas and comments (logged-in users only)
- **Process**: Single click, no reason/text required
- **Database**: Create `flags(target_type, target_id, user_id)`
- **Constraint**: User can only flag each item once
- **UI**: Flagged items show indicator to moderators only

### Moderator Flag Queue
- **Route**: `/admin/flags`
- **Display**: Content preview, flag count, flagging users, date
- **Grouping**: Multiple flags on same item grouped together
- **Actions**:
  - View full content in context
  - Mark as reviewed (clear all flags)
  - Mark off-topic (hide from public)
  - Delete permanently
  - Dismiss flags (keep content, remove flags)

### Moderator Dashboard
- **Route**: `/admin`
- **Widgets**:
  - Flagged items count
  - Recent ideas for review
  - Off-topic items list
  - Quick stats (users, ideas, comments)

### Admin-Only Features (role = 2)
- **Route**: `/admin/users`
- **Capabilities**:
  - View all users
  - Change user roles (0→1→2)
  - Ban/suspend users (future)
  - System settings (future: profanity filter toggle, stage customization)

## UI/UX Changes

### New Routes
```
/register           - Registration form
/login              - Login form
/logout             - Logout action
/profile            - User profile & settings
/admin              - Moderator dashboard (role >= 1)
/admin/flags        - Flag review queue (role >= 1)
/admin/moderation   - Off-topic items & bulk actions (role >= 1)
/admin/users        - User management (role = 2 only)
```

### Modified Routes
```
/                   - Main board (show login prompts if anonymous)
/ideas/:id          - Idea detail (add comments, author info, edit buttons)
```

### Header/Navigation
- **Logged In**: Show user name, dropdown menu (Profile, Logout)
- **Moderators**: "Admin" link in header
- **Anonymous**: Login/Register buttons

### Main Board Updates
- Remove anonymous submission form
- Show stage badges on idea cards
- Show pinned ideas at top with pin icon
- Show author name on each card
- Vote button disabled if not logged in (with tooltip)

### Idea Detail Page Updates
- Author info at top
- Edit button for idea author
- Comments section with login-required form
- Edit buttons on user's own comments
- Small, unobtrusive flag button
- Moderator tools (if role >= 1): stage dropdown, pin toggle, off-topic/delete

## Implementation Structure

### New Model Files

**src/models/user.rs**
```rust
- create_user(email, name, password) -> Result<User>
- authenticate(email, password) -> Option<User>
- get_by_id(id) -> Option<User>
- get_by_email(email) -> Option<User>
- update_role(id, role) -> Result<()>
- update_name(id, name) -> Result<()>
- update_password(id, password) -> Result<()>
- bootstrap_admin(email, password) -> Result<User>  // Check/create admin
```

**src/models/flag.rs**
```rust
- create_flag(user_id, target_type, target_id) -> Result<()>
- get_flagged_items() -> Vec<FlaggedItem>  // With counts grouped
- clear_flags(target_type, target_id) -> Result<()>
- has_flagged(user_id, target_type, target_id) -> bool
- get_flag_count(target_type, target_id) -> i32
```

### Updated Model Files

**src/models/idea.rs**
```rust
// Add user_id parameter to all operations
- create(user_id, title, description) -> Result<Idea>
- get_public() -> Vec<Idea>  // Filter: is_public=true AND is_off_topic=false
- get_by_user(user_id) -> Vec<Idea>  // For profile page
- get_with_author(id) -> Option<(Idea, User)>  // Join for display
- update_content(id, user_id, title, description) -> Result<()>  // Verify ownership
- update_stage(id, stage) -> Result<()>  // Moderator only
- toggle_pin(id) -> Result<()>  // Moderator only
- mark_off_topic(id, is_off_topic) -> Result<()>  // Moderator only
```

**src/models/vote.rs**
```rust
// Replace fingerprint with user_id
- has_voted(user_id, idea_id) -> bool
- toggle_vote(user_id, idea_id) -> Result<()>
- get_vote_count(idea_id) -> i32
```

**src/models/comment.rs**
```rust
// Add user_id to operations
- create(user_id, idea_id, content) -> Result<Comment>
- get_by_idea(idea_id, include_deleted: bool) -> Vec<(Comment, User)>  // Join with user
- update_content(id, user_id, content) -> Result<()>  // Verify ownership
- toggle_pin(id) -> Result<()>  // Moderator only
- soft_delete(id) -> Result<()>  // Set is_deleted=true
- hard_delete(id) -> Result<()>  // Permanent removal
```

### Server Functions
- Add authentication middleware to check session
- Pass `User` context to all server functions
- Return `AuthError` for protected actions
- Implement role checks for moderator/admin actions

### Route Structure
```
src/routes/
  auth.rs          - login, register, logout routes
  profile.rs       - user profile page
  admin/
    mod.rs         - dashboard
    flags.rs       - flag review queue
    moderation.rs  - off-topic items, bulk actions
    users.rs       - user management (admin only)
```

## Migration & Deployment

### Database Migration File
```
migrations/20260205_user_auth_system.up.sql
```

**Steps:**
1. Create users table
2. Create flags table
3. Create email_notifications table
4. ALTER ideas: add user_id, stage, is_public, is_off_topic, pinned_at
5. ALTER votes: replace voter_fingerprint with user_id, update constraint
6. ALTER comments: add user_id, is_pinned, is_deleted
7. DROP TABLE admin_users
8. TRUNCATE ideas, votes, comments (fresh start)

### Environment Variables
```bash
DATABASE_URL                # Existing
INITIAL_ADMIN_EMAIL         # Default: "admin"
INITIAL_ADMIN_PASSWORD      # Default: "admin"
# Future: SMTP_* settings when implementing email
```

### Testing Checklist
- [ ] Fresh database, verify admin bootstrap
- [ ] Register new users, test login/logout
- [ ] Submit ideas with profanity (should block)
- [ ] Submit valid ideas, verify auto-publish
- [ ] Vote and comment as different users
- [ ] Edit own ideas/comments
- [ ] Attempt to edit others' content (should fail)
- [ ] Flag content as regular user
- [ ] Review flags as moderator
- [ ] Test stage changes, pinning, off-topic marking
- [ ] Test soft delete vs hard delete
- [ ] Verify role permissions (user, moderator, admin)
- [ ] Test user management as admin

### Rollout Plan
1. **Pre-deployment**: Notify users of data wipe and maintenance window
2. **Deployment**: Run migration, wipe existing data
3. **Bootstrap**: First startup creates admin account from env vars
4. **Verification**: Test admin login, user registration flow
5. **Monitoring**: Watch logs for auth errors, bootstrap issues
6. **Post-deployment**: Share login instructions, admin credentials

## Future Enhancements (Deferred)

1. **UAB CAS SSO Integration**: Replace local auth with CAS for UAB users
2. **Email Notifications**: Implement SMTP sending for moderator alerts, user notifications
3. **Flag Reasons**: Add optional text field for flag details
4. **Edit History**: Track changes to ideas/comments with timestamps
5. **User Banning**: Suspend user accounts temporarily or permanently
6. **Stage Customization**: Admin UI to rename or add custom stages
7. **Comment Reactions**: Like/upvote comments
8. **Search & Filtering**: Search ideas by stage, author, keywords
9. **API Endpoints**: RESTful API for mobile app or integrations

## Success Criteria

- Users can register, login, and manage profiles
- All voting and commenting requires authentication
- Ideas auto-publish and display author information
- Moderators can manage content through stage lifecycle
- Flagging system allows community moderation
- Role-based permissions work correctly
- Profanity filter prevents obvious inappropriate content
- Admin can bootstrap from environment variables
- All existing anonymous data successfully wiped
- No authentication bypass vulnerabilities

---

**End of Design Document**
