# Changelog

All notable changes to the `fopost` crate are documented here.

## Unreleased

### Added

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
