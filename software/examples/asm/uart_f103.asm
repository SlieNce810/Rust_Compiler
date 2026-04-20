; target=stm32f403

main:
    push {r4-r7, fp, lr}
    mov fp, sp
    sub sp, sp, #32
    str #1, [fp, #-28]
    str #220, [fp, #-8]
    ; unsupported assign: call key1_read
    str v_t0, [fp, #-16]
    ; unsupported assign: call key2_read
    str v_t1, [fp, #-24]
    ldr r4, [fp, #-16]
    cmp r4, #0
    movgt v_t2, #1
    movle v_t2, #0
    cmp v_t2, #0
    beq L0
    str #80, [fp, #-8]
    b L1
L0:
    ldr r4, [fp, #-24]
    cmp r4, #0
    movgt v_t3, #1
    movle v_t3, #0
    cmp v_t3, #0
    beq L2
    str #520, [fp, #-8]
    b L3
L2:
L3:
L1:
    ldr r4, [fp, #-4]
    ldr r5, [fp, #-8]
    ; unsupported op
    ldr r4, [fp, #-28]
    mov r0, r4
    b main_epilogue
main_epilogue:
    add sp, sp, #32
    pop {r4-r7, fp, pc}
