push   r15
push   r14
push   rbx
mov    r14,rsi
mov    rbx,rdi
cmp    rsi,0x21
jb     <+0xbe>
mov    rax,QWORD PTR [rbx]
mov    rcx,QWORD PTR [rbx+r14*8-0x8]
mov    QWORD PTR [rbx],rcx
mov    QWORD PTR [rbx+r14*8-0x8],rax
mov    ecx,0x1
xor    r15d,r15d
jmp    <+0x48>
inc    rcx
cmp    r14,rcx
je     <+0x80>
mov    rdx,QWORD PTR [rbx+rcx*8-0x8]
cmp    rdx,rax
ja     <+0x40>
cmp    r15,r14
jae    <+0x280>
mov    rsi,QWORD PTR [rbx+r15*8]
mov    QWORD PTR [rbx+rcx*8-0x8],rsi
mov    QWORD PTR [rbx+r15*8],rdx
inc    r15
jmp    <+0x40>
cmp    r15,r14
jae    <+0x2c0>
mov    rax,QWORD PTR [rbx+r15*8]
mov    rcx,QWORD PTR [rbx+r14*8-0x8]
mov    QWORD PTR [rbx+r15*8],rcx
mov    QWORD PTR [rbx+r14*8-0x8],rax
mov    rdi,rbx
mov    rsi,r15
call   <hand_quicksort>
lea    rbx,[rbx+r15*8]
add    rbx,0x8
not    r15
add    r14,r15
cmp    r14,0x21
jae    <+0x20>
cmp    r14,0x2
jb     <+0x240>
lea    rdx,[r14-0x1]
add    r14,0xfffffffffffffffe
mov    eax,edx
and    eax,0x3
mov    ecx,0x1
cmp    r14,0x3
jb     <+0x1c5>
and    rdx,0xfffffffffffffffc
mov    esi,0x2
mov    edi,0x3
xor    r8d,r8d
jmp    <+0x115>
add    r8,0x4
add    rsi,0x4
add    rdi,0x4
cmp    r8,rdx
je     <+0x1c0>
mov    r9,QWORD PTR [rbx+rcx*8]
mov    r10,r8
mov    r11,QWORD PTR [rbx+r10*8]
cmp    r9,r11
jae    <+0x138>
mov    QWORD PTR [rbx+r10*8+0x8],r11
mov    QWORD PTR [rbx+r10*8],r9
add    r10,0xffffffffffffffff
jb     <+0x120>
mov    r9,QWORD PTR [rbx+rcx*8+0x8]
mov    r10,rcx
mov    r11,QWORD PTR [rbx+r10*8]
cmp    r9,r11
jae    <+0x158>
mov    QWORD PTR [rbx+r10*8+0x8],r11
mov    QWORD PTR [rbx+r10*8],r9
add    r10,0xffffffffffffffff
jb     <+0x140>
mov    r9,QWORD PTR [rbx+rcx*8+0x10]
mov    r10,rsi
mov    r11,QWORD PTR [rbx+r10*8]
cmp    r9,r11
jae    <+0x178>
mov    QWORD PTR [rbx+r10*8+0x8],r11
mov    QWORD PTR [rbx+r10*8],r9
add    r10,0xffffffffffffffff
jb     <+0x160>
mov    r9,QWORD PTR [rbx+rcx*8+0x18]
add    rcx,0x4
mov    r10,rdi
mov    r11,QWORD PTR [rbx+r10*8]
cmp    r9,r11
jae    <+0x100>
mov    QWORD PTR [rbx+r10*8+0x8],r11
mov    QWORD PTR [rbx+r10*8],r9
add    r10,0xffffffffffffffff
jb     <+0x190>
jmp    <+0x100>
test   rax,rax
je     <+0x240>
xor    edx,edx
jmp    <+0x208>
inc    rdx
cmp    rdx,rax
je     <+0x240>
mov    rdi,rcx
inc    rcx
mov    rsi,QWORD PTR [rbx+rdi*8]
mov    r8,QWORD PTR [rbx+rdi*8-0x8]
cmp    rsi,r8
jae    <+0x200>
mov    QWORD PTR [rbx+rdi*8],r8
mov    QWORD PTR [rbx+rdi*8-0x8],rsi
dec    rdi
jne    <+0x220>
jmp    <+0x200>
pop    rbx
pop    r14
pop    r15
ret
lea    rdx,[rip+?]
mov    rdi,r15
mov    rsi,r14
call   QWORD PTR [rip+?]
lea    rdx,[rip+?]
mov    rdi,r15
mov    rsi,r14
call   QWORD PTR [rip+?]
