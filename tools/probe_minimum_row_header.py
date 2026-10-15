"""Check first-header/minimum-row fixtures with PDFium (pip install pypdfium2).

python tools/probe_minimum_row_header.py --engine target/debug/docxide-pdf.exe --out /tmp/row-check
"""
import argparse
import json
from pathlib import Path
import subprocess
import pypdfium2 as pdfium


def markers(path):
    result = {}
    with pdfium.PdfDocument(path) as doc:
        for index in range(len(doc)):
            page = doc[index]
            textpage = page.get_textpage()
            text = textpage.get_text_range()
            for word in ["Before", "Minimum", "Second"]:
                if word in text:
                    result[word] = index + 1
            textpage.close()
            page.close()
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--engine', type=Path, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    root = Path(__file__).resolve().parents[1] / 'tests/fixtures/cases'
    rows = []
    for name, page in [('first-header-minimum-row', 2), ('first-header-small-control', 1),
                       ('first-header-no-minimum-control', 1),
                       ('first-header-minimum-overflow', 2),
                       ('first-header-no-minimum-overflow', 1)]:
        case = root / name
        output = args.out / (name + '.pdf')
        subprocess.run([str(args.engine.resolve()), str(case / 'input.docx'), str(output.resolve())],
                       check=True, capture_output=True)
        expected = {'Before': 1, 'Minimum': page, 'Second': page}
        assert markers(case / 'reference.pdf') == expected, name + ': Word reference'
        actual = markers(output)
        assert actual == expected, (name, actual, expected)
        rows.append({'case': name, 'markers': actual})
    (args.out / 'report.json').write_text(json.dumps(rows, indent=2), encoding='utf8')
    print('Five Word controls passed')


if __name__ == '__main__':
    main()
