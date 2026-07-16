# HelixPulse — portfolio build order

```
HelixCore FULL
    → products 1 … 20 (domain-deep, not just scaffolds)
        → HelixPulse P1 (embedded KV)
            → HelixPulse P2 (protocol subset)
                → HelixPulse P3 (full cluster)   ← “eventually”
                    → HelixPulse P4 (multi-region)
```

## Rule

**No cluster implementation work** until:

1. `docs/goals/HELIXCORE_FULL.md` can be honestly closed (or residual P1s accepted)  
2. Product forges 1–20 are past “template parent/child” for their MVP domain  
3. At least one Core path (e.g. multi-replica rate limit) is blocked without Pulse  

Until then this project stays **scaffold + vision only**.
