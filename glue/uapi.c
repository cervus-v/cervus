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
#include <linux/cred.h>
#include <linux/security.h>
#include <linux/kthread.h>

#define CERVUS_LOAD_CODE 1
#define EXEC_HEXAGON_E 1

const char *CLASS_NAME = "cervus";
const char *DEVICE_NAME = "cvctl";

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
static ssize_t wd_ioctl(struct file *, unsigned int cmd, unsigned long arg);

extern int run_code_in_hexagon_e(
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
    int executor;
    uid_t euid;
    size_t len;
    char code[0];
};

int execution_worker(void *data) {
    struct execution_info *einfo;
    int ret;

    einfo = data;

    printk(KERN_INFO "cervus: starting application for user %d\n", einfo -> euid);

    allow_signal(SIGKILL);
    //allow_signal(SIGTERM);

    switch(einfo -> executor) {
        case EXEC_HEXAGON_E:
            ret = run_code_in_hexagon_e(
                einfo -> code,
                einfo -> len,
                1048576,
                1048576 * 16,
                16384,
                1024,
                1024,
                NULL
            );
            break;

        default:
            ret = -1;
            printk(KERN_INFO "cervus: Unknown executor: %d\n", einfo -> executor);
    }

    printk(KERN_INFO "cervus: (%d) WebAssembly application exited with code %d\n", task_pid_nr(current), ret);
    vfree(einfo);

    return 0;
}

static struct file_operations cervus_ops = {
    .open = wd_open,
    .read = wd_read,
    .write = wd_write,
    .release = wd_release,
    .unlocked_ioctl = wd_ioctl
};

int uapi_init(void) {
    major_number = register_chrdev(0, DEVICE_NAME, &cervus_ops);
    if(major_number < 0) {
        printk(KERN_ALERT "cervus: Device registration failed\n");
        return major_number;
    }

    dev_class = class_create(THIS_MODULE, CLASS_NAME);
    if(IS_ERR(dev_class)) {
        unregister_chrdev(major_number, DEVICE_NAME);
        printk(KERN_ALERT "cervus: Device class creation failed\n");
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
        printk(KERN_ALERT "cervus: Device creation failed\n");
        return PTR_ERR(dev_handle);
    }

    printk(KERN_INFO "cervus: uapi initialized\n");
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
    return -EINVAL;
}

struct load_code_info {
    int executor;
    unsigned long len;
    void *addr;
};

static ssize_t handle_load_code(struct file *_file, void *arg) {
    struct load_code_info lci;
    struct execution_info *einfo;
    const struct cred *cred;

    if(copy_from_user(&lci, arg, sizeof(struct load_code_info))) {
        return -EFAULT;
    }

    einfo = vmalloc(sizeof(struct execution_info) + lci.len);
    if(einfo == NULL) {
        return -ENOMEM;
    }

    cred = current_cred();

    einfo -> executor = lci.executor;
    einfo -> euid = cred -> euid.val;
    einfo -> len = lci.len;
    if(copy_from_user(einfo -> code, lci.addr, lci.len)) {
        vfree(einfo);
        return -EFAULT;
    }

    if(IS_ERR(
        kthread_run(execution_worker, einfo, "cervus-worker")
    )) {
        vfree(einfo);
        return -ENOMEM;
    }

    return 0;
}

#define DISPATCH_CMD(cmd, f) case cmd: return (f)(file, (void *) arg);

static ssize_t wd_ioctl(struct file *file, unsigned int cmd, unsigned long arg) {
    switch(cmd) {
        DISPATCH_CMD(CERVUS_LOAD_CODE, handle_load_code)
        default:
            return -EINVAL;
    }

    return -EINVAL;
}
