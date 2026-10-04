#!/usr/bin/env python3
"""Mirror of tgtdb.py Tgt.getRect for test parity (no CONFIG_DIR / tgt.dat load)."""

import math
import sys


def get_rect(length, width, height, aspect, range_m, hfov, vfov, image_width, image_height):
    hfov = hfov * (math.pi / 180.0)
    asp_ang = aspect * math.pi / 180.0
    tgt_width = math.fabs(length * math.sin(asp_ang)) + math.fabs(width * math.cos(asp_ang))
    pix_width = tgt_width * image_width / (range_m * hfov)

    vfov = vfov * (math.pi / 180.0)
    pix_height = height * image_height / (range_m * vfov)
    return pix_width, pix_height


def main() -> None:
    if len(sys.argv) != 10:
        print("usage: tgtdb_get_rect.py L W H aspect range hfov vfov imw imh", file=sys.stderr)
        sys.exit(2)
    vals = [float(x) for x in sys.argv[1:]]
    pw, ph = get_rect(*vals)
    print(f"{pw} {ph}")


if __name__ == "__main__":
    main()
