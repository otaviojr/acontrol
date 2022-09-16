#include <linux/module.h>
#define INCLUDE_VERMAGIC
#include <linux/build-salt.h>
#include <linux/vermagic.h>
#include <linux/compiler.h>

BUILD_SALT;

MODULE_INFO(vermagic, VERMAGIC_STRING);
MODULE_INFO(name, KBUILD_MODNAME);

__visible struct module __this_module
__section(".gnu.linkonce.this_module") = {
	.name = KBUILD_MODNAME,
	.init = init_module,
#ifdef CONFIG_MODULE_UNLOAD
	.exit = cleanup_module,
#endif
	.arch = MODULE_ARCH_INIT,
};

#ifdef CONFIG_RETPOLINE
MODULE_INFO(retpoline, "Y");
#endif

static const struct modversion_info ____versions[]
__used __section("__versions") = {
	{ 0x647d912, "module_layout" },
	{ 0xae9849dd, "__request_region" },
	{ 0x8de527fa, "cdev_del" },
	{ 0x2d6fcc06, "__kmalloc" },
	{ 0x8e5ae04c, "cdev_init" },
	{ 0xf9a482f9, "msleep" },
	{ 0xee4d1bfd, "sg_init_table" },
	{ 0xd4c825af, "mem_map" },
	{ 0xc94d8e3b, "iomem_resource" },
	{ 0xf7802486, "__aeabi_uidivmod" },
	{ 0xa262c430, "device_destroy" },
	{ 0xb1ad28e0, "__gnu_mcount_nc" },
	{ 0xae353d77, "arm_copy_from_user" },
	{ 0x6091b333, "unregister_chrdev_region" },
	{ 0xa4f65c, "dma_request_chan" },
	{ 0xe424b209, "__platform_driver_register" },
	{ 0x51a910c0, "arm_copy_to_user" },
	{ 0xe707d823, "__aeabi_uidiv" },
	{ 0x5f754e5a, "memset" },
	{ 0xc5850110, "printk" },
	{ 0xe97c4103, "ioremap" },
	{ 0xa1c76e0a, "_cond_resched" },
	{ 0xe681d566, "device_create" },
	{ 0x4c0dcead, "_dev_err" },
	{ 0xc7684a86, "cdev_add" },
	{ 0x86332725, "__stack_chk_fail" },
	{ 0xd8b03092, "dma_map_page_attrs" },
	{ 0x58e0606c, "dev_driver_string" },
	{ 0x4384eb42, "__release_region" },
	{ 0xbfe34ca9, "platform_get_resource_byname" },
	{ 0x2cfde9a2, "warn_slowpath_fmt" },
	{ 0x739d4436, "dma_release_channel" },
	{ 0x37a0cba, "kfree" },
	{ 0xedc03953, "iounmap" },
	{ 0xf9116977, "class_destroy" },
	{ 0x7c274f38, "dma_unmap_page_attrs" },
	{ 0x8f678b07, "__stack_chk_guard" },
	{ 0x2107e0db, "platform_driver_unregister" },
	{ 0xa0058d45, "__class_create" },
	{ 0xe3ec2f2b, "alloc_chrdev_region" },
	{ 0xc31db0ce, "is_vmalloc_addr" },
};

MODULE_INFO(depends, "");

MODULE_ALIAS("of:N*T*Cbcm2835-buzzer");
MODULE_ALIAS("of:N*T*Cbcm2835-buzzerC*");

MODULE_INFO(srcversion, "2AE0A551C117B8A4BD62309");
