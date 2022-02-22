import subprocess
from cmd.command import Command
from cmd.build import BuildCmd

class DebugCmd(Command):

    def run(self):
        result = BuildCmd().run()
        if result.returncode:
            return 

        return subprocess.run("qemu-system-x86_64 -d int -s -S -kernel kernel.amd64.bin -serial stdio -no-reboot", shell=True)
