/**
 * @file   neopixek_drv.h
 * @author Otavio Ribeiro
 * @date   16 Dec 2018
 * @brief  A kernel module for controlling neopixel strip
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

#ifndef __NEOPIXEL_DRV_H__
#define __NEOPIXEL_DRV_H__

  #define FIRST_MINOR       1
  #define MINOR_CNT         3

  struct neopixel_pixel {
    unsigned long pixel;
    unsigned char red;
    unsigned char green;
    unsigned char blue;
  };

  #define DEV_MODE false
  #define DEBUG(fmt,...) if(DEV_MODE){\
    do {\
      printk(fmt, ##__VA_ARGS__);\
    } while (0);\
  }

#endif //__NEOPIXEK_DRV_H__
