# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Changelog

### Removed
- "id" from all of the API, because database id should not be exposed

### Fixed
- Output lastchangetime and countrycode in /json/stations/changed

## [0.5.1] - 2019-12-11
### Added
- "url_resolved" field to station lists
- "random" order to station lists

### Removed
- "negativevotes" field from station lists

### Fixed
- Fixed json result of vote endpoint
- Clean up tag and language fields on import
- Documentation cleanup and extensions

## [0.5.0] - 2019-12-08
### Added
- First documented release :)
