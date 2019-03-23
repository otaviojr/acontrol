#ifndef __BUZZER_PCM_H__
#define __BUZZER_PCM_H__

/* registers base address */
//TODO: configure registers here

int buzzer_pcm_init(struct platform_device* pdev);
int buzzer_pcm_unload( void );

#endif //__BUZZER_PCM_H__
