#ifndef __NEOPIXEL_PWM_H__
#define __NEOPIXEL_PWM_H__

/* registers base address */
#define IO_BASE			0x3F000000
#define IO_LEN			0xFFFFFF
#define PWM_OFFSET		0x0020C000
#define DMA_OFFSET		0x00007000
#define CLK_OFFSET	        0x00101000
#define GPIO_OFFSET		0x00200000

#define	PWM_CTL  0x00	// Control Register
#define PWM_STA  0x04	// Status Register
#define PWM_DMAC 0x08	// DMA Control Register
#define PWM_RNG1 0x10	// Channel 1 Range
#define PWM_DAT1 0x14	// Channel 1 Data
#define PWM_FIF1 0x18	// FIFO (for both channels - bytes are interleaved if both active)
#define PWM_RNG2 0x20	// Channel 2 Range
#define PWM_DAT2 0x24	// Channel 2 Data

#define DMA_CHANNEL_0	0x000
#define DMA_CHANNEL_1	0x100
#define DMA_CHANNEL_2	0x200
#define DMA_CHANNEL_3	0x300
#define DMA_CHANNEL_4	0x400
#define DMA_CHANNEL_5	0x500
#define DMA_CHANNEL_6	0x600
#define DMA_CHANNEL_7	0x700
#define DMA_CHANNEL_8	0x800
#define DMA_CHANNEL_9	0x900
#define DMA_CHANNEL_10	0xa00
#define DMA_CHANNEL_11	0xb00
#define DMA_CHANNEL_12	0xc00
#define DMA_CHANNEL_13	0xd00
#define DMA_CHANNEL_14	0xe00

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

int neopixel_pwm_init( void* __iomem base_addr );
int neopixel_pwm_unload( void );

#endif //__NEOPIXEL_PWM_H__
