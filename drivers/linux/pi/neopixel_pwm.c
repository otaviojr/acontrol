/**
 * @file   neopixel_pwm.c
 * @author Otavio Ribeiro
 * @date   17 Dec 2018
 * @brief  NeoPixel linux kernel PWM module
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
#include <linux/of_address.h>
#include <linux/io.h>
#include <linux/dma-mapping.h>
#include <linux/dmapool.h>
#include <linux/dmaengine.h>
#include <linux/slab.h>
#include <linux/delay.h>
#include <linux/kthread.h>

#include "neopixel_pwm.h"

static volatile void* __iomem pwm_base_addr;
static volatile void* __iomem pwmctl_cm_base_addr;

static unsigned int num_leds = 0;

static uint8_t* buffer = NULL;
static uint8_t* dma_buffer = NULL;

static unsigned long buffer_len;

static struct device* dev;
static struct dma_chan* dma_chan;
static struct dma_pool* neo_dma_pool;
static dma_addr_t dma_addr;
static struct dma_async_tx_descriptor * dma_desc;

//static struct completion cmp;

static struct resource* pwm_io_res;
static struct resource* pwmctl_cm_io_res;
static struct resource* phys_base_addr;
static struct resource* bus_base_addr;

static struct task_struct* hardware_test_task;

static dma_cookie_t dma_cookie;

#define BYTES_PER_LED		9
#define RESET_BYTES		44

#define PWM_DMA_DREQ 		5

#define BUS_ADDR_OFFSET		0xC0000000

static void pwm_enable( void )
{
  uint32_t reg;
  reg = readl(pwm_base_addr + PWM_CTL);
  reg |= PWM_CTL_PWEN1;
  writel(reg, pwm_base_addr + PWM_CTL);
}

static void pwm_disable( void )
{
  uint32_t reg;
  reg = readl(pwm_base_addr + PWM_CTL);
  reg &= ~PWM_CTL_PWEN1;
  writel(reg, pwm_base_addr + PWM_CTL);
}

//static void pwm_clear_fifo( void )
//{
//  uint32_t reg;
//  reg = readl(pwm_base_addr + PWM_CTL);
//  reg |= PWM_CTL_CLRF1;
//  writel(reg, pwm_base_addr + PWM_CTL);
//}

static void pwm_reset( void )
{
  uint32_t reg;

  writel(0, pwm_base_addr + PWM_DMAC);

  msleep(100);

  writel(32, pwm_base_addr + PWM_RNG1);
  writel(0, pwm_base_addr + PWM_DAT1);

  reg = PWM_CTL_MODE1 |
        PWM_CTL_USEF1 |
        PWM_CTL_MSEN1;

  writel(reg, pwm_base_addr + PWM_CTL);
  printk("NEOPIXEL: writing PWM CTL REGISTER =  0x%X", reg);

  reg = readl(pwm_base_addr + PWM_CTL);
  printk("NEOPIXEL: reading PWM CTL REGISTER =  0x%X", reg);

  reg = readl(pwm_base_addr + PWM_STA);
  printk("NEOPIXEL: PWM Status = 0x%X", reg);

  writel(0xFFFFFFFF, pwm_base_addr + PWM_STA);

  reg = readl(pwm_base_addr + PWM_STA);
  printk("NEOPIXEL: PWM Status = 0x%X", reg);

  reg = PWM_DMAC_ENAB |
        PWM_DMAC_PANIC(9) |
        PWM_DMAC_DREQ(9);

  writel(reg, pwm_base_addr + PWM_DMAC);
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

  pwm_reset();

  msleep(100);

  //2.5Mhz = 0,45us per bit
  reg = PWM_CM_CTL_PASSWORD | PWM_CM_DIV_DIVI(200) | PWM_CM_DIV_DIVF(492);
  writel(reg, pwmctl_cm_base_addr + PWM_CM_DIV);

  msleep(100);

  //PLLD - PLLD 500Mhz - MASH 1
  reg = PWM_CM_CTL_PASSWORD | PWM_CM_CTL_MASH(2) | PWM_CM_CTL_SRC(6);
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
  struct dma_tx_state state;
  enum dma_status status;

  int end = 0;

  status = dmaengine_tx_status(dma_chan, dma_cookie, &state);

  switch (status) {
    case DMA_IN_PROGRESS:
      //printk("NEOPIXEL(%s): Received DMA_IN_PROGRESS\n", __func__);
      break;

    case DMA_PAUSED:
      //printk("NEOPIXEL(%s): Received DMA_PAUSED\n", __func__);
      break;

    case DMA_ERROR:
      //printk("NEOPIXEL(%s): Received DMA_ERROR\n", __func__);
      end = 1;
      break;

    case DMA_COMPLETE:
      //printk("NEOPIXEL(%s): Received DMA_COMPLETE\n", __func__);
      end = 1;
      break;

    default:
      //printk("NEOPIXEL(%s): Received unknown status\n", __func__);
      end = 1;
      break;
  }

  if(end)
  {
    dma_pool_free(neo_dma_pool, dma_buffer, dma_addr);
  }

  //printk("NEOPIXEL: dma callback finished");
}

static void fill_dma_buffer( void )
{
  int i;
  volatile uint8_t* p_buffer = buffer;
  volatile uint8_t* p_dma_buffer = dma_buffer;

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
  //printk("NEOPIXEL(%s): DMA Started", __func__);

  dma_buffer = dma_pool_alloc(neo_dma_pool, GFP_KERNEL, &dma_addr);
  if(!dma_buffer)
  {
    printk("NEPIXEL(%s): No dma memory available", __func__);
    return -ENOMEM;
  }

  //printk("NEOOPIXEL(%s): dma_buffer_virt = 0x%x; dma_buffer_phys = 0x%x; dma_buffer_length = %lu", __func__, (unsigned int)dma_buffer, (unsigned int)dma_addr, buffer_len);

  fill_dma_buffer();

  dma_desc = dmaengine_prep_slave_single(dma_chan, dma_addr + BUS_ADDR_OFFSET, buffer_len, DMA_TO_DEVICE, DMA_PREP_INTERRUPT );

  if(dma_desc == NULL)
  {
    printk("NEOPIXEL(%s): Error preparing DMA transfer", __func__);
    return -EFAULT;
  }

  dma_desc->callback = neopixel_callback;
  dma_desc->callback_param = NULL;

  dma_cookie = dmaengine_submit(dma_desc);
  if (dma_submit_error(dma_cookie)) {
    printk("NEOPIXEL(%s): DMA submission failed\n", __func__);
    return -ENXIO;
  }

  dma_async_issue_pending(dma_chan);

  return 0;
}

void neopixel_pwm_set_pixel(unsigned int pixel, uint8_t red, uint8_t green, uint8_t blue)
{
  uint32_t color = ( ((uint32_t)green) << 16) | ( ((uint32_t)red) << 8) | blue;
  volatile uint8_t* buffer_ptr;
  uint8_t i, j, bits=0;

  if(pixel > num_leds) return;

  buffer_ptr = &buffer[pixel * BYTES_PER_LED];

  memset((uint8_t*)buffer_ptr, 0, BYTES_PER_LED);

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

void neopixel_clear_pixels( void )
{
  uint32_t i;
  for(i = 0; i < num_leds; i++){
    neopixel_pwm_set_pixel(i,0,0,0);
  }
  neopixel_pwm_show();
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
    .direction = DMA_MEM_TO_DEV
  };

  dev = &pdev->dev;

  phys_base_addr = platform_get_resource_byname(pdev, IORESOURCE_MEM, "neopixel-phys-base");
  if(!phys_base_addr){
    printk("NEOPIXEL(%s): phys base address not found",__func__);
    ret = -ENODEV;
    goto no_neopixel_resource;
  } else {
    printk("NEOPIXEL: phys base address 0x%lx - 0x%lx", (long unsigned int)phys_base_addr->start, (long unsigned int)phys_base_addr->end);
  }

  bus_base_addr = platform_get_resource_byname(pdev, IORESOURCE_MEM, "neopixel-bus-base");
  if(!bus_base_addr){
    printk("NEOPIXEL(%s): bus base address not found",__func__);
    ret = -ENODEV;
    goto no_neopixel_resource;
  } else {
    printk("NEOPIXEL: bus base address 0x%lx - 0x%lx", (long unsigned int)bus_base_addr->start, (long unsigned int)bus_base_addr->end);
  }

  pwm_io_res = platform_get_resource_byname(pdev, IORESOURCE_MEM, "neopixel-pwm");
  if(!pwm_io_res){
    printk("NEOPIXEL(%s): pwm base address not found",__func__);
    ret = -ENODEV;
    goto no_neopixel_pwm;
  } else {
    printk("NEOPIXEL: pwm base address 0x%lx - 0x%lx", (long unsigned int)phys_base_addr->start + pwm_io_res->start, (long unsigned int)phys_base_addr->start + pwm_io_res->end);
  }

  if  (!request_mem_region(phys_base_addr->start + pwm_io_res->start, resource_size(pwm_io_res), "neopixel-pwm")) {
    dev_err(dev, "pwm -  request_mem_region");
    printk("NEOPIXEL: pwm request region failed. Region already in use?");
    ret = -EINVAL;
    goto no_pwm_request_mem;
  }

  pwm_base_addr = ioremap(phys_base_addr->start + pwm_io_res->start, resource_size(pwm_io_res));

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
    printk("NEOPIXEL: pwmctl clock base address 0x%lx - 0x%lx", (long unsigned int)phys_base_addr->start + pwmctl_cm_io_res->start, (long unsigned int)phys_base_addr->start + pwmctl_cm_io_res->end);
  }

  /* SOME PART OF THE KERNEL IS USING THIS AREA. WE WILL USE ANYWAY, BUT, REQUESTING IT WILL FAIL */

  //if  (!request_mem_region(phys_base_addr->start + pwmctl_cm_io_res->start, resource_size(pwmctl_cm_io_res), "neopixel-pwm-cm")) {
  //  dev_err(&pdev->dev, "pwm -  request_mem_region");
  //  printk("NEOPIXEL: pwm request region failed. Region already in use?");
  //  return -EINVAL;
  //}

  pwmctl_cm_base_addr = ioremap(phys_base_addr->start + pwmctl_cm_io_res->start, resource_size(pwmctl_cm_io_res));

  if(!pwmctl_cm_base_addr){
    printk("NEOPIXEL: Error remapping pwmctl clock io memory");
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
  buffer = kzalloc(buffer_len, GFP_KERNEL | GFP_ATOMIC);
  if(buffer == NULL)
  {
    printk("Failed to allocate pwm buffer");
    goto no_buffer;
  }

  printk("NEOOPIXEL(%s): buffer_virt = 0x%x; buffer_length = %lu", __func__, (unsigned int)buffer, buffer_len);

  neo_dma_pool =  dma_pool_create("neopixel_dma", dev, buffer_len, 32, 4096);
  if(!neo_dma_pool){
    printk("NEOPIXEL(%s): Error creating dma memory pool.", __func__);
    goto no_dma_pool;
  }

  dma_chan = dma_request_slave_channel(dev, "neopixel-pwm-dma");
  if(dma_chan == NULL)
  {
    printk("Error requesting DMA channel");
    ret = -ENODEV;
    goto no_dma_request_channel;
  }

  cfg.dst_addr =  bus_base_addr->start + pwm_io_res->start + PWM_FIF1;
  if(dmaengine_slave_config(dma_chan, &cfg) < 0)
  {
    printk("Error configuring DMA\n");
    ret = -ENODEV;
    goto no_dma_config;
  }

  pwm_init();
  pwm_enable();

  neopixel_clear_pixels();

  return 0;

no_dma_config:
  dma_release_channel(dma_chan);

no_dma_request_channel:
no_dma_pool:
  kfree(buffer);

no_num_leds:
no_buffer:
  iounmap(pwmctl_cm_base_addr);

no_remap_pwm_ctl:
no_pwm_ctl_resource:
  iounmap(pwm_base_addr);

no_remap_pwm:
  release_mem_region(phys_base_addr->start + pwm_io_res->start, resource_size(pwm_io_res));

no_pwm_request_mem:
no_neopixel_pwm:
no_neopixel_resource:

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

  pwm_disable();

  iounmap(pwm_base_addr);
  iounmap(pwmctl_cm_base_addr);

  release_mem_region(phys_base_addr->start + pwm_io_res->start, resource_size(pwm_io_res));

  dma_release_channel(dma_chan);

  kfree(buffer);
  dma_pool_destroy(neo_dma_pool);

  return 0;
}

static void color_wipe(uint8_t wait, uint8_t red, uint8_t green, uint8_t blue) {
  uint16_t i;
  for(i=0; i < num_leds; i++) {
    neopixel_pwm_set_pixel(i, red, green, blue);
    neopixel_pwm_show();
    msleep(wait * 100);
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
    color_wipe(10, (stage == 0 ? 255 : 0), (stage == 1 ? 255 : 0), (stage == 2 ? 255 : 0));
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
