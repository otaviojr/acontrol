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

#include <linux/io.h>

#include "neopixel_pwm.h"

static void* __iomem dma_base_addr;
static void* __iomem pwm_base_addr;

int neopixel_pwm_init( void* __iomem dma_addr, void* __iomem pwm_addr)
{
  u32 v32;

  dma_base_addr = dma_addr;
  pwm_base_addr = pwm_addr;

  //stop all DMA operations
  v32 = readl(dma_base_addr + DMA_CS);
  writel(v32 | DMA_CS_ABORT | DMA_CS_RESET, dma_base_addr + DMA_CS);

  return 0;
}

int neopixel_pwm_unload( void )
{
  return 0;
}
