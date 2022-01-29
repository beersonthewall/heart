import subprocess
from cmd.command import Command

class BuildCmd(Command):

    def run(self):
        return subprocess.run("cd kernel && TRIPLE= make", shell=True)
