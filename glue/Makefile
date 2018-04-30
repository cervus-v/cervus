obj-m += waks.o
waks-objs += waks-entry.o uapi.o $(OBJ_LIST)
HDR_PATH := /lib/modules/$(shell uname -r)/build

all:
	make -C $(HDR_PATH) M=$(PWD) modules

clean:
	make -C $(HDR_PATH) M=$(PWD) clean
