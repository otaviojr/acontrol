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

#include <linux/delay.h>		            // sleep functions

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
static ssize_t dev_write(struct file *filp, const char __user *buf, size_t count, loff_t *pos);

static struct file_operations dev_file_operations = {
  .owner = THIS_MODULE,
  .open = dev_open,
  .release = dev_release,
  .unlocked_ioctl = dev_ioctl,
  .write = dev_write
};

static struct class *device_class;
static dev_t dev;
static struct cdev c_dev;
static struct device *char_device_object;

static int dev_open(struct inode* inodep, struct file* filep)
{
  DEBUG("BUZZER: Device openned");
  return 0;
}

static int dev_release(struct inode* inodep, struct file* filep)
{
  DEBUG("BUZZER: Device released");
  return 0;
}

static long dev_ioctl(struct file* filep, unsigned int cmd, unsigned long arg)
{
  long ret = 0;
  struct buzzer_tone tone;

  if (_IOC_TYPE(cmd) != BUZZER_IOC_MAGIC) return -EINVAL;
  if (_IOC_NR(cmd) > BUZZER_IOCTL_MAX_CMD) return -EINVAL;

  if (_IOC_DIR(cmd) & _IOC_READ) {
    ret = !access_ok((void __user *)arg, _IOC_SIZE(cmd));
  } else if (_IOC_DIR(cmd) & _IOC_WRITE) {
    ret = !access_ok((void __user *)arg, _IOC_SIZE(cmd));
  }

  if (ret) return -EACCES;

  switch(cmd){
    case BUZZER_IOCTL_GET_VERSION:
      if(copy_to_user((char*)arg, module_version, strlen(module_version) )){
        return -EACCES;
      }
      break;

    case BUZZER_IOCTL_PLAY_TONE:
      if(copy_from_user((struct buzzer_tone*)&tone, (struct buzzer_tone*)arg, sizeof(struct buzzer_tone))){
        return -EFAULT;
      }
      DEBUG("BUZZER: play_tone: %lu,%lu", tone.freq, tone.period);
      buzzer_pcm_play_tone(&tone);
      break;

    default:
      printk("BUZZER: Unknow ioctl command");
      return -EINVAL;
  }

  return ret;
}

static ssize_t dev_write(struct file *filp, const char __user *buf, size_t count, loff_t *pos)
{
  //TODO: We are living fine playing tone by tone. Maybe on the next version I improve this.
  return 0;
}

static int bcm2835_buzzer_probe(struct platform_device *pdev)
{
  int ret = 0;
  int result = 0;

  DEBUG("BUZZER: probe entered");

  device_class = class_create(THIS_MODULE, "buzzer");
  if(IS_ERR(device_class)) {
     printk(KERN_ALERT "BUZZER: Failed to create device class");
     ret = PTR_ERR(device_class);
     goto no_class_create;
  }

  /* character device interface */
  result = alloc_chrdev_region(&dev, FIRST_MINOR, MINOR_CNT, "buzzer");
  if(result < 0) {
    printk(KERN_ALERT "BUZZER: Failed registering region");
    ret = result;
    goto no_alloc_region;
  }

  cdev_init(&c_dev, &dev_file_operations);
  result = cdev_add(&c_dev, dev, MINOR_CNT);
  if(result < 0) {
    printk(KERN_ALERT "BUZZER: Error adding char device to region");
    ret = result;
    goto no_dev_init;
  }

  char_device_object = device_create(device_class, NULL, dev, NULL,  "buzzer");
  if(IS_ERR(char_device_object)) {
    printk(KERN_ALERT "BUZZER: Failed to create char device");
    ret = PTR_ERR(char_device_object);
    goto no_char_device_object;
  }

  ret = buzzer_pcm_load(pdev);
  if(ret != 0){
    printk(KERN_ALERT "BUZZER: Failed to load PCM");
    goto pcm_fails;
  }

  return 0;

pcm_fails:
  device_destroy(device_class, dev);

no_char_device_object:
  cdev_del(&c_dev);

no_dev_init:
  unregister_chrdev_region(dev,MINOR_CNT);

no_alloc_region:
  class_destroy(device_class);

no_class_create:
  return ret;
}

static int bcm2835_buzzer_remove(struct platform_device *pdev)
{
  DEBUG("BUZZER: remove entered");

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
