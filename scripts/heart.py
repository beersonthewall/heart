#!/bin/python3

import argparse
from cmd.build import BuildCmd
from cmd.run import RunCmd

def main(args):
    opts = args.opts[0]
    print(opts)
    if opts == 'b' or opts == 'build':
        BuildCmd().run()
    elif opts == 'r' or opts == 'run':
        RunCmd().run()
    else:
        raise Exception()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="Heart OS util.")
    parser.add_argument('opts', nargs=1, choices=['build', 'b', 'run', 'r'])
    main(parser.parse_args())

