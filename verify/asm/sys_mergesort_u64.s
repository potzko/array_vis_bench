push   r15
push   r14
push   r12
push   rbx
push   rax
cmp    rsi,0x2
jae    <+0x40>
add    rsp,0x8
pop    rbx
pop    r12
pop    r14
pop    r15
ret
mov    rbx,rsi
mov    r12,rdi
lea    r15,[rsi*8+0x0]
mov    rdi,r15
call   QWORD PTR [rip+?]
test   rax,rax
je     <+0xc0>
mov    r14,rax
mov    rdi,rax
mov    rsi,r12
mov    rdx,r15
call   QWORD PTR [rip+?]
mov    rdi,r12
mov    rsi,rbx
mov    rdx,r14
mov    rcx,rbx
call   <_RINvMNtCs_14merge_sort_lib8top_downINtB3_16TopDownMergeSortINtCs_20small_sort_insertion18InsertionSmallSortNtCs_29small_sort_insertion_strategy15LinearInsertionKj20_EKb0_KB3c_E7sort_cbyNtNtCs_11sort_logger7loggers10NoOpLoggerECs_13compare_sorts>
mov    rdi,r14
add    rsp,0x8
pop    rbx
pop    r12
pop    r14
pop    r15
jmp    QWORD PTR [rip+?]
mov    edi,0x8
mov    rsi,r15
call   QWORD PTR [rip+?]
mov    rbx,rax
mov    rdi,r14
call   QWORD PTR [rip+?]
mov    rdi,rbx
call   <_Unwind_Resume@plt>
