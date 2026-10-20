push   rax
cmp    rsi,0x2
jae    <+0x9>
pop    rax
ret
mov    rdx,rsi
shr    rdx,1
jmp    <+0x29>
data16 data16 data16 data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
test   rdx,rdx
je     <+0xb4>
dec    rdx
mov    rax,rdx
mov    rcx,rdx
lea    r8,[rax*2+0x1]
cmp    r8,rsi
jae    <+0x50>
nop
mov    rcx,QWORD PTR [rdi+r8*8]
cmp    rcx,QWORD PTR [rdi+rax*8]
ja     <+0x4d>
mov    r8,rax
mov    rcx,r8
lea    r8,[rax*2+0x2]
cmp    r8,rsi
jae    <+0x7b>
cmp    rcx,rsi
jae    <+0x1a5>
lea    r9,[rax+rax*1]
mov    r9,QWORD PTR [rdi+r9*8+0x10]
cmp    r9,QWORD PTR [rdi+rcx*8]
ja     <+0x78>
mov    r8,rcx
mov    rcx,r8
cmp    rcx,rax
je     <+0x20>
cmp    rax,rsi
jae    <+0x195>
cmp    rcx,rsi
jae    <+0x185>
mov    r8,QWORD PTR [rdi+rax*8]
mov    r9,QWORD PTR [rdi+rcx*8]
mov    QWORD PTR [rdi+rax*8],r9
mov    QWORD PTR [rdi+rcx*8],r8
mov    rax,rcx
lea    r8,[rax*2+0x1]
cmp    r8,rsi
jb     <+0x40>
jmp    <+0x50>
mov    rax,rsi
jmp    <+0xca>
nop    DWORD PTR [rax+0x0]
cmp    rax,0x2
jb     <+0x7>
dec    rax
cmp    rax,rsi
jae    <+0x1b5>
mov    rcx,QWORD PTR [rdi]
mov    rdx,QWORD PTR [rdi+rax*8]
mov    QWORD PTR [rdi],rdx
mov    QWORD PTR [rdi+rax*8],rcx
xor    edx,edx
mov    rcx,rdx
lea    r8,[rdx*2+0x1]
cmp    r8,rax
jb     <+0x120>
cs nop WORD PTR [rax+rax*1+0x0]
lea    r8,[rdx*2+0x2]
cmp    r8,rax
jb     <+0x13d>
cmp    rcx,rdx
jne    <+0x15b>
jmp    <+0xc0>
data16 data16 cs nop WORD PTR [rax+rax*1+0x0]
mov    rcx,QWORD PTR [rdi+r8*8]
cmp    rcx,QWORD PTR [rdi+rdx*8]
ja     <+0x12d>
mov    r8,rdx
mov    rcx,r8
lea    r8,[rdx*2+0x2]
cmp    r8,rax
jae    <+0x10d>
lea    r9,[rdx+rdx*1]
mov    r9,QWORD PTR [rdi+r9*8+0x10]
cmp    r9,QWORD PTR [rdi+rcx*8]
ja     <+0x14f>
mov    r8,rcx
mov    rcx,r8
cmp    rcx,rdx
je     <+0xc0>
cmp    rcx,rsi
jae    <+0x185>
mov    r8,QWORD PTR [rdi+rdx*8]
mov    r9,QWORD PTR [rdi+rcx*8]
mov    QWORD PTR [rdi+rdx*8],r9
mov    QWORD PTR [rdi+rcx*8],r8
mov    rdx,rcx
lea    r8,[rdx*2+0x1]
cmp    r8,rax
jb     <+0x120>
jmp    <+0x100>
lea    rdx,[rip+0x462b4]
mov    rdi,rcx
call   QWORD PTR [rip+0x48793]
lea    rdx,[rip+0x462a4]
mov    rdi,rax
call   QWORD PTR [rip+0x48783]
lea    rdx,[rip+0x4627c]
mov    rdi,rcx
call   QWORD PTR [rip+0x48773]
lea    rdx,[rip+0x462e4]
mov    rdi,rax
call   QWORD PTR [rip+0x48763]
