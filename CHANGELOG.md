# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.1] 2020-01-08
### Added
- Prometheus docs

### Fixed
- SQL error in endpoint /checks?seconds=x
- Database column mixup for state and language

### Changed
- Run as non root user in docker by default

## [0.6.0] - 2020-01-06
### Added
- Changelog
- Check fields: "metainfo_overrides_database", "public", "name", "description", "tags", "countrycode", "homepage", "favicon", "loadbalancer"

### Removed
- "id" from all of the API, because database id should not be exposed
- Endpoint /stations/broken 

### Changed
- Faster station import from pull source
- Restructured more code to the new style of connecting to the database, this should enable other types of databases (e.g.: postgresql) on the long term, this means also more error checking and use of transactions.

### Fixed
- Output lastchangetime and countrycode in /json/stations/changed
- Correctly collect and summarize different sources of station checks
- "numberofentries" added to pls output
- Less usages of "unwrap()"

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
