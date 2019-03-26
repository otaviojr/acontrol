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

int buzzer_pcm_init(struct platform_device* pdev);
int buzzer_pcm_unload( void );

#endif //__BUZZER_PCM_H__
