; target=stm32f403

main:
    push {r4-r7, fp, lr}
    mov fp, sp
    sub sp, sp, #12
    str #0, [fp, #-4]
    str #10000, [fp, #-8]
    str #0, [fp, #-12]
L0:
    ldr r4, [fp, #-4]
    ldr r5, [fp, #-8]
    cmp r4, r5
    movlt v_t0, #1
    movge v_t0, #0
    cmp v_t0, #0
    beq L1
    ldr r4, [fp, #-4]
    add v_t1, r4, #1
    str v_t1, [fp, #-4]
    b L0
L1:
    ldr r4, [fp, #-4]
    ldr r5, [fp, #-8]
    cmp r4, r5
    moveq v_t2, #1
    movne v_t2, #0
    cmp v_t2, #0
    beq L2
    str #1, [fp, #-12]
    b L3
L2:
    str #0, [fp, #-12]
L3:
    ldr r4, [fp, #-12]
    mov r0, r4
    b main_epilogue
main_epilogue:
    add sp, sp, #12
    pop {r4-r7, fp, pc}
