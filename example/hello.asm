#define STDOUT $A000

    LDA #$48  ; H
    STA STDOUT
    LDA #$65  ; e
    STA STDOUT
    LDA #$6C  ; l
    STA STDOUT
    STA STDOUT
    LDA #$6F  ; o
    STA STDOUT
    LDA #$0A  ; \n
    STA STDOUT

    SED ; this is just to signal completion
hang:
    ; hang forever
    NOP
    JMP hang
