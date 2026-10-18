# Changelog

All notable changes to the `fopost` crate are documented here.

## 0.4.0 - 2026-10-18

### Added

- `client.queue()` wraps the posting queue: `slots`, `create_slot`, `update_slot`,
  `delete_slot`, `next` and `preview`. `CreatePost::queue()` and `UpdatePost::queue()` send
  `schedule: "queue"` so the API picks the next free slot (`PostSchedule`). A changed or
  deleted slot answers with the queued posts it moved (`QueueMove`).
- `client.connect_sessions()` mints and reads hosted connect links (`create`, `get`);
  `CreateConnectSession` carries the end user's id, the networks offered, a redirect URL and
  `ConnectBranding`. The hosted `url` is present on creation only.
- `client.logs()` reads the request log (`GET /logs`) with every filter the API takes; the
  page carries `meta.next_cursor` (pass it back as `before`) and a `summary` of the window.
- `posts().unpublish` takes a live post down from every network and reports each
  `PlatformRemoval`; `PostStatus::Unpublished`.
- `webhooks().deliveries`, `delivery` and `redeliver` read the delivery log and queue a
  delivery again; `WebhookDeliveryDetail` carries the body exactly as it was sent.
- `automations().versions`, `version` and `restore_version` read and restore saved graph
  versions (`AutomationVersion`, `AutomationVersionDetail`, `WorkflowGraph`).
- `analytics().inbox` reads `GET /analytics/inbox`: totals, deltas, the daily series, volume
  by platform, type and account, response-time buckets and the weekday-hour heatmap.

### Fixed

- `accounts().delete_discord_message` now returns `()`: the API answers 204 with no body,
  so decoding a `DiscordAck` always failed.


### Added

- `accounts().reddit_subreddits`, `reddit_subreddit_rules`, `reddit_flairs` and
  `set_reddit_default_subreddit` read a Reddit account's destinations and pick its default;
  `validate().subreddit` checks a subreddit before posting, and `inbox().vote` casts a vote
  with `direction` `up`, `down` or `none` to take an earlier vote back. A flair id is valid
  only in the subreddit it came from.
- `client.blogs()` wraps the blogs, articles and products a connected site already owns,
  addressed by the platform's own ids (`list_blogs`, `list_articles`, `get_article`,
  `create_article`, `update_article`, `delete_article`, `list_products`,
  `update_product`). `update_article` edits the live article in place and never creates a
  second post. Reads need `posts`; writes to the site need `publish` as well.
- `blog().suggest_seeds(workspace_id, &SuggestBlogSeeds)` suggests about 10 search terms
  from the blog goal and topics (needs `ai`, 402 on an empty balance), and
  `blog().submit_article` moves a draft, failed or rejected article to `in_review` (needs
  `posts`, 409 otherwise).
- `BlogSettings` gains `goal` (`BlogGoal`), `focus_topics`, `avoid_topics` and `plan_run`
  (`BlogPlanRun` with a `BlogPlanRunStatus`); `UpdateBlogSettings` gains `goal`,
  `focus_topics` and `avoid_topics`. `BlogKeyword::fit_reason` says why a keyword serves the
  goal.
- `BlogArticleStatus::InReview`.
- `client.blog()` wraps the Blog tag: settings and the website secret, keyword planning,
  and the article lifecycle (`request_article`, `get_article`, `update_article`,
  `regenerate_article`, `publish_article`, `reject_article`, `delete_article`).
- `verify_webhook` and `WebhookSignature` check a delivery's signature in constant time,
  preferring the timestamped `X-FoPost-Signature-256` header and refusing a delivery more
  than five minutes from the clock; `sign_webhook` produces the same signature for tests.
- `media().limits(workspace_id)` reads `GET /media/limits`: the per-file ceiling by any
  route, the multipart ceiling, the connected-account tier and the next one, and the
  allowed content types.
- `PresignUpload` carries `folder_id`, `origin`, `brand_role`, `brand_voice_id` and
  `person_consent`; `PresignedUpload` gains `max_file_bytes`. Presign accepts sizes up to
  5 GB.
- `media().upload_direct_with(input, source)` streams a `FileSource::Path` from disk
  through the presigned PUT instead of holding the file in memory.
- `posts().create_with_files` and `posts().update_with_files` attach up to five
  `MediaFile`s to a post's first content block, as one multipart request when every file
  fits the multipart ceiling and through direct upload otherwise.
- `ApiError::is_file_too_large()` and `ApiError::max_file_bytes()` read a 413
  `file_too_large`.

### Changed

- `client.blog()` matches the live API contract: the workspace travels as `workspace_id` on
  the query string, single objects decode bare and lists from `{data}`, and every response
  model reads snake_case. `BlogWebsiteTest` now carries `http_status` (what the website
  answered) instead of `ok`, `status` and `error`; a failed test is still an `Error::Api`
  with status 502 and `http_status` in its body.
- `blog().plan_keywords` no longer fills the calendar: results land as `suggested`
  keywords, and setting one to `planned` with `update_keyword` picks it and gives it the
  next free date.
- `blog().publish_article` and a `publish_at` on `update_article` need an article in
  review (or already scheduled or failed); a draft answers 409 until it is submitted.
- `BlogSettings::review_hold_days` is now the number of days before its planned date that
  a picked keyword starts being written.
- Direct upload no longer has a 50 MB ceiling; the account's limit applies.

### Removed

- **Breaking:** `BlogSettings::auto_publish`, `UpdateBlogSettings::auto_publish` and its
  builder method. Every article now goes through `submit_article` before it publishes.
