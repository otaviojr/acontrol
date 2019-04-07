/**
 * @file   buzzer_pcm.c
 * @author Otavio Ribeiro
 * @date   23 Mar 2019
 * @brief  Buzzer PCM linux kernel device driver
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

#include "buzzer_drv.h"
#include "buzzer_pcm.h"

static volatile void* __iomem pcm_base_addr;
static volatile void* __iomem pcmctl_cm_base_addr;

static uint8_t* buffer = NULL;

static unsigned long buffer_len;

static struct device* dev;
static struct dma_chan* dma_chan;
static dma_addr_t dma_addr;
static struct dma_async_tx_descriptor * dma_desc;

static struct resource* pcm_io_res;
static struct resource* pcmctl_cm_io_res;
static struct resource* phys_base_addr;
static struct resource* bus_base_addr;

static dma_cookie_t dma_cookie;

#define PCM_DMA_DREQ 		  2
#define BUS_ADDR_OFFSET		0xC0000000

#define PCM_FREQUENCY     125000

static int pcm_clock_init( void )
{
  uint32_t reg, counter;

  //disable pcm clock
  reg = PCM_CM_CTL_PASSWORD | PCM_CM_CTL_KILL;
  writel(reg, pcmctl_cm_base_addr + PCM_CM_CTL);

  msleep(100);

  counter = 0;
  while( ((reg = readl(pcmctl_cm_base_addr + PCM_CM_CTL)) & PCM_CM_CTL_BUSY) != 0)
  {
    msleep(100);
    printk("Waiting pcm busy bit: 0x%X\n", reg);
    if(counter++ == 100){
      printk("Timeout waiting busy bit.\n");
      return -1;
    }
  }

  msleep(100);

  //125Khz = 1 bit every 8*10^-6 seconds
  reg = PCM_CM_CTL_PASSWORD | PCM_CM_DIV_DIVI(100) | PCM_CM_DIV_DIVF(0);
  writel(reg, pcmctl_cm_base_addr + PCM_CM_DIV);

  msleep(100);

  //PLLD - PLLD 500Mhz - MASH 0
  reg = PCM_CM_CTL_PASSWORD | PCM_CM_CTL_MASH(0) | PCM_CM_CTL_SRC(6);
  writel(reg, pcmctl_cm_base_addr + PCM_CM_CTL);
  printk("writing PCM_CM_CTL=0x%X",reg);

  msleep(100);

  reg = readl(pcmctl_cm_base_addr + PCM_CM_CTL);
  printk("reading PCM_CM_CTL=0x%X", reg);

  reg |= PCM_CM_CTL_PASSWORD | PCM_CM_CTL_ENAB;

  writel(reg, pcmctl_cm_base_addr + PCM_CM_CTL);
  printk("writing PCM_CM_CTL=0x%X",reg);

  msleep(100);

  reg = readl(pcmctl_cm_base_addr + PCM_CM_CTL);
  printk("reading PCM_CM_CTL=0x%X", reg);

  return 0;
}

static int pcm_init( void ) 
{
  uint32_t reg;

  //Disable clock before change the source
  reg = readl(pcm_base_addr + PCM_MODE_A);
  reg |= PCM_MODE_A_CLK_DIS;
  writel(reg, pcm_base_addr + PCM_MODE_A);

  msleep(100);

  //configure pcm clock source and parameters
  if(pcm_clock_init() < 0) {
    return -1;
  }

  reg = PCM_MODE_A_FLEN(127) | //A frame is around 1ms of buzzer audio
        PCM_MODE_A_FSLEN(127);
  writel(reg, pcm_base_addr + PCM_MODE_A);

  msleep(100);

  reg = 0;
  writel(reg, pcm_base_addr + PCM_RXC_A);

  msleep(100);

  reg = PCM_TXC_A_CH1EN;
  writel(reg, pcm_base_addr + PCM_TXC_A);

  msleep(100);

  reg = PCM_DREQ_A_TX_PANIC(10) |
        PCM_DREQ_A_TX(63);
  writel(reg, pcm_base_addr + PCM_DREQ_A);

  msleep(100);

  reg = 0;
  writel(reg, pcm_base_addr + PCM_INTEN_A);

  msleep(100);

  reg = PCM_CS_A_STBY |
        PCM_CS_A_DMAEN |
        PCM_CS_A_RXCLR |
        PCM_CS_A_TXCLR |
        PCM_CS_A_TXON |
        PCM_CS_A_EN;
  writel(reg, pcm_base_addr + PCM_CS_A);

  msleep(100);

  return 0;
}

static void buzzer_callback(void * param)
{
  struct dma_tx_state state;
  enum dma_status status;

  int end = 0;

  status = dmaengine_tx_status(dma_chan, dma_cookie, &state);

  switch (status) {
    case DMA_IN_PROGRESS:
      printk("BUZZER(%s): Received DMA_IN_PROGRESS\n", __func__);
      break;

    case DMA_PAUSED:
      printk("BUZZER(%s): Received DMA_PAUSED\n", __func__);
      break;

    case DMA_ERROR:
      printk("BUZZER(%s): Received DMA_ERROR\n", __func__);
      end = 1;
      break;

    case DMA_COMPLETE:
      printk("BUZZER(%s): Received DMA_COMPLETE\n", __func__);
      end = 1;
      break;

    default:
      printk("BUZZER(%s): Received unknown status\n", __func__);
      end = 1;
      break;
  }

  if(end)
  {
    dma_unmap_single(dev, dma_addr, buffer_len, DMA_TO_DEVICE);
    dma_addr = 0;
  }

  printk("BUZZER: dma callback finished");
}

static int start_dma( void )
{
  printk("BUZZER(%s): DMA Started", __func__);

  if(dma_addr != 0){
    dma_unmap_single(dev, dma_addr, buffer_len, DMA_TO_DEVICE);
  }

  dma_addr = dma_map_single(dev, buffer, buffer_len, DMA_TO_DEVICE);
  if(dma_mapping_error(dev, dma_addr))
  {
    printk("BUZZER(%s): No dma memory available", __func__);
    return -ENOMEM;
  }

  printk("BUZZER(%s): dma_buffer_virt = 0x%x; dma_buffer_phys = 0x%x; dma_buffer_length = %lu", 
					__func__, 
					(unsigned int)buffer, 
					(unsigned int)dma_addr, 
					buffer_len);

  dma_desc = dmaengine_prep_slave_single(dma_chan, dma_addr + BUS_ADDR_OFFSET, buffer_len, DMA_TO_DEVICE, DMA_PREP_INTERRUPT );

  if(dma_desc == NULL)
  {
    printk("BUZZER(%s): Error preparing DMA transfer", __func__);
    return -EFAULT;
  }

  dma_desc->callback = buzzer_callback;
  dma_desc->callback_param = NULL;

  dma_cookie = dmaengine_submit(dma_desc);
  if (dma_submit_error(dma_cookie)) {
    printk("BUZZER(%s): DMA submission failed\n", __func__);
    return -ENXIO;
  }

  dma_async_issue_pending(dma_chan);

  return 0;
}

int buzzer_pcm_play_tone(struct buzzer_tone* tone) {

  unsigned long i;
  unsigned int out = 0xFF;
  unsigned long bytes = (PCM_FREQUENCY / tone->freq)/8;

  printk("BUZZER(%s): playing tone, switching after %lu bytes\n", __func__, bytes);

  dmaengine_terminate_sync(dma_chan);

  if(buffer != NULL) {
    kfree(buffer);
  }

  // we multiply for 10 to get the right period
  // for any unknown reason. Stoping with math... 
  // starting tunning by using heuristic algorithm.... 3 a.m and 
  // I have to  work tomorow
  buffer_len = (((PCM_FREQUENCY/1000) * tone->period)/8) *10; 

  printk("BUZZER(%s): Playing tone buffer length: %lu", __func__, buffer_len);

  buffer = kzalloc(buffer_len, GFP_KERNEL | GFP_ATOMIC);
  if(buffer == NULL) {
    printk("Failed to allocate pcm buffer\n");
    return -1;
  }

  for(i = 0; i < buffer_len; i++) {
    buffer[i] = out;

    if(i % bytes == 0){
      out = (out == 0xFF ? 0x00 : 0xFF);
    }
  }

  start_dma();

  return 0;
}

int buzzer_pcm_load( struct platform_device *pdev )
{
  int ret = 0;

  //struct device_node *np = pdev->dev.of_node;

  struct dma_slave_config cfg = {
    .src_addr_width = DMA_SLAVE_BUSWIDTH_4_BYTES,
    .dst_addr_width = DMA_SLAVE_BUSWIDTH_4_BYTES,
    .slave_id = PCM_DMA_DREQ,
    .direction = DMA_MEM_TO_DEV
  };

  dev = &pdev->dev;

  phys_base_addr = platform_get_resource_byname(pdev, IORESOURCE_MEM, "buzzer-phys-addr");
  if(!phys_base_addr){
    printk("BUZZER(%s): phys base address not found",__func__);
    ret = -ENODEV;
    goto no_buzzer_resource;
  } else {
    printk("BUZZER(%s): phys base address 0x%lx - 0x%lx", __func__, (long unsigned int)phys_base_addr->start, (long unsigned int)phys_base_addr->end);
  }

  bus_base_addr = platform_get_resource_byname(pdev, IORESOURCE_MEM, "buzzer-bus-addr");
  if(!bus_base_addr){
    printk("BUZZER(%s): bus base address not found",__func__);
    ret = -ENODEV;
    goto no_buzzer_resource;
  } else {
    printk("BUZZER(%s): bus base address 0x%lx - 0x%lx", __func__, (long unsigned int)bus_base_addr->start, (long unsigned int)bus_base_addr->end);
  }

  pcm_io_res = platform_get_resource_byname(pdev, IORESOURCE_MEM, "buzzer-pcm");
  if(!pcm_io_res){
    printk("BUZZER: pcm base address not found");
    ret = -ENODEV;
    goto no_buzzer_pcm;
  } else {
    printk("BUZZER: pcm base address 0x%lx - 0x%lx", (long unsigned int)phys_base_addr->start + pcm_io_res->start, (long unsigned int)phys_base_addr->start + pcm_io_res->end);
  }

  if(!request_mem_region(phys_base_addr->start + pcm_io_res->start, resource_size(pcm_io_res), "buzzer-pcm")) {
    dev_err(dev, "pcm -  request_mem_region");
    printk("BUZZER: pcm request region failed. Region already in use?");
    ret = -EINVAL;
    goto no_pcm_request_mem;
  }

  pcm_base_addr = ioremap(phys_base_addr->start + pcm_io_res->start, resource_size(pcm_io_res));
  if(!pcm_base_addr){
    printk("BUZZER: Error remapping pcm io memory");
    ret = -ENOMEM;
    goto no_remap_pcm;
  } else {
    printk("BUZZER: PCM address remapped");
  }

  pcmctl_cm_io_res = platform_get_resource_byname(pdev, IORESOURCE_MEM, "buzzer-pcmctl-cm");
  if(!pcmctl_cm_io_res){
    printk("BUZZER: pcmctl clock base address not found");
    ret = -ENODEV;
    goto no_pcm_ctl_resource;
  } else {
    printk("BUZZER: pcmctl clock base address 0x%lx - 0x%lx", (long unsigned int)phys_base_addr->start + pcmctl_cm_io_res->start, (long unsigned int)phys_base_addr->start + pcmctl_cm_io_res->end);
  }

  /*if(!request_mem_region(phys_base_addr->start + pcmctl_cm_io_res->start, resource_size(pcmctl_cm_io_res), "buzzer-pcmctl")) {
    dev_err(dev, "pcm cm-  request_mem_region");
    printk("BUZZER: pcm c, request region failed. Region already in use?");
    ret = -EINVAL;
    goto no_pcm_ctl_request_mem;
  }*/

  pcmctl_cm_base_addr = ioremap(phys_base_addr->start + pcmctl_cm_io_res->start, resource_size(pcmctl_cm_io_res));
  if(!pcmctl_cm_base_addr){
    printk("BUZZER: Error remapping pcmctl clock io memory");
    ret = -ENOMEM;
    goto no_remap_pcm_ctl;
  } else {
    printk("BUZZER: PCMCTL clock address remapped");
  }

  dma_chan = dma_request_slave_channel(dev, "buzzer-pcm-dma");
  if(!dma_chan) {
    printk("BUZZER(%s): Error requesting DMA channel", __func__);
    ret = -ENODEV;
    goto no_dma_request_channel;
  }

  cfg.dst_addr =  bus_base_addr->start + pcm_io_res->start + PCM_FIFO_A;
  if(dmaengine_slave_config(dma_chan, &cfg) < 0) {
    printk("BUZZER(%s): Error configuring DMA\n", __func__);
    ret = -ENODEV;
    goto no_dma_config;
  }

  if(pcm_init() < 0){
    printk("BUZZER(%s): Error configuring PCM device\n", __func__);
    ret = -ENODEV;
    goto pcm_init_error;
  }

  return 0;

pcm_init_error:
no_dma_config:
  dma_release_channel(dma_chan);

no_dma_request_channel:
  iounmap(pcmctl_cm_base_addr);

no_remap_pcm_ctl:
  //release_mem_region(phys_base_addr->start + pcmctl_cm_io_res->start, resource_size(pcmctl_cm_io_res));

no_pcm_ctl_resource:
  iounmap(pcm_base_addr);

no_remap_pcm:
  release_mem_region(phys_base_addr->start + pcm_io_res->start, resource_size(pcm_io_res));

no_pcm_request_mem:
no_buzzer_pcm:
no_buzzer_resource:

  return ret;
}

int buzzer_pcm_unload( void )
{
  dmaengine_terminate_async(dma_chan);
  dmaengine_synchronize(dma_chan);

  iounmap(pcm_base_addr);
  iounmap(pcmctl_cm_base_addr);

  release_mem_region(phys_base_addr->start + pcm_io_res->start, resource_size(pcm_io_res));
  //release_mem_region(phys_base_addr->start + pcmctl_cm_io_res->start, resource_size(pcmctl_cm_io_res));

  dma_release_channel(dma_chan);

  if(buffer != NULL){
    kfree(buffer);
  }

  return 0;
}
