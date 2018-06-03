#ifndef _CV_VMM_H_
#define _CV_VMM_H_

#define TLS_BASE 0x10000UL
#define RT_BASE 0x100000000UL
#define VIRT_BASE 0x800000000UL

#define VMM_4G (1024UL * 1024 * 1024 * 4)

int cv_vmm_init(void);

#endif
