#!/usr/bin/env python3
"""Check which of our font files are the builds Word embedded in reference PDFs.

    tools/font_builds.py <reference.pdf>... -- <font-file-or-dir>...

A reference PDF embeds a subset of every font Word used, with the subset's
head, hhea and OS/2 tables intact and the original glyph ids kept. For each
embedded face this prints whether a file of the same family and style on our
side has the same vertical metrics, the same advance widths and the same
outlines for every glyph the subset kept. "GENUINE" means the file is the
build Word used as far as the document can tell; a revision number alone
proves nothing (Word's Bookman Old Style 2.35 differs from an older 2.35).

Needs mutool (mupdf) to pull the subsets out of the PDFs.
"""
import os
import struct
import subprocess
import sys
import tempfile


def tables(data, off=0):
    num = struct.unpack(">H", data[off + 4:off + 6])[0]
    out = {}
    for i in range(num):
        rec = off + 12 + 16 * i
        toff, ln = struct.unpack(">II", data[rec + 8:rec + 16])
        out[data[rec:rec + 4]] = (toff, ln)
    return out


def name(data, t, nid):
    off = t[b"name"][0]
    count, strs = struct.unpack(">HH", data[off + 2:off + 6])
    best = None
    for i in range(count):
        rec = off + 6 + 12 * i
        pid, _eid, lid, n, ln, so = struct.unpack(">HHHHHH", data[rec:rec + 12])
        if n != nid:
            continue
        raw = data[off + strs + so:off + strs + so + ln]
        s = raw.decode("utf-16-be", "replace") if pid in (0, 3) else raw.decode("mac-roman", "replace")
        if pid == 3 and lid == 0x409:
            return s
        best = best or s
    return best or "?"


class Face:
    """One TrueType face: names, vertical metrics, and per-glyph widths and outlines."""

    def __init__(self, path, off=0):
        self.path = path
        d = self.data = open(path, "rb").read()
        t = self.t = tables(d, off)
        if b"glyf" not in t:
            raise ValueError("not a TrueType outline font")
        self.family, self.sub = name(d, t, 1), name(d, t, 2)
        h = t[b"head"][0]
        major, minor = struct.unpack(">hH", d[h + 4:h + 8])
        self.rev = f"{major}.{minor / 65536:.2f}"
        self.upm = struct.unpack(">H", d[h + 18:h + 20])[0]
        self.long_loca = struct.unpack(">h", d[h + 50:h + 52])[0] == 1
        hh = t[b"hhea"][0]
        self.hhea = struct.unpack(">hhh", d[hh + 4:hh + 10])
        self.num_hm = struct.unpack(">H", d[hh + 34:hh + 36])[0]
        o = t[b"OS/2"][0]
        self.typo = struct.unpack(">hhh", d[o + 68:o + 74])
        self.win = struct.unpack(">HH", d[o + 74:o + 78])
        self.num_glyphs = struct.unpack(">H", d[t[b"maxp"][0] + 4:t[b"maxp"][0] + 6])[0]

    def metrics(self):
        return (self.upm, self.hhea, self.typo, self.win)

    def advance(self, gid):
        hm = self.t[b"hmtx"][0]
        i = min(gid, self.num_hm - 1)
        return struct.unpack(">H", self.data[hm + 4 * i:hm + 4 * i + 2])[0]

    def glyph(self, gid):
        loca, glyf = self.t[b"loca"][0], self.t[b"glyf"][0]
        if self.long_loca:
            a, b = struct.unpack(">II", self.data[loca + 4 * gid:loca + 4 * gid + 8])
        else:
            a, b = (x * 2 for x in struct.unpack(">HH", self.data[loca + 2 * gid:loca + 2 * gid + 4]))
        return self.data[glyf + a:glyf + b]


def faces_in(path):
    data = open(path, "rb").read()
    if data[:4] == b"ttcf":
        n = struct.unpack(">I", data[8:12])[0]
        offs = [struct.unpack(">I", data[12 + 4 * i:16 + 4 * i])[0] for i in range(n)]
    elif data[:4] in (b"\x00\x01\x00\x00", b"true"):
        offs = [0]
    else:
        return []
    out = []
    for off in offs:
        try:
            out.append(Face(path, off))
        except Exception:
            pass
    return out


def load(paths):
    by_key = {}
    for p in paths:
        files = [os.path.join(r, f) for r, _, fs in os.walk(p) for f in fs] if os.path.isdir(p) else [p]
        for f in sorted(files):
            if f.lower().endswith((".ttf", ".ttc", ".otf")):
                for face in faces_in(f):
                    by_key.setdefault((face.family.lower(), face.sub.lower()), face)
    return by_key


def extract(pdfs):
    tmp = tempfile.mkdtemp(prefix="font_builds-")
    for pdf in pdfs:
        subprocess.run(["mutool", "extract", os.path.abspath(pdf)], cwd=tmp, capture_output=True)
    return tmp


def main():
    argv = sys.argv[1:]
    if "--" not in argv or argv.index("--") == 0 or argv.index("--") == len(argv) - 1:
        sys.exit(__doc__)
    i = argv.index("--")
    tmp = extract(argv[:i])
    subsets, ours = load([tmp]), load(argv[i + 1:])
    for key in sorted(subsets):
        s = subsets[key]
        label = f"{s.family} {s.sub} (Word: rev {s.rev}, hhea {s.hhea}, win {s.win})"
        o = ours.get(key)
        if o is None:
            print(f"MISSING  {label}")
            continue
        same_metrics = s.metrics() == o.metrics()
        if s.num_glyphs != o.num_glyphs:
            print(f"{'METRICS ' if same_metrics else 'DIFFERENT'} {label}: {o.path} rev {o.rev}, "
                  f"hhea {o.hhea}; glyph ids differ, outlines not comparable")
            continue
        kept = [g for g in range(s.num_glyphs) if s.glyph(g)]
        widths = sum(1 for g in kept if s.advance(g) != o.advance(g))
        outlines = sum(1 for g in kept if s.glyph(g) != o.glyph(g))
        if same_metrics and not widths and not outlines:
            print(f"GENUINE  {label}: {o.path} rev {o.rev}, {len(kept)} glyphs checked")
        else:
            print(f"DIFFERENT {label}: {o.path} rev {o.rev}, hhea {o.hhea}, win {o.win}, "
                  f"{widths} width and {outlines} outline differences in {len(kept)} glyphs")


if __name__ == "__main__":
    main()
