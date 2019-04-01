/**
 * @file   neopixel_pwm.h
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

#ifndef __NEOPIXEL_PWM_H__
#define __NEOPIXEL_PWM_H__

/* registers base address */
#define	PWM_CTL  0x00	// Control Register
#define PWM_STA  0x04	// Status Register
#define PWM_DMAC 0x08	// DMA Control Register
#define PWM_RNG1 0x10	// Channel 1 Range
#define PWM_DAT1 0x14	// Channel 1 Data
#define PWM_FIF1 0x18	// FIFO (for both channels - bytes are interleaved if both active)
#define PWM_RNG2 0x20	// Channel 2 Range
#define PWM_DAT2 0x24	// Channel 2 Data

#define PWM_CTL_MSEN2			    (1 << 15)
#define PWM_CTL_USEF2			    (1 << 13)
#define PWM_CTL_POLA2			    (1 << 12)
#define PWM_CTL_SBIT2			    (1 << 11)
#define PWM_CTL_RPTL2			    (1 << 10)
#define PWM_CTL_MODE2			    (1 << 9)
#define PWM_CTL_PWEN2			    (1 << 8)
#define PWM_CTL_MSEN1			    (1 << 7)
#define PWM_CTL_CLRF1			    (1 << 6)
#define PWM_CTL_USEF1			    (1 << 5)
#define PWM_CTL_POLA1			    (1 << 4)
#define PWM_CTL_SBIT1			    (1 << 3)
#define PWM_CTL_RPTL1			    (1 << 2)
#define PWM_CTL_MODE1			    (1 << 1)
#define PWM_CTL_PWEN1			    (1 << 0)

#define PWM_DMAC_ENAB			    (1 << 31)
#define PWM_DMAC_PANIC(v)		  (v << 8)
#define PWM_DMAC_DREQ(v)		  (v << 0)

#define PWM_CM_CTL	0x00
#define PWM_CM_DIV	0x04

#define PWM_CM_CTL_PASSWORD		0x5A000000
#define PWM_CM_CTL_MASH(v)		(v << 9)
#define PWM_CM_CTL_FLIP			  (1 << 8)
#define PWM_CM_CTL_BUSY			  (1 << 7)
#define PWM_CM_CTL_KILL			  (1 << 5)
#define PWM_CM_CTL_ENAB			  (1 << 4)
#define PWM_CM_CTL_SRC(v)		  (v << 0)

#define PWM_CM_DIV_DIVI(v)		(v << 12)
#define PWM_CM_DIV_DIVF(v)		(v << 0)

int neopixel_pwm_init(struct platform_device* pdev);
int neopixel_pwm_unload( void );
void neopixel_pwm_set_pixel(unsigned int pixel, uint8_t red, uint8_t green, uint8_t blue);
int neopixel_pwm_show( void );
int neopixel_pwm_get_num_leds( void );
int neopixel_pwm_hardware_test( void );
int neopixel_pwm_stop( void );

#endif //__NEOPIXEL_PWM_H__
