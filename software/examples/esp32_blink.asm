; target=esp32

main:
    entry a1, 40
    s32i #1, a1, 20
    s32i #0, a1, 24
    s32i #5000, a1, 8
    s32i #1, a1, 12
L0:
    l32i a4, a1, 20
    beqz a4, L1
    l32i a4, a1, 24
    seq v_t0, a4, #0
    beqz v_t0, L2
    s32i #1, a1, 24
    j L3
L2:
    s32i #0, a1, 24
L3:
    l32i a4, a1, 24
    s32i a4, a1, 16
    l32i a4, a1, 8
    s32i a4, a1, 4
L4:
    l32i a4, a1, 4
    sgt v_t1, a4, #0
    beqz v_t1, L5
    l32i a4, a1, 4
    sub v_t2, a4, #1
    s32i v_t2, a1, 4
    j L4
L5:
    j L0
L1:
    mov a2, #0
    j main_epilogue
main_epilogue:
    addi a1, a1, 40
    ret
