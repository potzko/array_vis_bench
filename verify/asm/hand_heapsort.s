push   rax
cmp    rsi,0x2
jae    <+0x40>
pop    rax
ret
mov    rdx,rsi
shr    rdx,1
jmp    <+0x89>
test   rdx,rdx
je     <+0x140>
dec    rdx
mov    rax,rdx
mov    rcx,rdx
lea    r8,[rax*2+0x1]
cmp    r8,rsi
jae    <+0xb0>
mov    rcx,QWORD PTR [rdi+r8*8]
cmp    rcx,QWORD PTR [rdi+rax*8]
ja     <+0xad>
mov    r8,rax
mov    rcx,r8
lea    r8,[rax*2+0x2]
cmp    r8,rsi
jae    <+0xdb>
cmp    rcx,rsi
jae    <+0x300>
lea    r9,[rax+rax*1]
mov    r9,QWORD PTR [rdi+r9*8+0x10]
cmp    r9,QWORD PTR [rdi+rcx*8]
ja     <+0xd8>
mov    r8,rcx
mov    rcx,r8
cmp    rcx,rax
je     <+0x80>
cmp    rax,rsi
jae    <+0x2c0>
cmp    rcx,rsi
jae    <+0x280>
mov    r8,QWORD PTR [rdi+rax*8]
mov    r9,QWORD PTR [rdi+rcx*8]
mov    QWORD PTR [rdi+rax*8],r9
mov    QWORD PTR [rdi+rcx*8],r8
mov    rax,rcx
lea    r8,[rax*2+0x1]
cmp    r8,rsi
jb     <+0xa0>
jmp    <+0xb0>
mov    rax,rsi
jmp    <+0x18a>
cmp    rax,0x2
jb     <+0x7>
dec    rax
cmp    rax,rsi
jae    <+0x340>
mov    rcx,QWORD PTR [rdi]
mov    rdx,QWORD PTR [rdi+rax*8]
mov    QWORD PTR [rdi],rdx
mov    QWORD PTR [rdi+rax*8],rcx
xor    edx,edx
mov    rcx,rdx
lea    r8,[rdx*2+0x1]
cmp    r8,rax
jb     <+0x200>
lea    r8,[rdx*2+0x2]
cmp    r8,rax
jb     <+0x21d>
cmp    rcx,rdx
jne    <+0x23b>
jmp    <+0x180>
mov    rcx,QWORD PTR [rdi+r8*8]
cmp    rcx,QWORD PTR [rdi+rdx*8]
ja     <+0x20d>
mov    r8,rdx
mov    rcx,r8
lea    r8,[rdx*2+0x2]
cmp    r8,rax
jae    <+0x1cd>
lea    r9,[rdx+rdx*1]
mov    r9,QWORD PTR [rdi+r9*8+0x10]
cmp    r9,QWORD PTR [rdi+rcx*8]
ja     <+0x22f>
mov    r8,rcx
mov    rcx,r8
cmp    rcx,rdx
je     <+0x180>
cmp    rcx,rsi
jae    <+0x280>
mov    r8,QWORD PTR [rdi+rdx*8]
mov    r9,QWORD PTR [rdi+rcx*8]
mov    QWORD PTR [rdi+rdx*8],r9
mov    QWORD PTR [rdi+rcx*8],r8
mov    rdx,rcx
lea    r8,[rdx*2+0x1]
cmp    r8,rax
jb     <+0x200>
jmp    <+0x1c0>
lea    rdx,[rip+?]
mov    rdi,rcx
call   QWORD PTR [rip+?]
lea    rdx,[rip+?]
mov    rdi,rax
call   QWORD PTR [rip+?]
lea    rdx,[rip+?]
mov    rdi,rcx
call   QWORD PTR [rip+?]
lea    rdx,[rip+?]
mov    rdi,rax
call   QWORD PTR [rip+?]
