# -*- coding: utf-8 -*-
import sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")
from docx import Document

DOCX = r"D:\AIWORK\CITYSIM\_dec3.docx"
OUT = r"D:\AIWORK\CITYSIM\_dec3_dump.txt"

doc = Document(DOCX)
with open(OUT, "w", encoding="utf-8") as f:
    for i, p in enumerate(doc.paragraphs):
        t = p.text.strip()
        if t:
            style = p.style.name if p.style is not None else "(none)"
            f.write(f"[P{i}|{style}] {t}\n")
    for ti, table in enumerate(doc.tables):
        f.write(f"\n[TABLE {ti}]\n")
        for ri, row in enumerate(table.rows):
            cells = [c.text.strip().replace("\n", " ") for c in row.cells]
            f.write(f"  R{ri}: " + " | ".join(cells) + "\n")

print("dump OK")
