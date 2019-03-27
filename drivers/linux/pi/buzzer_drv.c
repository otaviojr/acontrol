/**
 * @file   buzzer_drv.c
 * @author Otavio Ribeiro
 * @date   23 Mar 2019
 * @brief  A kernel module to use PCM to drive a piezo buzzer
 *
 * Copyright (c) 2019 Otávio Ribeiro <otavio.ribeiro@gmail.com>
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

#include "buzzer_ioctl.h"
#include "buzzer_drv.h"
#include "buzzer_pcm.h"

static char* module_version = "0.0.1";

MODULE_ALIAS("platform:bcm2835-buzzer");
MODULE_LICENSE("GPL");
MODULE_AUTHOR("Otavio Ribeiro");
MODULE_DESCRIPTION("acontrol buzzer kernel driver");
MODULE_VERSION("0.1");

/* platform driver */
static int bcm2835_buzzer_probe(struct platform_device *pdev);
static int bcm2835_buzzer_remove(struct platform_device *pdev);

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

static int dev_open(struct inode* inodep, struct file* filep)
{
  printk("BUZZER: Device openned");
  return 0;
}

static int dev_release(struct inode* inodep, struct file* filep)
{
  printk("BUZZER: Device released");
  return 0;
}

static long dev_ioctl(struct file* filep, unsigned int cmd, unsigned long arg)
{
  long ret = 0;
  //long value = 0;
  //struct neopixel_pixel pixel;

  if (_IOC_TYPE(cmd) != BUZZER_IOC_MAGIC) return -EINVAL;
  if (_IOC_NR(cmd) > BUZZER_IOCTL_MAX_CMD) return -EINVAL;

  if (_IOC_DIR(cmd) & _IOC_READ) {
    ret = !access_ok(VERIFY_WRITE, (void *)arg, _IOC_SIZE(cmd));
  } else if (_IOC_DIR(cmd) & _IOC_WRITE) {
    ret = !access_ok(VERIFY_READ, (void *)arg, _IOC_SIZE(cmd));
  }

  return -EACCES;

  //if (ret) return -EACCES;

  //switch(cmd){
  //  case NEOPIXEL_IOCTL_GET_VERSION:
  //    if(copy_to_user((char*)arg, module_version, strlen(module_version) )){
  //      return -EACCES;
  //    }
  //    break;

  //  case  NEOPIXEL_IOCTL_GET_NUM_LEDS:
  //    value = neopixel_pwm_get_num_leds();
  //    if(copy_to_user((long*)arg, (long*)&value, sizeof(long))){
  //      return -EFAULT;
  //    }
  //    break;

  //  case NEOPIXEL_IOCTL_SET_PIXEL:
  //    if(copy_from_user((struct neopixel_pixel*)&pixel, (struct neopixel_pixel*)arg, sizeof(struct neopixel_pixel))){
  //      return -EFAULT;
  //    }
      //printk("NEOPIXEL: set_pixel: %lu,%d,%d,%d", pixel.pixel, pixel.red, pixel.green, pixel.blue);
  //    neopixel_pwm_set_pixel(pixel.pixel, pixel.red, pixel.green, pixel.blue);
  //    break;

  //  case NEOPIXEL_IOCTL_SHOW:
      //printk("NEOPIXEL: show");
  //    value = neopixel_pwm_show();
  //    if(copy_to_user((long*)arg, (long*)&value, sizeof(long))){
  //      return -EFAULT;
  //    }
  //    break;

  //  case NEOPIXEL_IOCTL_HARDWARE_TEST:
  //    neopixel_pwm_hardware_test();
  //    value = 0;
  //    if(copy_to_user((long*)arg, (long*)&value, sizeof(long))){
  //      return -EFAULT;
  //    }
  //    break;

  //  default:
  //    printk("NEOPIXEL: Unknow ioctl command");
  //    return -EINVAL;
  //}

  //return ret;
}

static int bcm2835_buzzer_probe(struct platform_device *pdev)
{
  int result = 0;

  printk("BUZZER: probe entered");

  device_class = class_create(THIS_MODULE, "buzzer");
  if(IS_ERR(device_class))
  {
     printk(KERN_ALERT "BUZZER: Failed to create device class");
     return PTR_ERR(device_class);
  }

  /* character device interface */
  result = alloc_chrdev_region(&dev, FIRST_MINOR, MINOR_CNT, "buzzer");
  if(result < 0)
  {
    printk(KERN_ALERT "BUZZER: Failed registering region");
    return result;
  }
  cdev_init(&c_dev, &dev_file_operations);
  result = cdev_add(&c_dev, dev, MINOR_CNT);
  if(result < 0)
  {
    printk(KERN_ALERT "BUZZER: Error adding char device to region");
    return result;
  }

  char_device_object = device_create(device_class, NULL, dev, NULL,  "buzzer");
  if(IS_ERR(char_device_object))
  {
    printk(KERN_ALERT "BUZZER: Failed to create char device");
    return PTR_ERR(char_device_object);
  }

  buzzer_pcm_load(pdev);

  return 0;
}

static int bcm2835_buzzer_remove(struct platform_device *pdev)
{
  printk("BUZZER: remove entered");

  buzzer_pcm_unload();

  device_destroy(device_class, dev);
  cdev_del(&c_dev);
  unregister_chrdev_region(dev,MINOR_CNT);
  class_destroy(device_class);

  return 0;
}

static const struct of_device_id bcm2835_buzzer_match[] = {
    { .compatible = "bcm2835-buzzer" },
    { }
};
MODULE_DEVICE_TABLE(of, bcm2835_buzzer_match);

static struct platform_driver bcm2835_buzzer_driver = {
	.probe	= bcm2835_buzzer_probe,
	.remove	= bcm2835_buzzer_remove,
	.driver = {
		.name = "bcm2835-buzzer",
                .owner = THIS_MODULE,
		.of_match_table = of_match_ptr(bcm2835_buzzer_match),
	},
};

module_platform_driver(bcm2835_buzzer_driver);
