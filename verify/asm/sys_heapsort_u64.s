push   rbx
cmp    rsi,0x2
jb     <+0x15c>
lea    rcx,[rsi-0x2]
shr    rcx,1
jmp    <+0x26>
data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
sub    rcx,0x1
jb     <+0x9f>
lea    r10,[rcx*2+0x1]
cmp    r10,rsi
jae    <+0x20>
lea    r9,[rcx+rcx*1]
mov    rdx,QWORD PTR [rdi+rcx*8]
mov    r8,rcx
xchg   ax,ax
mov    rax,r10
cmp    QWORD PTR [rdi+r10*8],rdx
ja     <+0x4c>
mov    rax,r8
lea    r10,[r9+0x2]
cmp    r10,rsi
jae    <+0x6f>
cmp    rax,rsi
jae    <+0x17e>
mov    r9,QWORD PTR [rdi+r9*8+0x10]
cmp    r9,QWORD PTR [rdi+rax*8]
ja     <+0x6c>
mov    r10,rax
mov    rax,r10
cmp    rax,r8
je     <+0x20>
cmp    rax,rsi
jae    <+0x15e>
mov    r9,QWORD PTR [rdi+rax*8]
mov    QWORD PTR [rdi+r8*8],r9
mov    QWORD PTR [rdi+rax*8],rdx
lea    r9,[rax+rax*1]
lea    r10,[rax*2+0x1]
mov    r8,rax
cmp    r10,rsi
jb     <+0x40>
jmp    <+0x20>
mov    rdx,rsi
data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
lea    rax,[rdx-0x1]
cmp    rdx,0x2
mov    r8,rax
cmove  r8,rdx
cmp    rax,rsi
jae    <+0x15e>
mov    rcx,QWORD PTR [rdi]
mov    r9,QWORD PTR [rdi+rdx*8-0x8]
mov    QWORD PTR [rdi],r9
mov    QWORD PTR [rdi+rdx*8-0x8],rcx
cmp    rax,0x2
jb     <+0x149>
mov    r9,QWORD PTR [rdi]
mov    ebx,0x1
xor    r11d,r11d
xor    r10d,r10d
nop    DWORD PTR [rax+0x0]
mov    rcx,rbx
cmp    QWORD PTR [rdi+rbx*8],r9
ja     <+0xfc>
mov    rcx,r10
lea    rbx,[r11+0x2]
cmp    rbx,rax
jae    <+0x11f>
cmp    rcx,rsi
jae    <+0x18e>
mov    r11,QWORD PTR [rdi+r11*8+0x10]
cmp    r11,QWORD PTR [rdi+rcx*8]
ja     <+0x11c>
mov    rbx,rcx
mov    rcx,rbx
cmp    rcx,r10
je     <+0x149>
cmp    rcx,rsi
jae    <+0x16e>
mov    r11,QWORD PTR [rdi+rcx*8]
mov    QWORD PTR [rdi+r10*8],r11
mov    QWORD PTR [rdi+rcx*8],r9
lea    r11,[rcx+rcx*1]
lea    rbx,[rcx*2+0x1]
mov    r10,rcx
cmp    rbx,rax
jb     <+0xf0>
cmp    rdx,0x2
je     <+0x15c>
mov    rdx,r8
cmp    r8,0x2
jae    <+0xb0>
pop    rbx
ret
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
