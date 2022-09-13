#define N  $99
#define f0 $100
#define f1 $101
#define result $104

fibonacci:
    LDA #$00 ; fib := 0
    STA f0   ; f0 := 0
    LDX #$01
    STX f1   ; f1 := 1

    LDY N    
    DEY      ; i := N - 1
    BMI end  ; finish if N - 1 < 0
loop:
    ; fib := f0 + f1
    LDA #$00 ; fib := 0
    ADC f0   ; fib += f0
    ADC f1   ; fib += f1

    LDX f1   
    STX f0   ; f0 := f1
    STA f1   ; f1 := fib
    
    DEY       ; i := i - 1
    CPY #$00  ;
    BNE loop  ; goto loop if i > 0
    ; done
    STA result ; result := fib

end:
    SED ; this is just to signal completion
hang:
    ; hang forever
    NOP
    JMP hang
