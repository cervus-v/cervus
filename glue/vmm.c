#include <linux/slab.h>
#include <linux/mm.h>
#include <linux/mman.h>

#include "vmm.h"

static int map_area(unsigned long base, unsigned long len) {
    void *ret;
    ret = (void *) vm_mmap(
        NULL,
        base,
        len,
        PROT_READ | PROT_WRITE,
        MAP_SHARED | MAP_ANONYMOUS | MAP_FIXED | MAP_NORESERVE, // FIXME: MAP_NORESERVE is dangerous
        0
    );
    if(IS_ERR(ret)) {
        return PTR_ERR(ret);
    }

    if(ret != (void *) base) {
        return -EINVAL;
    }

    return 0;
}

int cv_vmm_init(void) {
    int ret;

    ret = map_area(TLS_BASE, PAGE_SIZE);
    if(ret < 0) return ret;

    ret = map_area(RT_BASE, VMM_4G);
    if(ret < 0) return ret;

    ret = map_area(VIRT_BASE, VMM_4G);
    if(ret < 0) return ret;

    return 0;
}
