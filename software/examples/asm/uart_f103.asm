; target=stm32f403

main:
    push {r4-r7, fp, lr}
    mov fp, sp
    sub sp, sp, #12
    str #1, [fp, #-12]
    str #5000, [fp, #-8]
    ldr r4, [fp, #-8]
    str r4, [fp, #-4]
L0:
    ldr r4, [fp, #-4]
    cmp r4, #0
    movgt v_t0, #1
    movle v_t0, #0
    cmp v_t0, #0
    beq L1
    ldr r4, [fp, #-4]
    sub v_t1, r4, #1
    str v_t1, [fp, #-4]
    b L0
L1:
    ldr r4, [fp, #-12]
    mov r0, r4
    b main_epilogue
main_epilogue:
    add sp, sp, #12
    pop {r4-r7, fp, pc}
