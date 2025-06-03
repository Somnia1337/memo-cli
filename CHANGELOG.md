# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2025-06-03

### Changed

- The files now gets shuffled before choosing from, this should avoid `--top` picking the same top files between runs.

## [0.3.0] - 2025-05-30

### Added

- Useful args `--dry` and `--top`:
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
