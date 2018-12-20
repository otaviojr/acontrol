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

#define PWM_CTL_MSEN2			(1 << 15)
#define PWM_CTL_USEF2			(1 << 13)
#define PWM_CTL_POLA2			(1 << 12)
#define PWM_CTL_SBIT2			(1 << 11)
#define PWM_CTL_RPTL2			(1 << 10)
#define PWM_CTL_MODE2			(1 << 9)
#define PWM_CTL_PWEN2			(1 << 8)
#define PWM_CTL_MSEN1			(1 << 7)
#define PWM_CTL_CLRF1			(1 << 6)
#define PWM_CTL_USEF1			(1 << 5)
#define PWM_CTL_POLA1			(1 << 4)
#define PWM_CTL_SBIT1			(1 << 3)
#define PWM_CTL_RPTL1			(1 << 2)
#define PWM_CTL_MODE1			(1 << 1)
#define PWM_CTL_PWEN1			(1 << 0)

#define PWM_DMAC_ENAB			(1 << 31)
#define PWM_DMAC_PANIC(v)		(v << 8)
#define PWM_DMAC_DREQ(v)		(v << 0)

#define PWM_CM_CTL	0x00
#define PWM_CM_DIV	0x04

#define PWM_CM_CTL_MASH(v)		(v << 9)
#define PWM_CM_CTL_FLIP			(1 << 8)
#define PWM_CM_CTL_BUSY			(1 << 7)
#define PWM_CM_CTL_KILL			(1 << 5)
#define PWM_CM_CTL_ENAB			(1 << 4)
#define PWM_CM_CTL_SRC(v)		(v << 0)

#define PWM_CM_DIV_DIVI(v)		(v << 12)
#define PWM_CM_DIV_DIVF(v)		(v << 0)

#define DMA_CS		0x00	// Control & Status register
#define DMA_CONBLK_AD	0x04	// Address of Control Block (must be 256-BYTE ALIGNED!!!)
#define DMA_TI		0x08	// Transfer Information (populated from CB)
#define DMA_SOURCE_AD	0x0C	// Source address, populated from CB. Physical address.
#define DMA_DEST_AD	0x10	// Destination address, populated from CB. Bus address.
#define DMA_TXFR_LEN	0x14	// Transfer length, populated from CB
#define DMA_STRIDE	0x18	// Stride, populated from CB
#define DMA_NEXTCONBK	0x1C	// Next control block address, populated from CB
#define DMA_DEBUG	0x20	// Debug settings

#define DMA_CS_RESET			(1<<31)
#define DMA_CS_ABORT			(1<<30)
#define DMA_CS_DISDEBUG			(1<<29)
#define DMA_CS_WAIT			(1<<28)
#define DMA_CS_PANIC_PRIORITY(v)	(v<<20)
#define DMA_CS_PRIORITY(v)		(v<<16)
#define DMA_CS_ERROR			(1<<8)
#define DMA_CS_WAITING			(1<<6)
#define DMA_CS_DREQ_STOPS		(1<<5)
#define DMA_CS_PAUSED			(1<<4)
#define DMA_CS_DREQ			(1<<3)
#define DMA_CS_INT			(1<<2)
#define DMA_CS_END			(1<<1)
#define DMA_CS_ACTIVE			(1<<0)

int neopixel_pwm_init(struct platform_device* pdev);
int neopixel_pwm_unload( void );

#endif //__NEOPIXEL_PWM_H__
