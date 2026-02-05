- # Balzerboard (UAB) Database Structure

  This schema supports both UAB CAS SSO and local email/password authentication for testing and development.

  ## Tables

  ### 1. users
  The central identity record.
  - **id**: `serial` | [cite_start]Primary Key[cite: 1].
  - **uab_id**: `varchar(100)` | Unique BlazerID from CAS SSO; [cite_start]NULL for local test accounts.
  - **name**: `varchar(100)` | [cite_start]Full name used to link identity to submissions.
  - **email**: `varchar(200)` | [cite_start]UAB email address; unique for both SSO and local login[cite: 3, 11].
  - **role**: `smallint` | [cite_start]0: User, 1: Moderator, 2: Admin[cite: 7, 8, 9, 10].
  - **created_on**: `timestamptz`.

  ### 2. local_auth (New: Testing Only)
  Stores credentials for non-SSO login during testing.
  - **user_id**: `int` | Foreign Key to `users.id`.
  - **password_hash**: `varchar(255)` | Securely hashed password.
  - **is_test_account**: `boolean` | Flag to identify accounts created for the testing phase.

  ### 3. ideas
  [cite_start]Submissions that require moderation before becoming public[cite: 14].
  - **id**: `serial` | [cite_start]Primary Key[cite: 1].
  - **user_id**: `int` | [cite_start]Foreign Key to `users.id`.
  - **title**: `varchar(100)` | [cite_start]Displayed on the idea card[cite: 33].
  - **description**: `text` | [cite_start]Full content for detailed view[cite: 36].
  - **stage**: `varchar(50)` | [cite_start]Default: "Ideate"; supports renaming[cite: 18, 19].
  - **is_public**: `boolean` | [cite_start]Default `false`; requires moderator approval[cite: 14].
  - **is_off_topic**: `boolean` | [cite_start]Allows moderators to hide content[cite: 15].
  - **pinned_at**: `timestamptz` | [cite_start]Null if not pinned[cite: 26].
  - **created_on**: `timestamptz`.

  ### 4. votes & comments
  - [cite_start]**votes**: Unique `(user_id, idea_id)` to track upvotes[cite: 21, 35].
  - [cite_start]**comments**: Includes `is_pinned` and `is_deleted` for moderation[cite: 22, 25, 26].

  ### 5. email_notifications (SMTP)
  [cite_start]Queues moderator updates and user notifications[cite: 3].
  - **id**: `serial` | [cite_start]Primary Key[cite: 1].
  - **recipient_email**: `varchar(200)` | [cite_start]Target UAB email address[cite: 3].
  - **subject**: `varchar(255)` | [cite_start]Subject line for SMTP[cite: 3].
  - **body**: `text` | [cite_start]Email content[cite: 3].
  - **status**: `smallint` | 0: Pending, 1: Sent, 2: Failed.
  - **created_at**: `timestamptz`.

  ### 6. flags
  [cite_start]Moderation tracking for inappropriate ideas or comments[cite: 23].
  - **id**: `serial` | [cite_start]Primary Key[cite: 1].
  - **target_id**: `int` | ID of the flagged idea or comment.
  - **user_id**: `int` | The reporting user.
  - [cite_start]**reason**: `text`[cite: 23].