TODO
 - Implement general interrupt poller
 - Implement cycle counting for CPU

BACKLOG
 - CPU reset (specifically, the program counter. Why subtract 4?)
    - Found that for nestest, C004 is the correct reset vector, ref: https://forums.nesdev.org/viewtopic.php?t=14268

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
05/09/23
 - Refactored CPU to produce cycle counts
 - Added test (passes!)

05/07/23
 - Added tile_viewer.rs to test rendering a single tile (works!)

05/03/23
 - Finished interrupt handler

05/02/23
 - Added new interrupt.rs file
 - Added nmi_interrupt_signal to PPU

05/01/23
 - Competed PPUADDR and PPUDATA read and write
 - Completed OAMDMA write (but did not wire it into CPU bus)
 - Wrote test cases
