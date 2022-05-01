QEMU = qemu-system-x86_64

.PHONY: run debug build

run: kernel.amd64.bin
	$(QEMU) -d int -kernel kernel.amd64.bin -serial stdio -no-reboot

debug: kernel.amd64.bin
	$(QEMU) -d int -s -S -kernel kernel.amd64.bin -serial stdio -no-reboot

build: kernel.amd64.bin

kernel.amd64.bin:
	TRIPLE= $(MAKE) -C kernel

clean:
	@rm kernel.amd64*
	$(MAKE) -C kernel clean
