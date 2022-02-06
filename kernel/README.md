# Heart

## Memory Map

### Physical Address Space

0x0000000000000000 -> Null pointer always set to not present


0xf4240 (1MB)      -> likely start userspace here


0xffffffff80000000 -> kernel starts (-2GB)
                      kernel_stack_base

0xffffffffffffffff -> kernel_heap_base
