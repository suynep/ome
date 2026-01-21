# Change Log


### Changed 
- `uuid4` ids for the `id` field of `Order`
- Nanosecond, high-precision timestamps for the `timestamp` field of `Order`
- `VecDeque` for handling unbounded `trades` field growth of `MatchingEngine` (current cap at 500)

### Fixed
- FIX: Market Order accumulation in cancel sets and orders map
- FIX: Remove lazy deletion to implement eviction on-spot
- Unnecessary Heap Allocations 
- Orders not being flushed on cancellation


### Heaptrack logs
> Processor: AMD Ryzen 5 7535HS @ 4.604GHz
> Memory: 16G DDR4

**For 1.5 million buy orders, 1.5 million sell orders, and 1.8 million trades**, the following heaptrack log was obtained:

```bash
total runtime: 71.66s.
calls to allocation functions: 18871314 (263363/s)
temporary memory allocations: 652350 (9104/s)
peak heap memory consumption: 259.24M # note this
peak RSS (including heaptrack overhead): 294.16M # note this
total memory leaked: 23.90K
```

This result seems quite good (~260M peak after removing unnecessary heap allocations).


