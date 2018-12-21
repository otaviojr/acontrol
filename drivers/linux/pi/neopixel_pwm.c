/**
 * @file   neopixel_pwm.c
 * @author Otavio Ribeiro
 * @date   17 Dec 2018
 * @brief  NeoPixel PWM hardware
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
#include <linux/kernel.h>
#include <linux/module.h>
#include <linux/slab.h>
#include <linux/platform_device.h>
#include <linux/of.h>
#include <linux/io.h>
#include <linux/dma-mapping.h>
#include <linux/dmaengine.h>
#include <linux/delay.h>
#include <linux/kthread.h>

#include "neopixel_pwm.h"

static void* __iomem pwm_base_addr;
static void* __iomem pwmctl_cm_base_addr;

static unsigned int num_leds;
static uint8_t* buffer;
static struct device* dev;
static struct dma_chan* dma_chan;
static dma_addr_t dma_addr;

static struct resource* pwm_io_res;
static struct resource* pwmctl_cm_io_res;

static struct task_struct * hardware_test_task = NULL;

#define BYTES_PER_LED		9
#define RESET_BYTES		15

#define PWM_DMA_DREQ 		5

static int pwm_init( void )
{
  uint32_t reg;
  writel(32, pwm_base_addr + PWM_RNG1);
  writel(0, pwm_base_addr + PWM_DAT1);

  reg = PWM_CTL_PWEN1 |
        PWM_CTL_MODE1 |
        PWM_CTL_USEF1 |
        PWM_CTL_CLRF1 |
        PWM_CTL_MSEN1;

  writel(reg, pwm_base_addr + PWM_CTL);

  reg = PWM_DMAC_ENAB |
        PWM_DMAC_PANIC(4) |
        PWM_DMAC_DREQ(8);

  writel(reg, pwm_base_addr + PWM_DMAC);

  //PLLD - 500Mhz - MASH 1
  reg = readl(pwmctl_cm_base_addr + PWM_CM_CTL);
  reg |= PWM_CM_CTL_MASH(1) | PWM_CM_CTL_SRC(6);
  writel(reg, pwmctl_cm_base_addr + PWM_CM_CTL);

  reg |= PWM_CM_CTL_ENAB;
  writel(reg, pwmctl_cm_base_addr + PWM_CM_CTL);

  //2.5Mhz = 0,4us per bit
  reg = PWM_CM_DIV_DIVI(200) | PWM_CM_DIV_DIVF(0);
  writel(reg, pwmctl_cm_base_addr + PWM_CM_DIV);

  return 0;
}

static void neopixel_callback(void * param)
{
  dma_unmap_single(dev, dma_addr, num_leds * BYTES_PER_LED + RESET_BYTES, DMA_TO_DEVICE);
  dma_addr = 0;
  printk("NEOPIXEL: dma finished");
}

static int start_dma( void )
{
  struct dma_async_tx_descriptor *desc;

  if(dma_addr != 0){
    return -EFAULT;
  }

  dma_addr = dma_map_single(dev, buffer, num_leds * BYTES_PER_LED + RESET_BYTES, DMA_TO_DEVICE);

  if(dma_addr != 0)
  {
    printk("Error mapping DMA buffer");
    return -EFAULT;
  }

  desc = dmaengine_prep_slave_single(dma_chan, dma_addr, num_leds * BYTES_PER_LED + RESET_BYTES, DMA_TO_DEVICE, DMA_PREP_INTERRUPT);
  if(desc == NULL)
  {
    printk("Error preparing DMA transfer");
    return -EFAULT;
  }

  desc->callback = neopixel_callback;
  dma_async_issue_pending(dma_chan);

  return 0;
}

void neopixel_pwm_set_pixel(unsigned int pixel, unsigned char  red, unsigned char green, unsigned char blue)
{
  uint32_t color = (green << 16) | (red << 8) | blue;
  uint8_t* buffer_ptr;
  uint8_t i, j, bits=0;

  if(pixel > num_leds) return;

  buffer_ptr = &buffer[(pixel * 24 * 3)/8];

  for(i = 0; i < 24; i++)
  {
    if((color & 0x1000000) == 1)
    {
      for(j = 0; j < 3; j++)
      {
        bits++;
        if(bits % 8 == 0) { buffer_ptr++; bits = 0; }
        *buffer_ptr <<=1;
        *buffer_ptr |= (j <= 1 ? 1 : 0);
      }
    } else {
      for(j = 0; j < 3; j++)
      {
        bits++;
        if(bits % 8 == 0) { buffer_ptr++; bits = 0; }
        *buffer_ptr <<=1;
        *buffer_ptr |= (j == 0 ? 1 : 0);
      }
    }
    color <<= 1;
  }
}

int neopixel_pwm_get_num_leds( void )
{
  return num_leds;
}

int neopixel_pwm_show( void )
{
  return start_dma();
}

int neopixel_pwm_init( struct platform_device *pdev )
{
  struct device_node *np = pdev->dev.of_node;
  struct dma_slave_config cfg =
  {
    .src_addr_width = DMA_SLAVE_BUSWIDTH_4_BYTES,
    .dst_addr_width = DMA_SLAVE_BUSWIDTH_4_BYTES,
    .slave_id = PWM_DMA_DREQ,
    .direction = DMA_MEM_TO_DEV,
    .src_addr = 0,
  };

  dev = &pdev->dev;

  dma_addr = 0;

  pwm_io_res = platform_get_resource_byname(pdev, IORESOURCE_MEM, "neopixel-pwm");
  if(!pwm_io_res){
    printk("NEOPIXEL: pwm base address not found");
    return -ENODEV;
  } else {
    printk("NEOPIXEL: pwm base address 0x%lx - 0x%lx", (long unsigned int)pwm_io_res->start, (long unsigned int)pwm_io_res->end);
  }

  if  (!request_mem_region(pwm_io_res->start, resource_size(pwm_io_res), "neopixel-pwm")) {
    dev_err(&pdev->dev, "pwm -  request_mem_region");
    printk("NEOPIXEL: pwm request region failed. Region already in use?");
    return -EINVAL;
  }

  pwm_base_addr = ioremap(pwm_io_res->start, resource_size(pwm_io_res));

  if(!pwm_base_addr){
    printk("NWOPIXEL: Error remapping pwm io memory");
    return -ENOMEM;
  } else {
    printk("NEOPIXEL: PWM address remapped");
  }

  pwmctl_cm_io_res = platform_get_resource_byname(pdev, IORESOURCE_MEM, "neopixel-pwmctl-cm");
  if(!pwmctl_cm_io_res){
    printk("NEOPIXEL: pwmctl clock base address not found");
    return -ENODEV;
  } else {
    printk("NEOPIXEL: pwmctl clock base address 0x%lx - 0x%lx", (long unsigned int)pwmctl_cm_io_res->start, (long unsigned int)pwmctl_cm_io_res->end);
  }

  //if  (!request_mem_region(pwmctl_cm_io_res->start, resource_size(pwmctl_cm_io_res), "neopixel-pwm-cm")) {
  //  dev_err(&pdev->dev, "pwm -  request_mem_region");
  //  printk("NEOPIXEL: pwm request region failed. Region already in use?");
  //  return -EINVAL;
  //}  

  pwmctl_cm_base_addr = ioremap(pwmctl_cm_io_res->start, resource_size(pwmctl_cm_io_res));

  if(!pwmctl_cm_base_addr){
    printk("NWOPIXEL: Error remapping pwmctl clock io memory");
    return -ENOMEM;
  } else {
    printk("NEOPIXEL: PWMCTL clock address remapped");
  }

  if(of_property_read_u32(np, "num-leds", &num_leds) ) {
    dev_err(&pdev->dev, "of_property_read_u32\n");
    return -EINVAL;
  } else {
    printk("NEOPIXEL: number of leds = %d", num_leds);
  }

  buffer = kmalloc(num_leds * BYTES_PER_LED + RESET_BYTES, GFP_KERNEL);
  if(buffer == NULL)
  {
    printk("Failed to allocate pwm buffer\n");
    return -ENOMEM;
  }
 memset(buffer,0,num_leds * BYTES_PER_LED + RESET_BYTES);

  dma_chan = dma_request_slave_channel(dev, "neopixel-pwm-dma");
  if(dma_chan == NULL)
  {
    printk("Error requesting DMA channel");
    return -ENODEV;
  }

  cfg.dst_addr = (phys_addr_t)pwm_base_addr + PWM_FIF1;
  if(dmaengine_slave_config(dma_chan, &cfg) < 0)
  {
    printk("Error allocating DMA channel\n");
    return -ENODEV;
  }

  pwm_init();

  return 0;
}

int neopixel_pwm_unload( void )
{
  if(hardware_test_task)
  {
    //kthread_stop(hardware_test_task);
  }

  iounmap(pwm_base_addr);
  iounmap(pwmctl_cm_base_addr);

  release_mem_region(pwm_io_res->start, resource_size(pwm_io_res));
  //release_mem_region(pwmctl_cm_io_res->start, resource_size(pwmctl_cm_io_res));

  dma_release_channel(dma_chan);
  kfree(buffer);
  return 0;
}

static void color_wipe(uint8_t wait, uint8_t red, uint8_t green, uint8_t blue) {
  uint16_t i;
  for(i=0; i < num_leds; i++) {
    neopixel_pwm_set_pixel(i, red, green, blue);
    neopixel_pwm_show();
    msleep(wait * 1000);
  }
}

static int hardware_test(void* args)
{
   int stage = 0;
   printk(KERN_INFO "NEOPIXEL: Hardware test started \n");
   while(!kthread_should_stop())
   {
      set_current_state(TASK_RUNNING);
      color_wipe(1, (stage == 0 ? 255 : 0), (stage == 1 ? 255 : 0), (stage == 2 ? 255 : 0));
      set_current_state(TASK_INTERRUPTIBLE);
      msleep(1000);
      stage++;
      if(stage == 3) break;
   }
   printk(KERN_INFO "NEOPIXEL: Hardware test ended \n");
   return 0;
}

int neopixel_pwm_hardware_test( void ) 
{
   hardware_test_task = kthread_run(hardware_test, NULL, "neopixel_hardware_test");
   if(IS_ERR(hardware_test_task))
   {
      printk(KERN_ALERT "NEOPIXEL: Failed to create hardware test task");
      return PTR_ERR(hardware_test_task);
   }
   return 0;
}
