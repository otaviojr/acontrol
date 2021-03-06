/**
 * @file   buzzer_pcm.h
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

#ifndef __BUZZER_PCM_H__
#define __BUZZER_PCM_H__

/* registers base address */
#define PCM_CS_A		            0x00
#define PCM_FIFO_A		          0x04
#define PCM_MODE_A		          0x08
#define PCM_RXC_A		            0x0C
#define PCM_TXC_A		            0x10
#define PCM_DREQ_A		          0x14
#define PCM_INTEN_A		          0x18
#define PCM_INTSTC_A	          0x1C
#define PCM_GRAY		            0x20

#define PCM_CS_A_STBY			      (1 << 25)
#define PCM_CS_A_SYNC			      (1 << 24)
#define PCM_CS_A_RXSEX		      (1 << 23)
#define PCM_CS_A_RXF			      (1 << 22)
#define PCM_CS_A_TXE			      (1 << 21)
#define PCM_CS_A_RXD			      (1 << 20)
#define PCM_CS_A_TXD			      (1 << 19)
#define PCM_CS_A_RXR			      (1 << 18)
#define PCM_CS_A_TXW			      (1 << 17)
#define PCM_CS_A_RXERR		      (1 << 16)
#define PCM_CS_A_TXERR		      (1 << 15)
#define PCM_CS_A_RXSYNC		      (1 << 14)
#define PCM_CS_A_TXSYNC		      (1 << 13)
#define PCM_CS_A_DMAEN		      (1 << 9)
#define PCM_CS_A_RXTHR(v)	      (v << 7)
#define PCM_CS_A_TXTHR(v)	      (v << 5)
#define PCM_CS_A_RXCLR          (1 << 4)
#define PCM_CS_A_TXCLR          (1 << 3)
#define PCM_CS_A_TXON           (1 << 2)
#define PCM_CS_A_RXON           (1 << 1)
#define PCM_CS_A_EN             (1 << 0)

#define PCM_MODE_A_CLK_DIS      (1 << 28)
#define PCM_MODE_A_PDMN         (1 << 27)
#define PCM_MODE_A_PDME         (1 << 26)
#define PCM_MODE_A_FRXP         (1 << 25)
#define PCM_MODE_A_FTXP         (1 << 24)
#define PCM_MODE_A_CLKM         (1 << 23)
#define PCM_MODE_A_CLKI         (1 << 22)
#define PCM_MODE_A_FSM          (1 << 21)
#define PCM_MODE_A_FSI          (1 << 20)
#define PCM_MODE_A_FLEN(v)      (v << 10)
#define PCM_MODE_A_FSLEN(v)     (v << 0)

#define PCM_RXC_A_CH1WEX        (1 << 31)
#define PCM_RXC_A_CH1EN         (1 << 30)
#define PCM_RXC_A_CH1POS(v)     (v << 20)
#define PCM_RXC_A_CH1WID(v)     (v << 16)
#define PCM_RXC_A_CH2WEX        (1 << 15)
#define PCM_RXC_A_CH2EN         (1 << 14)
#define PCM_RXC_A_CH2POS(v)     (v << 4)
#define PCM_RXC_A_CH2WID(v)     (v << 0)

#define PCM_TXC_A_CH1WEX        (1 << 31)
#define PCM_TXC_A_CH1EN         (1 << 30)
#define PCM_TXC_A_CH1POS(v)     (v << 20)
#define PCM_TXC_A_CH1WID(v)     (v << 16)
#define PCM_TXC_A_CH2WEX        (1 << 15)
#define PCM_TXC_A_CH2EN         (1 << 14)
#define PCM_TXC_A_CH2POS(v)     (1 << 4)
#define PCM_TXC_A_CH2WID(v)     (v << 0)

#define PCM_DREQ_A_TX_PANIC(v)  (v << 24)
#define PCM_DREQ_A_RX_PANIC(v)  (v << 16)
#define PCM_DREQ_A_TX(v)        (v << 8)
#define PCM_DREQ_A_RX(v)        (v << 0)

#define PCM_INTEN_A_RXERR       (1 << 3)
#define PCM_INTEN_A_TXERR       (1 << 2)
#define PCM_INTEN_A_RXR         (1 << 1)
#define PCM_INTEN_A_TXW         (1 << 0)

#define PCM_INTSTC_A_RXERR      (1 << 3)
#define PCM_INTSTC_A_TXERR      (1 << 2)
#define PCM_INTSTC_A_RXR        (1 << 1)
#define PCM_INTSTC_A_TXW        (1 << 0)

/* pcm clt registers base address */
#define PCM_CM_CTL	0x00
#define PCM_CM_DIV	0x04

#define PCM_CM_CTL_PASSWORD		  0x5A000000
#define PCM_CM_CTL_MASH(v)		  (v << 9)
#define PCM_CM_CTL_FLIP			    (1 << 8)
#define PCM_CM_CTL_BUSY			    (1 << 7)
#define PCM_CM_CTL_KILL			    (1 << 5)
#define PCM_CM_CTL_ENAB			    (1 << 4)
#define PCM_CM_CTL_SRC(v)		    (v << 0)

#define PCM_CM_DIV_DIVI(v)		  (v << 12)
#define PCM_CM_DIV_DIVF(v)		  (v << 0)

int buzzer_pcm_load(struct platform_device* pdev);
int buzzer_pcm_unload( void );
int buzzer_pcm_play_tone(struct buzzer_tone* tone);

#endif //__BUZZER_PCM_H__
