# Change Log

## 2026-01-20

### Changed 
- `uuid4` ids for the `id` field of Orders
- Nanosecond, high-precision timestamps for the `timestamp` field of Orders

### Fixed
- Unnecessary Heap Allocations 
- Orders not being flushed on cancellation


### Added
- Benchmarks in regards to timing using Criterion
