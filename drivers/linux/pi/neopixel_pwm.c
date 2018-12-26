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
static uint8_t* dma_buffer;

static unsigned long buffer_len;

static struct device* dev;
static struct dma_chan* dma_chan;
static dma_addr_t dma_addr;

static struct resource* pwm_io_res;
static struct resource* pwmctl_cm_io_res;

static struct task_struct * hardware_test_task = NULL;

#define BYTES_PER_LED		9
#define RESET_BYTES		42

#define PWM_DMA_DREQ 		5


static void pwm_enable( void )
{
  uint32_t reg;

  reg = PWM_DMAC_ENAB |
        PWM_DMAC_PANIC(4) |
        PWM_DMAC_DREQ(4);

  writel(reg, pwm_base_addr + PWM_DMAC);

  writel(32, pwm_base_addr + PWM_RNG1);
  writel(0, pwm_base_addr + PWM_DAT1);

  reg = PWM_CTL_PWEN1 |
        PWM_CTL_MODE1 |
        PWM_CTL_USEF1 |
        PWM_CTL_CLRF1 |
        PWM_CTL_MSEN1;

  writel(reg, pwm_base_addr + PWM_CTL);
  printk("NEOPIXEL: writing PWM CTL REGISTER =  0x%X", reg);

  reg = readl(pwm_base_addr + PWM_CTL);
  printk("NEOPIXEL: reading PWM CTL REGISTER =  0x%X", reg);

  reg = readl(pwm_base_addr + PWM_STA);
  printk("NEOPIXEL: PWM Status = 0x%X", reg);

  writel(0xFFFF, pwm_base_addr + PWM_STA);
}

static int pwm_init( void )
{
  uint32_t reg;

  //disable pwm clock
  reg = PWM_CM_CTL_PASSWORD | PWM_CM_CTL_KILL;
  writel(reg, pwmctl_cm_base_addr + PWM_CM_CTL);

  msleep(100);

  while( (reg = readl(pwmctl_cm_base_addr + PWM_CM_CTL)) & PWM_CM_CTL_BUSY )
  {
    msleep(100);
    printk("Waiting busy bit");
  }

  pwm_enable();

  //2.5Mhz = 0,45us per bit
  reg = PWM_CM_CTL_PASSWORD | PWM_CM_DIV_DIVI(210) | PWM_CM_DIV_DIVF(192);
  writel(reg, pwmctl_cm_base_addr + PWM_CM_DIV);

  msleep(100);

  //PLLD - PLLD 500Mhz - MASH 1
  reg = PWM_CM_CTL_PASSWORD | PWM_CM_CTL_MASH(1) | PWM_CM_CTL_SRC(6);
  writel(reg, pwmctl_cm_base_addr + PWM_CM_CTL);

  msleep(100);

  reg |= PWM_CM_CTL_ENAB;
  writel(reg, pwmctl_cm_base_addr + PWM_CM_CTL);
  printk("writing PWM_CM_CTL=0x%X",reg);

  msleep(100);

  reg = readl(pwmctl_cm_base_addr + PWM_CM_CTL);
  printk("reading PWM_CM_CTL=0x%X",reg);

  return 0;
}

static void neopixel_callback(void * param)
{
  dma_unmap_single(dev, dma_addr, buffer_len, DMA_TO_DEVICE);
  dma_addr = 0;
  printk("NEOPIXEL: dma finished");
}

static void set_dma_buffer( void )
{
  int i;
  uint8_t* p_buffer = buffer;
  uint8_t* p_dma_buffer = dma_buffer;

  for(i = 0; i < buffer_len/4; i++){
    p_dma_buffer[3] = p_buffer[0];
    p_dma_buffer[2] = p_buffer[1];
    p_dma_buffer[1] = p_buffer[2];
    p_dma_buffer[0] = p_buffer[3];

    p_buffer += 4;
    p_dma_buffer += 4;
  }
}

