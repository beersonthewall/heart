import subprocess
from cmd.command import Command

class BuildCmd(Command):

    def run(self):
        subprocess.run("cd kernel && TRIPLE= make", shell=True)
