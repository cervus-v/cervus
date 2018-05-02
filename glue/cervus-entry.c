#include <linux/module.h>
#include <linux/slab.h>
#include <linux/delay.h>
#include <linux/sched.h>
#include <linux/sched/signal.h>
#include <linux/semaphore.h>

#include "kctx.h"

// FIXME: Return -1 on SIGKILL only
#define CHK_FATAL_SIGNAL() \
    if(signal_pending(current)) { \
        printk(KERN_INFO "cervus: (%d) Terminating execution due to signal\n", task_pid_nr(current)); \
        return -1; \
    }

extern int uapi_init(void);
extern void uapi_cleanup(void);

extern int cervus_global_init(void);
extern void cervus_global_cleanup(void);

void *_GLOBAL_OFFSET_TABLE_ = NULL;

void lapi_printk(const char *base, size_t len) {
    printk(KERN_INFO "cervus: %.*s\n", (int) len, base);
}

unsigned char * lapi_kmalloc(size_t len) {
    unsigned char *mem = NULL;

    mem = vmalloc(len);

    // FIXME: Is this correct?
    while(mem == NULL) {
        cond_resched();
        mem = vmalloc(len);
    }

    return mem;
}

void lapi_kfree(unsigned char *ptr) {
    vfree((char *) ptr);
}

void lapi_bug(void) {
    panic("Cervus has panicked unexpectedly. This is a bug.\n");
}

static const char * get_log_prefix_for_level(int level) {
    switch(level) {
        case 1:
            return "[ERROR]";

        case 3:
            return "[WARNING]";

        case 6:
            return "[INFO]";

        default:
            return "";
    }
}

int lapi_env_get_uid(void *raw_kctx) {
    struct kernel_context *kctx = raw_kctx;
    return kctx -> euid;
}

static ssize_t write_trusted_cstr(struct file *file, const char *trusted_text) {
    size_t len = strlen(trusted_text);
    return kernel_write(file, trusted_text, len, 0);
}

struct file * lapi_env_get_stdin(void *raw_kctx) {
    struct kernel_context *kctx = raw_kctx;
    return kctx -> stdin;
}

struct file * lapi_env_get_stdout(void *raw_kctx) {
    struct kernel_context *kctx = raw_kctx;
    return kctx -> stdout;
}

struct file * lapi_env_get_stderr(void *raw_kctx) {
    struct kernel_context *kctx = raw_kctx;
    return kctx -> stderr;
}

ssize_t lapi_env_write_file(
    void *kctx,
    struct file *file,
    const char *data,
    size_t len,
    long long offset
) {
    return kernel_write(file, data, len, offset);
}

ssize_t lapi_env_read_file(
    void *kctx,
    struct file *file,
    char *data_out,
    size_t len,
    long long offset
) {
    return kernel_read(file, offset, data_out, len);
}

void lapi_env_log(void *raw_kctx, int level, const char *text_base, size_t text_len) {
    struct kernel_context *kctx = raw_kctx;

    if(kctx -> stderr) {
        write_trusted_cstr(kctx -> stderr, get_log_prefix_for_level(level));
        write_trusted_cstr(kctx -> stderr, " ");
        kernel_write(kctx -> stderr, text_base, text_len, 0);
        write_trusted_cstr(kctx -> stderr, "\n");
    } else {
        printk(KERN_INFO "cervus: (%d) %s %.*s\n",
            task_pid_nr(current),
            get_log_prefix_for_level(level),
            (int) text_len,
            text_base
        );
    }
}

int lapi_env_yield(void *kctx) {
    CHK_FATAL_SIGNAL();
    cond_resched();

    return 0;
}

int lapi_env_msleep(void *kctx, unsigned int ms) {
    CHK_FATAL_SIGNAL();
    msleep_interruptible(ms);
    CHK_FATAL_SIGNAL();

    return 0;
}

int lapi_env_reschedule(void *kctx) {
    schedule();
    CHK_FATAL_SIGNAL();
    return 0;
}

struct semaphore * lapi_semaphore_new(void) {
    struct semaphore *sem = kmalloc(sizeof(struct semaphore), GFP_KERNEL);
    if(!sem) return NULL;

    sema_init(sem, 0);
    return sem;
}

void lapi_semaphore_destroy(struct semaphore *sem) {
    kfree(sem);
}

void lapi_semaphore_up(struct semaphore *sem) {
    up(sem);
}

int lapi_semaphore_down(struct semaphore *sem) {
    int ret;

    CHK_FATAL_SIGNAL();
    ret = down_interruptible(sem);
    CHK_FATAL_SIGNAL();

    if(ret) {
        return -1;
    }

    return 0;
}

int __init init_module(void) {
    int ret;

    ret = cervus_global_init();
    if(ret) {
        printk(KERN_ALERT "cervus: global initialization failed with code %d\n", ret);
        return -EINVAL;
    }

    ret = uapi_init();
    if(ret) {
        printk(KERN_ALERT "cervus: uapi initialization failed with code %d\n", ret);
        return -EINVAL;
    }

    printk(KERN_INFO "cervus: service initialized\n");
    return 0;
}

void __exit cleanup_module(void) {
    cervus_global_cleanup();
    uapi_cleanup();
    printk(KERN_INFO "cervus: service stopped\n");
}

MODULE_LICENSE("GPL");
