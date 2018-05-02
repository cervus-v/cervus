#ifndef _KCTX_H_
#define _KCTX_H_

#include <linux/fs.h>

struct kernel_context {
    uid_t euid;
    struct file *stdin;
    struct file *stdout;
    struct file *stderr;
};

#endif
