obj-m += neopixel.o
neopixel-objs := neopixel_drv.o 

# EXTRA_CFLAGS := 

all:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) modules
clean:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) clean

