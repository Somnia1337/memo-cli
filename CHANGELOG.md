# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.2] - 2025-07-12

### Changed

- A bunch of refactors.

## [0.4.1] - 2025-07-06

### Changed

- Every filename printed is now a clickable link.

### Removed

- The reviews count for the selected files.

## [0.4.0] - 2025-07-06

### Added

- Command line arg `--date`: denote an arbitrary date as "today" in the form of "%Y-%m-%d", this might be useful when preparing the notes to memo for the next few days.

## [0.3.4] - 2025-07-04

### Changed

- Moved "./revs" to Memo vault.
  - This prepares for synchronization with MacOS in the future.

## [0.3.3] - 2025-07-01

### Changed

- The saved data is sorted by path ascending.
  - This requires a format change (from `HashMap` to `Vec`).

## [0.3.2] - 2025-06-30

### Changed

- The printed list of files is sorted by weight ascending.

### Build

- Update cargo deps.

## [0.3.1] - 2025-06-03

### Changed

- The files now gets shuffled before choosing from, this should avoid `--top` picking the same top files between runs.

## [0.3.0] - 2025-05-30

### Added

- Command line args `--dry` and `--top`:
  - `--dry`: show note states, but don't modify review history.
  - `--top`: choose the notes with the highest priorities.
  - These 2 args are compatible.

## [0.2.0] - 2025-05-19

### Added

- Support for seperating subjects (101, 301, 408).

## [0.1.0] - 2025-05-10

### Added

- This project as a scientific memorizing helper, hopefully serve as a daily review helper with scientific weight calculations and quick links to each note.
- A somewhat arbitrary algorithm for calculating each notes' weight, and randomly chooses the notes to review today.
- A storage system, recording the paths to each note, its last reviewed date, and total review count.
