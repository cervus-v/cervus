#ifndef _KCTX_H_
#define _KCTX_H_

#include <linux/fs.h>

#define MAX_N_ARGS 256
#define MAX_ARG_LEN 1024

struct kernel_string {
    unsigned long len;
    char *data;
};

struct kernel_context {
    uid_t euid;
    struct file *stdin;
    struct file *stdout;
    struct file *stderr;

    int n_args;
    struct kernel_string *args;
};

#endif
