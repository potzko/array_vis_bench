push   rax
cmp    rsi,0x2
jb     <+0x11a>
lea    rdx,[rsi-0x2]
shr    rdx,1
jmp    <+0x4a>
sub    rdx,0x1
jb     <+0x100>
lea    r8,[rdx*2+0x1]
cmp    r8,rsi
jae    <+0x40>
lea    r10,[rdx+rdx*1]
mov    rax,rdx
xchg   ax,ax
cmp    rax,rsi
jae    <+0x2c3>
mov    r11,QWORD PTR [rdi+r8*8]
mov    r9,QWORD PTR [rdi+rax*8]
lea    rcx,[r10+0x2]
cmp    r11,r9
cmovbe r8,rax
cmovbe r11,r9
cmp    rcx,rsi
jae    <+0xc0>
cmp    r8,rsi
jae    <+0x2c0>
mov    r10,QWORD PTR [rdi+r10*8+0x10]
mov    r11,QWORD PTR [rdi+r8*8]
cmp    r10,r11
cmovbe rcx,r8
cmova  r11,r10
cmp    rcx,rax
jne    <+0xcc>
jmp    <+0x40>
mov    rcx,r8
cmp    rcx,rax
je     <+0x40>
cmp    rcx,rsi
jae    <+0x240>
mov    QWORD PTR [rdi+rax*8],r11
mov    QWORD PTR [rdi+rcx*8],r9
lea    r10,[rcx+rcx*1]
lea    r8,[rcx*2+0x1]
mov    rax,rcx
cmp    r8,rsi
jb     <+0x60>
jmp    <+0x40>
lea    rcx,[rsi-0x1]
mov    rax,QWORD PTR [rdi]
mov    rdx,QWORD PTR [rdi+rsi*8-0x8]
mov    QWORD PTR [rdi],rdx
mov    QWORD PTR [rdi+rsi*8-0x8],rax
cmp    rcx,0x2
jae    <+0x159>
pop    rax
ret
mov    rax,QWORD PTR [rdi]
mov    rdx,QWORD PTR [rdi+rcx*8-0x8]
mov    QWORD PTR [rdi],rdx
mov    QWORD PTR [rdi+rcx*8-0x8],rax
dec    rcx
cmp    rcx,0x1
jbe    <+0x11a>
mov    rdx,QWORD PTR [rdi]
mov    r10d,0x1
xor    r9d,r9d
xor    r8d,r8d
mov    rax,r10
cmp    QWORD PTR [rdi+r10*8],rdx
jbe    <+0x1c0>
lea    r10,[r9+0x2]
cmp    r10,rcx
jb     <+0x1cc>
cmp    rax,r8
jne    <+0x1e6>
jmp    <+0x140>
mov    rax,r8
lea    r10,[r9+0x2]
cmp    r10,rcx
jae    <+0x182>
mov    r9,QWORD PTR [rdi+r9*8+0x10]
cmp    r9,QWORD PTR [rdi+rax*8]
ja     <+0x1da>
mov    r10,rax
mov    rax,r10
cmp    rax,r8
je     <+0x140>
cmp    rax,rsi
jae    <+0x280>
mov    r9,QWORD PTR [rdi+rax*8]
mov    QWORD PTR [rdi+r8*8],r9
mov    QWORD PTR [rdi+rax*8],rdx
lea    r9,[rax+rax*1]
lea    r10,[rax*2+0x1]
mov    r8,rax
cmp    r10,rcx
jb     <+0x170>
jmp    <+0x140>
lea    rdx,[rip+?]
mov    rdi,rcx
call   QWORD PTR [rip+?]
lea    rdx,[rip+?]
mov    rdi,rax
call   QWORD PTR [rip+?]
mov    rax,r8
lea    rdx,[rip+?]
mov    rdi,rax
call   QWORD PTR [rip+?]
