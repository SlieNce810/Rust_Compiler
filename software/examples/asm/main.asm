; target=stm32f403

main:
    push {r4-r7, fp, lr}
    mov fp, sp
    sub sp, sp, #12
    str #4, [fp, #-4]
    str #3, [fp, #-8]
    ldr r4, [fp, #-8]
    mul v_t0, r4, #2
    ldr r4, [fp, #-4]
    add v_t1, r4, v_t0
    str v_t1, [fp, #-12]
    ldr r4, [fp, #-12]
    cmp r4, #5
    movgt v_t2, #1
    movle v_t2, #0
    cmp v_t2, #0
    beq L0
    ldr r4, [fp, #-12]
    sub v_t3, r4, #1
    str v_t3, [fp, #-12]
    b L1
L0:
    ldr r4, [fp, #-12]
    add v_t4, r4, #1
    str v_t4, [fp, #-12]
L1:
L2:
    ldr r4, [fp, #-12]
    cmp r4, #0
    movgt v_t5, #1
    movle v_t5, #0
    cmp v_t5, #0
    beq L3
    ldr r4, [fp, #-12]
    sub v_t6, r4, #1
    str v_t6, [fp, #-12]
    b L2
L3:
    ldr r4, [fp, #-12]
    mov r0, r4
    b main_epilogue
main_epilogue:
    add sp, sp, #12
    pop {r4-r7, fp, pc}
