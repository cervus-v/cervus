.globl __cv_enter_protected
__cv_enter_protected:
    # rdi: target function: (private_data, recover) -> ()
    # rsi: private data
    # returns: rax (0 for normal exit, 1 for unwind)
    pushq %rbp
    movq %rsp, %rbp

    pushq %rbx
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15

    movq %rdi, %rcx
    movq %rsi, %rdi
    movq %rsp, %rsi

    call *%rcx

    movq %rbp, %rsp
    popq %rbp

    mov $0, %rax
    ret

.globl __cv_begin_unwind
__cv_begin_unwind:
    # rdi: recover
    movq %rdi, %rsp
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rbx
    popq %rbp

    mov $1, %rax
    ret
