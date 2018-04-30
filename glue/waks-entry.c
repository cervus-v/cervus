#include <linux/module.h>
#include <linux/slab.h>
#include <linux/delay.h>
#include <linux/sched.h>
#include <linux/sched/signal.h>

#define CHK_FATAL_SIGNAL() \
    if(fatal_signal_pending(current)) { \
        return -1; \
    }

extern int uapi_init(void);
extern void uapi_cleanup(void);

void *_GLOBAL_OFFSET_TABLE_ = NULL;

void lapi_printk(const char *base, size_t len) {
    printk("%.*s\n", (int) len, base);
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
    panic("wasm-linux bug\n");
}

void lapi_env_log(void *kctx, int level, const char *text_base, size_t text_len) {
    printk(KERN_INFO "[waks] %.*s\n", (int) text_len, text_base);
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

int __init init_module(void) {
    printk(KERN_INFO "wasm-linux loaded\n");
    uapi_init();
    return 0;
}

void __exit cleanup_module(void) {
    uapi_cleanup();
    printk(KERN_INFO "wasm-linux unloaded\n");
}

MODULE_LICENSE("GPL");
