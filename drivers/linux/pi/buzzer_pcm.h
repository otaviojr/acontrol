#ifndef __BUZZER_PCM_H__
#define __BUZZER_PCM_H__

/* registers base address */
#define PCM_CS_A		0x00
#define PCM_FIFO_A		0x04
#define PCM_MODE_A		0x08
#define PCM_RXC_A		0x0C
#define PCM_TXC_A		0x10
#define PCM_DREQ_A		0x14
#define PCM_INTEN_A		0x18
#define PCM_INTSTC_A		0x1C
#define PCM_GRAY		0x20

/* pcm clt registers base address */
#define PCM_CM_CTL	0x00
#define PCM_CM_DIV	0x04

#define PCM_CM_CTL_PASSWORD		0x5A000000
#define PCM_CM_CTL_MASH(v)		(v << 9)
#define PCM_CM_CTL_FLIP			(1 << 8)
#define PCM_CM_CTL_BUSY			(1 << 7)
#define PCM_CM_CTL_KILL			(1 << 5)
#define PCM_CM_CTL_ENAB			(1 << 4)
#define PCM_CM_CTL_SRC(v)		(v << 0)

#define PCM_CM_DIV_DIVI(v)		(v << 12)
#define PCM_CM_DIV_DIVF(v)		(v << 0)

int buzzer_pcm_load(struct platform_device* pdev);
int buzzer_pcm_unload( void );

#endif //__BUZZER_PCM_H__
