QEMU = qemu-system-x86_64

.PHONY: run debug build

run: build
	$(QEMU) -kernel kernel.amd64.bin -serial stdio -no-reboot

debug: build
	$(QEMU) -d int -s -S -kernel kernel.amd64.bin -serial stdio -no-reboot

build:
	TRIPLE= $(MAKE) -C kernel

clean:
	@rm kernel.amd64*
	$(MAKE) -C kernel clean
