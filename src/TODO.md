TODO
 - Implement general interrupt poller
 - Scrolling

BACKLOG
 - CPU reset (specifically, the program counter. Why subtract 4?)
    - Found that for nestest, C004 is the correct reset vector, ref: https://forums.nesdev.org/viewtopic.php?t=14268

~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
05/17/23
 - Fixed palette issue, completed user input
 - Everything works??????

05/11/23 - 05/14/23
 - Got the rendering to work (kind of), still lots of bugs

05/09/23
 - Refactored CPU to produce cycle counts
 - Added test (passes!)
 - Made progress on display

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
