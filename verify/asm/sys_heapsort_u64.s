push   rbx
cmp    rsi,0x2
jb     <+0x17c>
lea    rcx,[rsi-0x2]
shr    rcx,1
jmp    <+0x46>
sub    rcx,0x1
jb     <+0xc0>
lea    r10,[rcx*2+0x1]
cmp    r10,rsi
jae    <+0x40>
lea    r9,[rcx+rcx*1]
mov    rdx,QWORD PTR [rdi+rcx*8]
mov    r8,rcx
xchg   ax,ax
mov    rax,r10
cmp    QWORD PTR [rdi+r10*8],rdx
ja     <+0x6c>
mov    rax,r8
lea    r10,[r9+0x2]
cmp    r10,rsi
jae    <+0x8f>
cmp    rax,rsi
jae    <+0x200>
mov    r9,QWORD PTR [rdi+r9*8+0x10]
cmp    r9,QWORD PTR [rdi+rax*8]
ja     <+0x8c>
mov    r10,rax
mov    rax,r10
cmp    rax,r8
je     <+0x40>
cmp    rax,rsi
jae    <+0x180>
mov    r9,QWORD PTR [rdi+rax*8]
mov    QWORD PTR [rdi+r8*8],r9
mov    QWORD PTR [rdi+rax*8],rdx
lea    r9,[rax+rax*1]
lea    r10,[rax*2+0x1]
mov    r8,rax
cmp    r10,rsi
jb     <+0x60>
jmp    <+0x40>
mov    rdx,rsi
lea    rax,[rdx-0x1]
cmp    rdx,0x2
mov    r8,rax
cmove  r8,rdx
cmp    rax,rsi
jae    <+0x180>
mov    rcx,QWORD PTR [rdi]
mov    r9,QWORD PTR [rdi+rdx*8-0x8]
mov    QWORD PTR [rdi],r9
mov    QWORD PTR [rdi+rdx*8-0x8],rcx
cmp    rax,0x2
jb     <+0x169>
mov    r9,QWORD PTR [rdi]
mov    ebx,0x1
xor    r11d,r11d
xor    r10d,r10d
mov    rcx,rbx
cmp    QWORD PTR [rdi+rbx*8],r9
ja     <+0x11c>
mov    rcx,r10
lea    rbx,[r11+0x2]
cmp    rbx,rax
jae    <+0x13f>
cmp    rcx,rsi
jae    <+0x240>
mov    r11,QWORD PTR [rdi+r11*8+0x10]
cmp    r11,QWORD PTR [rdi+rcx*8]
ja     <+0x13c>
mov    rbx,rcx
mov    rcx,rbx
cmp    rcx,r10
je     <+0x169>
cmp    rcx,rsi
jae    <+0x1c0>
mov    r11,QWORD PTR [rdi+rcx*8]
mov    QWORD PTR [rdi+r10*8],r11
mov    QWORD PTR [rdi+rcx*8],r9
lea    r11,[rcx+rcx*1]
lea    rbx,[rcx*2+0x1]
mov    r10,rcx
cmp    rbx,rax
jb     <+0x110>
cmp    rdx,0x2
je     <+0x17c>
mov    rdx,r8
cmp    r8,0x2
jae    <+0xd0>
pop    rbx
ret
xchg   ax,ax
lea    rdx,[rip+?]
mov    rdi,rax
call   QWORD PTR [rip+?]
lea    rdx,[rip+?]
mov    rdi,rcx
call   QWORD PTR [rip+?]
lea    rdx,[rip+?]
mov    rdi,rax
call   QWORD PTR [rip+?]
lea    rdx,[rip+?]
mov    rdi,rcx
call   QWORD PTR [rip+?]
