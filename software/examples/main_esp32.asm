; target=esp32

main:
    entry a1, 28
    s32i #4, a1, 4
    s32i #3, a1, 8
    l32i a4, a1, 8
    mull v_t0, a4, #2
    l32i a4, a1, 4
    add v_t1, a4, v_t0
    s32i v_t1, a1, 12
    l32i a4, a1, 12
    sgt v_t2, a4, #5
    beqz v_t2, L0
    l32i a4, a1, 12
    sub v_t3, a4, #1
    s32i v_t3, a1, 12
    j L1
L0:
    l32i a4, a1, 12
    add v_t4, a4, #1
    s32i v_t4, a1, 12
L1:
L2:
    l32i a4, a1, 12
    sgt v_t5, a4, #0
    beqz v_t5, L3
    l32i a4, a1, 12
    sub v_t6, a4, #1
    s32i v_t6, a1, 12
    j L2
L3:
    l32i a4, a1, 12
    mov a2, a4
    j main_epilogue
main_epilogue:
    addi a1, a1, 28
    ret