static int start_dma( void )
{
  struct dma_async_tx_descriptor *desc;

  if(dma_addr != 0) return -EFAULT;

  set_dma_buffer();

  printk("NEOPIXEL: start_dma");

  dma_addr = dma_map_single(dev, dma_buffer, buffer_len, DMA_TO_DEVICE);
  if(dma_mapping_error(dev, dma_addr))
  {
    printk("Error mapping DMA buffer");
    return -EFAULT;
  }

  dma_sync_single_for_device(dev, dma_addr, buffer_len, DMA_TO_DEVICE);

  desc = dmaengine_prep_slave_single(dma_chan, dma_addr, buffer_len, DMA_TO_DEVICE, DMA_PREP_INTERRUPT);
  if(desc == NULL)
  {
    printk("Error preparing DMA transfer");
    return -EFAULT;
  }

  desc->callback = neopixel_callback;
  desc->callback_param = NULL;
  dmaengine_submit(desc);
  dma_async_issue_pending(dma_chan);

  return 0;
}

void neopixel_pwm_set_pixel(unsigned int pixel, uint8_t red, uint8_t green, uint8_t blue)
{
  uint32_t color = ( ((uint32_t)green) << 16) | ( ((uint32_t)red) << 8) | blue;
  uint8_t* buffer_ptr;
  uint8_t i, j, bits=0;

  if(pixel > num_leds) return;

  printk("NEOPIXEL: Setting pixel (0x%X)", color);

  //TODO: *9
  buffer_ptr = &buffer[(pixel * 24 * 3)/8];

  memset(buffer_ptr, 0, BYTES_PER_LED);

  for(i = 0; i < 24; i++)
  {
    for(j = 0; j < 3; j++)
    {
      if(bits == 8) { buffer_ptr++; bits = 0; }
      *buffer_ptr <<=1;
      if(color & 0x800000)
        *buffer_ptr |= (j <= 1 ? 1 : 0);
      else
        *buffer_ptr |= (j == 0 ? 1 : 0);
      bits++;
    }
    color <<= 1;
  }
}

void neopixel_pwm_clear_pixels( void )
{
  uint32_t i;
  for(i = 0; i < num_leds; i++){
    neopixel_pwm_set_pixel(i,0,0,0);
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
  int ret = 0;

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
    ret = -ENODEV;
    goto no_neopixel_pwm;
  } else {
    printk("NEOPIXEL: pwm base address 0x%lx - 0x%lx", (long unsigned int)pwm_io_res->start, (long unsigned int)pwm_io_res->end);
  }

  if  (!request_mem_region(pwm_io_res->start, resource_size(pwm_io_res), "neopixel-pwm")) {
    dev_err(dev, "pwm -  request_mem_region");
    printk("NEOPIXEL: pwm request region failed. Region already in use?");
    ret = -EINVAL;
    goto no_pwm_request_mem;
  }

  pwm_base_addr = ioremap(pwm_io_res->start, resource_size(pwm_io_res));

  if(!pwm_base_addr){
    printk("NWOPIXEL: Error remapping pwm io memory");
    ret = -ENOMEM;
    goto no_remap_pwm;
  } else {
    printk("NEOPIXEL: PWM address remapped");
  }

  pwmctl_cm_io_res = platform_get_resource_byname(pdev, IORESOURCE_MEM, "neopixel-pwmctl-cm");
  if(!pwmctl_cm_io_res){
    printk("NEOPIXEL: pwmctl clock base address not found");
    ret = -ENODEV;
    goto no_pwm_ctl_resource;
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
    ret = -ENOMEM;
    goto no_remap_pwm_ctl;
  } else {
    printk("NEOPIXEL: PWMCTL clock address remapped");
  }

  if(of_property_read_u32(np, "num-leds", &num_leds) ) {
    dev_err(dev, "of_property_read_u32\n");
    ret = -EINVAL;
    goto no_num_leds;
  } else {
    printk("NEOPIXEL: number of leds = %d", num_leds);
  }

  buffer_len = num_leds * BYTES_PER_LED + RESET_BYTES;

  buffer = kmalloc(buffer_len, GFP_KERNEL);
  if(buffer == NULL)
  {
    printk("Failed to allocate pwm buffer");
    goto no_buffer;
  }
  memset(buffer,0,buffer_len);

  dma_buffer = kmalloc(buffer_len, GFP_KERNEL | GFP_DMA);
  if(dma_buffer == NULL)
  {
    printk("Failed to allocate pwm dma buffer");
    goto no_dma_buffer;
  }

  memset(dma_buffer,0,buffer_len);

  dma_chan = dma_request_slave_channel(dev, "neopixel-pwm-dma");
  if(dma_chan == NULL)
  {
    printk("Error requesting DMA channel");
    ret = -ENODEV;
    goto no_dma_request_channel;
  }

  cfg.dst_addr = (dma_addr_t)0x7e20c000 + PWM_FIF1;
  if(dmaengine_slave_config(dma_chan, &cfg) < 0)
  {
    printk("Error allocating DMA channel\n");
    ret = -ENODEV;
    goto no_dma_config;
  }

   neopixel_pwm_clear_pixels();

  pwm_init();

  return 0;

no_dma_config:
no_dma_request_channel:
  dma_release_channel(dma_chan);

no_dma_buffer:
  kfree(dma_buffer);

no_buffer:
  kfree(buffer);

no_num_leds:
no_remap_pwm_ctl:
  iounmap(pwmctl_cm_base_addr);

no_pwm_ctl_resource:
no_remap_pwm:
  iounmap(pwm_base_addr);

no_pwm_request_mem:
  release_mem_region(pwm_io_res->start, resource_size(pwm_io_res));

no_neopixel_pwm:
  return ret;
}

