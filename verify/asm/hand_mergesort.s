push   r15
push   r14
push   r12
push   rbx
push   rax
cmp    rsi,0x2
jae    <+0x1a>
add    rsp,0x8
pop    rbx
pop    r12
pop    r14
pop    r15
ret
mov    rbx,rsi
mov    r15,rdi
lea    r12,[rsi*8+0x0]
mov    esi,0x1
mov    rdi,r12
call   QWORD PTR [rip+?]
test   rax,rax
je     <+0x63>
mov    r14,rax
mov    rdi,r15
mov    rsi,rbx
mov    rdx,rax
mov    rcx,rbx
call   <_RNvCsdFxcksAL2oa_13compare_sorts18hand_mergesort_rec>
mov    rdi,r14
add    rsp,0x8
pop    rbx
pop    r12
pop    r14
pop    r15
jmp    QWORD PTR [rip+?]
mov    edi,0x8
mov    rsi,r12
call   QWORD PTR [rip+?]
mov    rbx,rax
mov    rdi,r14
call   QWORD PTR [rip+?]
mov    rdi,rbx
call   <_Unwind_Resume@plt>
