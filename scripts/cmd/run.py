import subprocess
from cmd.command import Command
from cmd.build import BuildCmd

class RunCmd(Command):

    def run(self):
        BuildCmd().run()
        subprocess.run("qemu-system-x86_64 -kernel kernel.amd64.bin -serial stdio", shell=True)