int neopixel_pwm_stop( void )
{
  if(hardware_test_task)
  {
    kthread_stop(hardware_test_task);
    hardware_test_task = NULL;
  }
  return 0;
}

int neopixel_pwm_unload( void )
{
  if(hardware_test_task)
  {
    kthread_stop(hardware_test_task);
    hardware_test_task = NULL;
  }

  dmaengine_terminate_async(dma_chan);
  dmaengine_synchronize(dma_chan);

  iounmap(pwm_base_addr);
  iounmap(pwmctl_cm_base_addr);

  release_mem_region(pwm_io_res->start, resource_size(pwm_io_res));
  //release_mem_region(pwmctl_cm_io_res->start, resource_size(pwmctl_cm_io_res));

  dma_release_channel(dma_chan);

  kfree(buffer);
  kfree(dma_buffer);

  return 0;
}

static void color_wipe(uint8_t wait, uint8_t red, uint8_t green, uint8_t blue) {
  uint16_t i;
  for(i=0; i < num_leds; i++) {
    neopixel_pwm_set_pixel(i, red, green, blue);
    neopixel_pwm_show();
    msleep(wait * 1000);
    if(kthread_should_stop())
    {
      break;
    }
  }
}

static int hardware_test(void* args)
{
  int stage = 0;
  printk(KERN_INFO "NEOPIXEL: Hardware test started \n");
  while(!kthread_should_stop())
  {
    set_current_state(TASK_RUNNING);
    color_wipe(5, (stage == 0 ? 255 : 0), (stage == 1 ? 255 : 0), (stage == 2 ? 255 : 0));
    stage++;
    if(stage == 4)
    {
      hardware_test_task = NULL;
      printk(KERN_INFO "NEOPIXEL: Hardware test ended - completed\n");
      do_exit(0);
      break;
    }
    set_current_state(TASK_INTERRUPTIBLE);
    msleep(1000);
  }
  printk(KERN_INFO "NEOPIXEL: Hardware test ended - aborted\n");
  hardware_test_task = NULL;
  return 0;
}

int neopixel_pwm_hardware_test( void )
{
  if(hardware_test_task){
    kthread_stop(hardware_test_task);
  }
  hardware_test_task = kthread_run(hardware_test, NULL, "neopixel_hardware_test");
  if(IS_ERR(hardware_test_task))
  {
     printk(KERN_ALERT "NEOPIXEL: Failed to create hardware test task");
     return PTR_ERR(hardware_test_task);
  }
  return 0;
}
