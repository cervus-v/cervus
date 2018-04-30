#include <linux/module.h>
#include <linux/slab.h>
#include <linux/sched.h>
#include <linux/sched/signal.h>
#include <linux/device.h>
#include <linux/kernel.h>
#include <linux/kfifo.h>
#include <linux/semaphore.h>
#include <linux/fs.h>
#include <linux/uaccess.h>
#include <linux/kthread.h>

const char *CLASS_NAME = "waks";
const char *DEVICE_NAME = "waks_load";

static int major_number;
static struct class *dev_class = NULL;
static struct device *dev_handle = NULL;
static int uapi_initialized = 0;

int uapi_init(void);
void uapi_cleanup(void);
static int wd_open(struct inode *, struct file *);
static int wd_release(struct inode *, struct file *);
static ssize_t wd_read(struct file *, char *, size_t, loff_t *);
static ssize_t wd_write(struct file *, const char *, size_t, loff_t *);

extern int run_code(
    const unsigned char *code_base,
    size_t code_len,
    size_t mem_default_len,
    size_t mem_max_len,
    size_t max_slots,
    size_t stack_len,
    size_t call_stack_len,
    void *kctx
);

struct execution_info {
    size_t len;
    char code[0];
};

int execution_worker(void *data) {
    struct execution_info *einfo;
    int ret;

    einfo = data;

    allow_signal(SIGKILL);

    ret = run_code(
        einfo -> code,
        einfo -> len,
        1048576,
        1048576 * 16,
        16384,
        1024,
        1024,
        NULL
    );

    printk(KERN_INFO "WebAssembly application exited with code %d\n", ret);

    vfree(einfo);

    return 0;
}

static struct file_operations waks_ops = {
    .open = wd_open,
    .read = wd_read,
    .write = wd_write,
    .release = wd_release
};

int uapi_init(void) {
    major_number = register_chrdev(0, DEVICE_NAME, &waks_ops);
    if(major_number < 0) {
        printk(KERN_ALERT "waks::exec_backend: Device registration failed\n");
        return major_number;
    }

    dev_class = class_create(THIS_MODULE, CLASS_NAME);
    if(IS_ERR(dev_class)) {
        unregister_chrdev(major_number, DEVICE_NAME);
        printk(KERN_ALERT "waks::exec_backend: Device class creation failed\n");
        return PTR_ERR(dev_class);
    }

    dev_handle = device_create(
        dev_class,
        NULL,
        MKDEV(major_number, 0),
        NULL,
        DEVICE_NAME
    );
    if(IS_ERR(dev_handle)) {
        class_destroy(dev_class);
        unregister_chrdev(major_number, DEVICE_NAME);
        printk(KERN_ALERT "waks::exec_backend: Device creation failed\n");
        return PTR_ERR(dev_handle);
    }

    printk(KERN_INFO "waks::exec_backend: UAPI initialized\n");
    uapi_initialized = 1;

    return 0;
}

void uapi_cleanup(void) {
    if(!uapi_initialized) return;

    // TODO: Is it possible that we still have open handles
    // to the UAPI device at this point?
    device_destroy(dev_class, MKDEV(major_number, 0));
    class_unregister(dev_class);
    class_destroy(dev_class);
    unregister_chrdev(major_number, DEVICE_NAME);
}

static int wd_open(struct inode *_inode, struct file *_file) {
    return 0;
}

static int wd_release(struct inode *_inode, struct file *_file) {
    return 0;
}

static ssize_t wd_read(struct file *_file, char *_data, size_t _len, loff_t *_offset) {
    return 0;
}

static ssize_t wd_write(struct file *_file, const char *data, size_t len, loff_t *offset) {
    struct execution_info *einfo;

    if(len > 1048576 * 16) {
        return -EINVAL;
    }

    printk(KERN_INFO "wd_write: Code length: %lu\n", len);

    einfo = vmalloc(sizeof(struct execution_info) + len);
    if(einfo == NULL) {
        return -ENOMEM;
    }

    einfo -> len = len;

    if(copy_from_user(einfo -> code, data, len)) {
        vfree(einfo);
        return -EFAULT;
    }

    kthread_run(execution_worker, einfo, "[anonymous webassembly application]");

    return len;
}
