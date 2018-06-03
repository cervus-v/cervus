#include <linux/module.h>
#include <linux/slab.h>
#include <linux/delay.h>
#include <linux/sched.h>
#include <linux/signal.h>
#include <linux/mm.h>
#include <linux/sched/signal.h>
#include <linux/semaphore.h>
#include <linux/uaccess.h>

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

unsigned long lapi_get_total_ram_bytes(void) {
    return totalram_pages * PAGE_SIZE;
}

void lapi_oom_score_adj_current(short score) {
    unsigned long irq_flags;

    // TODO: Is the locking correct?
    spin_lock_irqsave(&current -> sighand -> siglock, irq_flags);

    // TODO: Check oom_score_adj_min?
    // TODO: Do what __set_oom_adj does?

    current -> signal -> oom_score_adj = score;
    spin_unlock_irqrestore(&current -> sighand -> siglock, irq_flags);
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

unsigned int lapi_env_get_n_args(void *raw_kctx) {
    struct kernel_context *kctx = raw_kctx;
    return kctx -> n_args;
}

ssize_t lapi_env_read_arg(void *raw_kctx, unsigned int id, char *out, size_t max_len) {
    size_t copy_len;
    struct kernel_context *kctx = raw_kctx;

    if(id >= kctx -> n_args) {
        return -1;
    }

    copy_len = kctx -> args[id].len < max_len ? kctx -> args[id].len : max_len;
    memcpy(out, kctx -> args[id].data, copy_len);
    
    return copy_len;
}

/*
static ssize_t write_trusted_cstr(struct file *file, const char *trusted_text) {
    size_t len = strlen(trusted_text);
    return kernel_write(file, trusted_text, len, 0);
}
*/

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

struct file * lapi_env_open_file(
    void *kctx,
    const char *name_base,
    size_t name_len,
    const char *flags_base,
    size_t flags_len
) {
    size_t i;
    int open_flags = 0;
    unsigned char allow_read = 0, allow_write = 0;
    char *name_buf;
    struct file *file;

    if(name_len > 255) {
        return NULL;
    }

    for(i = 0; i < flags_len; i++) {
        switch(flags_base[i]) {
            case 'r': allow_read = 1; break;
            case 'w': allow_write = 1; break;
            default: break;
        }
    }

    if(allow_read && allow_write) {
        open_flags |= O_RDWR;
    } else if(allow_read) {
        open_flags |= O_RDONLY;
    } else if(allow_write) {
        open_flags |= O_WRONLY;
    }

    name_buf = kmalloc(name_len + 1, GFP_KERNEL);
    if(!name_buf) {
        return NULL;
    }

    memcpy(name_buf, name_base, name_len);
    name_buf[name_len] = 0;
    file = filp_open(name_buf, open_flags, 0);
    kfree(name_buf);

    if(!file || IS_ERR(file)) {
        return NULL;
    } else {
        return file;
    }
}

void lapi_env_close_file(struct file *file) {
    filp_close(file, 0);
}

ssize_t lapi_env_write_file(
    void *kctx,
    struct file *file,
    const char *data,
    size_t len,
    long long offset
) {
    ssize_t ret;

    CHK_FATAL_SIGNAL();
    ret = kernel_write(file, data, len, offset);
    CHK_FATAL_SIGNAL();

    return ret;
}

ssize_t lapi_env_read_file(
    void *kctx,
    struct file *file,
    char *data_out,
    size_t len,
    long long offset
) {
    ssize_t ret;

    CHK_FATAL_SIGNAL();
    ret = kernel_read(file, offset, data_out, len);
    CHK_FATAL_SIGNAL();

    return ret;
}

void lapi_env_log(void *raw_kctx, int level, const char *text_base, size_t text_len) {
    struct kernel_context *kctx = raw_kctx;

    // Only root can print to kernel log
    if(kctx -> euid == 0) {
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
