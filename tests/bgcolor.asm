;
; A properly timed VIC implementation should display a 72px long, 8px high "staircase" color pattern when run.
;

!to "bgcolor.prg", cbm

; launch routine
*=$0801
!byte $0c, $08, $0a, $00, $9e, $20, $38, $31, $39, $32

*=$2000
.start:
    jsr clearScreen
    lda #$00
    sta $d021
.loop:
    inc $d021
    jmp .loop
    rts

clearScreen:
      lda #$00
      tax
      sta $d020
      sta $d021
      lda #$20
.clrLoop:
      sta $0400, x
      sta $0500, x
      sta $0600, x
      sta $0700, x
      dex
      bne .clrLoop
      rts