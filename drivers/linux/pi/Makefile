obj-m += neopixel.o buzzer.o
neopixel-objs := neopixel_drv.o neopixel_pwm.o
buzzer-objs := buzzer_drv.o buzzer_pcm.o

# EXTRA_CFLAGS := 

all:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) modules
clean:
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) clean

