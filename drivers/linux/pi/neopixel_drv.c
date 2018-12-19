/**
 * @file   otarover_blue_drv.c
 * @author Otavio Ribeiro
 * @date   24 Dec 2017
 * @brief  A kernel module for controlling beaglebone blue board
 *
 * Copyright (c) 2018 Otávio Ribeiro <otavio.ribeiro@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
 * THE SOFTWARE.
 *
 */

#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/ioctl.h>
#include <linux/platform_device.h>
#include <linux/uaccess.h>
#include <linux/cdev.h>
#include <linux/gpio.h>                 // Required for the GPIO functions
#include <linux/kthread.h>              // Required for threads code

#include <linux/delay.h>		// sleep functions

#include "neopixel_ioctl.h"
#include "neopixel_drv.h"
#include "neopixel_pwm.h"

static char* module_version = "0.0.1";

MODULE_ALIAS("platform:bcm2835-neopixel");
MODULE_LICENSE("GPL");
MODULE_AUTHOR("Otavio Ribeiro");
MODULE_DESCRIPTION("acontrol neopixel kernel driver");
MODULE_VERSION("0.1");

/** GPIO2_2 - (2 * 32) + 2 **/
static unsigned int gpio_neopixel_data = 6;
module_param(gpio_neopixel_data, uint, S_IRUGO);
MODULE_PARM_DESC(gpio_neopixel_data, " GPIO NEOPIXEL DATA PIN (default=6)");

/* char device interface */
static int dev_open(struct inode* inodep, struct file* filep);
static int dev_release(struct inode* inodep, struct file* filep);
static long dev_ioctl(struct file* filep, unsigned int cmd, unsigned long arg);

static struct file_operations dev_file_operations = {
  .owner = THIS_MODULE,
  .open = dev_open,
  .release = dev_release,
  .unlocked_ioctl = dev_ioctl
};

static struct class *device_class;
static dev_t dev;
static struct cdev c_dev;
static struct device *char_device_object;

struct resource *io_res;
void * __iomem base_addr; /* Virtual Base Address */
unsigned long remap_size; /* Device Memory Size */

static int dev_open(struct inode* inodep, struct file* filep)
{
  return 0;
}

static int dev_release(struct inode* inodep, struct file* filep)
{
  return 0;
}

static long dev_ioctl(struct file* filep, unsigned int cmd, unsigned long arg)
{
  long ret = 0;

  if (_IOC_TYPE(cmd) != NEOPIXEL_IOC_MAGIC) return -EINVAL;
  if (_IOC_NR(cmd) > NEOPIXEL_IOCTL_MAX_CMD) return -EINVAL;

  if (_IOC_DIR(cmd) & _IOC_READ) {
    ret = !access_ok(VERIFY_WRITE, (void *)arg, _IOC_SIZE(cmd));
  } else if (_IOC_DIR(cmd) & _IOC_WRITE) {
    ret = !access_ok(VERIFY_READ, (void *)arg, _IOC_SIZE(cmd));
  }

  if (ret) return -EACCES;

  switch(cmd){
    case NEOPIXEL_IOCTL_GET_VERSION:
      if(copy_to_user((char*)arg, module_version, strlen(module_version) )){
        return -EACCES;
      }
      break;

    default:
      printk("NEOPIXEL: Unknow ioctl command");
      return -EINVAL;
  }

  return ret;
}

static int bcm2835_neopixel_probe(struct platform_device *pdev)
{
  int result = 0;

  printk("NEOPIXEL: probe entered");

  io_res = platform_get_resource(pdev, IORESOURCE_MEM,0);
  if(!io_res){
    printk("NEOPIXEL: dma base address not found");
  } else {
    printk("NEOPIXEL: Ok... we have an address 0x%lx - 0x%lx", io_res->start, io_res->end);
  }

  remap_size = io_res->end - io_res->start + 1;
  base_addr = ioremap(io_res->start, remap_size);

  if(!base_addr){
    printk("NWOPIXEL: Error remapping io memory");
  } else {
    printk("NEOPIXEL: Address remapped");
  }

  /* request the gpio connected to our neopixel strip data pin */
  gpio_request(gpio_neopixel_data, "sysfs");
  /* set as output and turn it off */
  gpio_direction_output(gpio_neopixel_data, 0);
  /* export and do not allow direction change */
  gpio_export(gpio_neopixel_data, false);

   device_class = class_create(THIS_MODULE, "neopixel");
   if(IS_ERR(device_class))
   {
      printk(KERN_ALERT "NEOPIXEL: Failed to create device class");
      return PTR_ERR(device_class);
   }

  /* character device interface */
  result = alloc_chrdev_region(&dev, FIRST_MINOR, MINOR_CNT, "neopixel");
  if(result < 0)
  {
    printk(KERN_ALERT "NEOPIXEL: Failed registering region");
    return result;
  }
  cdev_init(&c_dev, &dev_file_operations);
  result = cdev_add(&c_dev, dev, MINOR_CNT);
  if(result < 0)
  {
    printk(KERN_ALERT "NEOPIXEL: Error adding char device to region");
    return result;
  }

  char_device_object = device_create(device_class, NULL, dev, NULL,  "neopixel");
  if(IS_ERR(char_device_object))
  {
    printk(KERN_ALERT "NEOPIXEL: Failed to create char device");
    return PTR_ERR(char_device_object);
  }

  neopixel_pwm_init(base_addr);

  return 0;
}

static int bcm2835_neopixel_remove(struct platform_device *pdev)
{
  printk("NEOPIXEL: remove entered");

  //neopixel_pwm_init();

  neopixel_pwm_unload();

  gpio_set_value(gpio_neopixel_data, 0);
  gpio_unexport(gpio_neopixel_data);
  gpio_free(gpio_neopixel_data);

  device_destroy(device_class, dev);
  cdev_del(&c_dev);
  unregister_chrdev_region(dev,MINOR_CNT);
  class_destroy(device_class);

  iounmap(base_addr);

  return 0;
}

static const struct of_device_id bcm2835_neopixel_match[] = {
    { .compatible = "bcm2835-neopixel" },
    { }
};
MODULE_DEVICE_TABLE(of, bcm2835_neopixel_match);

static struct platform_driver bcm2835_neopixel_driver = {
	.probe	= bcm2835_neopixel_probe,
	.remove	= bcm2835_neopixel_remove,
	.driver = {
		.name = "bcm2835-neopixel",
                .owner = THIS_MODULE,
		.of_match_table = of_match_ptr(bcm2835_neopixel_match),
	},
};

module_platform_driver(bcm2835_neopixel_driver);
