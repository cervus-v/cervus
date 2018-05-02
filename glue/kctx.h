#ifndef _KCTX_H_
#define _KCTX_H_

#include <linux/fs.h>

struct user_string {
    unsigned long len;
    const char __user *data;
};

struct kernel_context {
    uid_t euid;
    struct file *stdin;
    struct file *stdout;
    struct file *stderr;

    int n_args;
    const struct user_string __user *args;
};

#endif
