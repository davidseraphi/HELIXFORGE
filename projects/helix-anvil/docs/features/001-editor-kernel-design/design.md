# 001 — Native editor kernel design · Design

## Status

**Planned.** Fill during 001 active work. Placeholder shape only.

## Target shape (draft — not ratified)

```
ui shell → editor commands/view → buffer kernel → filesystem / optional LSP
```

## Decisions to make in this packet

1. Buffer representation (rope vs gap buffer vs other)  
2. Coordinate system (bytes vs chars vs UTF-16 at LSP edge)  
3. Shell toolkit  
4. Multi-buffer / tab model for P0  
5. Test strategy for kernel without GUI  

## Non-decisions (later packets)

LSP protocol details, git UI, HelixCode remote open.
