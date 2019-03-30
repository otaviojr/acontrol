/**
 * @file   neopixel_ioctl.h
 * @author Otavio Ribeiro
 * @date   16 Dec 2018
 * @brief  Kernel IOCTL definitions
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

#ifndef __NEOPIXEL_IOCTL_H__
#define __NEOPIXEL_IOCTL_H__

  #define NEOPIXEL_IOC_MAGIC                  'N'

  #define NEOPIXEL_IOCTL_GET_VERSION          _IOR(NEOPIXEL_IOC_MAGIC, 1, unsigned char[6])
  #define NEOPIXEL_IOCTL_GET_NUM_LEDS	        _IOR(NEOPIXEL_IOC_MAGIC, 2, long*)
  #define NEOPIXEL_IOCTL_SET_PIXEL            _IOW(NEOPIXEL_IOC_MAGIC, 3, struct neopixel_pixel*)
  #define NEOPIXEL_IOCTL_SHOW		              _IOR(NEOPIXEL_IOC_MAGIC, 4, long*)
  #define NEOPIXEL_IOCTL_HARDWARE_TEST        _IOR(NEOPIXEL_IOC_MAGIC, 5, long*)
  //#define OTAROVER_IOCTL_SET_M_ENABLE       _IOW(OTAROVER_IOC_MAGIC, 2, long*)
  //#define OTAROVER_IOCTL_GET_M1_SPEED       _IOR(OTAROVER_IOC_MAGIC, 3, long*)
  //#define OTAROVER_IOCTL_SET_M1_SPEED       _IOW(OTAROVER_IOC_MAGIC, 4, long*)
  //#define OTAROVER_IOCTL_GET_M2_SPEED       _IOR(OTAROVER_IOC_MAGIC, 5, long*)
  //#define OTAROVER_IOCTL_SET_M2_SPEED       _IOW(OTAROVER_IOC_MAGIC, 6, long*)
  //#define OTAROVER_IOCTL_GET_M1_DIRECTION   _IOR(OTAROVER_IOC_MAGIC, 7, long*)
  //#define OTAROVER_IOCTL_SET_M1_DIRECTION   _IOW(OTAROVER_IOC_MAGIC, 8, long*)
  //#define OTAROVER_IOCTL_GET_M2_DIRECTION   _IOR(OTAROVER_IOC_MAGIC, 9, long*)
  //#define OTAROVER_IOCTL_SET_M2_DIRECTION   _IOW(OTAROVER_IOC_MAGIC, 10, long*)
  //#define OTAROVER_IOCTL_GET_M1_CONFIG      _IOR(OTAROVER_IOC_MAGIC, 11, long*)
  //#define OTAROVER_IOCTL_SET_M1_CONFIG      _IOW(OTAROVER_IOC_MAGIC, 12, long*)
  //#define OTAROVER_IOCTL_GET_M2_CONFIG      _IOR(OTAROVER_IOC_MAGIC, 13, long*)
  //#define OTAROVER_IOCTL_SET_M2_CONFIG      _IOW(OTAROVER_IOC_MAGIC, 14, long*)
  //#define OTAROVER_IOCTL_CALIBRATE_SENSORS  _IOR(OTAROVER_IOC_MAGIC, 15, long*)
  //#define OTAROVER_IOCTL_GET_SENSOR_OFFSETS _IOR(OTAROVER_IOC_MAGIC, 16, long*)
  //#define OTAROVER_IOCTL_SET_SENSOR_OFFSETS _IOR(OTAROVER_IOC_MAGIC, 17, long*)
  //#define OTAROVER_IOCTL_READ_SENSORS       _IOR(OTAROVER_IOC_MAGIC, 18, long*)
  #define NEOPIXEL_IOCTL_MAX_CMD            5

  //#define OTAROVER_IOCTL_DC_MOTOR_ENABLE    1
  //#define OTAROVER_IOCTL_DC_MOTOR_DISABLE   0

  //#define OTAROVER_IOCTL_DIR_FORWARD        1
  //#define OTAROVER_IOCTL_DIR_STOPPED        0
  //#define OTAROVER_IOCTL_DIR_BACKWARD       -1

  //#define OTAROVER_IOCTL_CONFIG_NORMAL      1
  //#define OTAROVER_IOCTL_CONFIG_REVERSE     -1

#endif //__NEOPIXEL_IOCTL_H__
