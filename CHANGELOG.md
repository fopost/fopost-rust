# Changelog

All notable changes to the `fopost` crate are documented here.

## Unreleased

### Added

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

- Direct upload no longer has a 50 MB ceiling; the account's limit applies.
