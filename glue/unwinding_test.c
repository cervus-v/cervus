#include "unwinding.h"
#include <stdio.h>

void __attribute__((noinline)) target2(unsigned long cb) {
    printf("target2\n");
    __cv_begin_unwind(cb);
}

void __attribute__((noinline)) target(void *data, unsigned long cb) {
    printf("target called with data %p & cb %p\n", data, cb);
    target2(cb);
    printf("After target2\n");
}

int main() {
    int status = __cv_enter_protected(target, NULL);
    printf("Returned to main (status = %d)\n", status);
    return 0;
}
