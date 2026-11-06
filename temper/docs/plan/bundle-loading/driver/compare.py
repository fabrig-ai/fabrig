#!/usr/bin/env python3
"""List all computed-style and box differences in render.py's snapshots.

RENDER_DIRECTORY. No tolerance or ignored property hides a difference;
the landing's account explains the resulting list beside the screenshots.
"""
import argparse
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    comparisons = []
    for width in (1440, 1000, 420):
        for request, landed in (("request-01", "downloading"), ("request-02", "eight-seconds"),
                                ("request-03", "starting"), ("request-04", "dropped"),
                                ("request-04-hover", "dropped-hover"),
                                ("request-04-focus", "dropped-focus")):
            before = json.loads((args.directory / f"{width}-{request}.json").read_text())["snapshot"]
            after = json.loads((args.directory / f"{width}-{landed}.json").read_text())["snapshot"]
            row = {"width": width, "request": request, "landed": landed,
                   "elements_before": len(before["elements"]), "elements_after": len(after["elements"]),
                   "title": [before["title"], after["title"]], "differences": []}
            properties = set()
            for index, (left, right) in enumerate(zip(before["elements"], after["elements"])):
                styles = {key: [left["style"].get(key), right["style"].get(key)]
                          for key in left["style"].keys() | right["style"].keys()
                          if left["style"].get(key) != right["style"].get(key)}
                properties.update(styles)
                boxes = {key: [left["box"][key], right["box"][key]]
                         for key in left["box"] if left["box"][key] != right["box"][key]}
                if styles or boxes or left["text"] != right["text"]:
                    row["differences"].append({"index": index, "tag": left["tag"], "id": left["id"],
                                               "styles": styles, "box": boxes,
                                               "text": [left["text"], right["text"]]})
            comparisons.append(row)
            print(width, request, landed, "properties:", sorted(key for key in properties if not key.startswith("--")),
                  "custom properties:", sum(key.startswith("--") for key in properties))
    (args.directory / "comparison.json").write_text(json.dumps(comparisons, indent=2, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
