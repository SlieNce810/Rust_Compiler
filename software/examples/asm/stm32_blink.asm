; target=stm32f403

main:
    push {r4-r7, fp, lr}
    mov fp, sp
    sub sp, sp, #24
    str #1, [fp, #-12]
    str #0, [fp, #-16]
    str #5000, [fp, #-8]
    str #1, [fp, #-20]
L0:
    ldr r4, [fp, #-12]
    cmp r4, #0
    beq L1
    ldr r4, [fp, #-16]
    cmp r4, #0
    moveq v_t0, #1
    movne v_t0, #0
    cmp v_t0, #0
    beq L2
    str #1, [fp, #-16]
    b L3
L2:
    str #0, [fp, #-16]
L3:
    ldr r4, [fp, #-16]
    str r4, [fp, #-24]
    ldr r4, [fp, #-8]
    str r4, [fp, #-4]
L4:
    ldr r4, [fp, #-4]
    cmp r4, #0
    movgt v_t1, #1
    movle v_t1, #0
    cmp v_t1, #0
    beq L5
    ldr r4, [fp, #-4]
    sub v_t2, r4, #1
    str v_t2, [fp, #-4]
    b L4
L5:
    b L0
L1:
    mov r0, #0
    b main_epilogue
main_epilogue:
    add sp, sp, #24
    pop {r4-r7, fp, pc}
