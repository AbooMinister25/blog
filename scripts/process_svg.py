"""Embed fonts and optimize an SVG file"""

import argparse
import base64
import re
import subprocess
from xml.etree import ElementTree

import requests

URL_REGEX = re.compile(
    r"https:\/\/(www\.)?[a-zA-Z]+\.[a-z]+/([a-zA-Z]+)\.[a-z\d]+"
)  # forgive me for the hacky regex


FONT_TEMPLATE = """
@font-facce {{
    font-family: {name};
    src: url(data:font/woff2;base64,{data});
}}
"""


parser = argparse.ArgumentParser(description="Embed fonts and optimize an SVG file")
parser.add_argument("filename", type=str, help="The filename of the svg to process")


def main():
    args = parser.parse_args()
    filename = args.filename

    tree = ElementTree.parse(filename)
    root = tree.getroot()

    for defs in root.findall("{http://www.w3.org/2000/svg}defs"):
        for style in defs.findall("{http://www.w3.org/2000/svg}style"):
            embedded_fonts: list[str] = []
            for url_match in URL_REGEX.finditer(style.text):
                url = url_match.group(0)
                font_name = url_match.group(2)
                data = requests.get(url).content
                encoded = base64.b64encode(data)
                font_str = FONT_TEMPLATE.format(name=font_name, data=encoded)
                embedded_fonts.append(font_str)

            style.text = "\n".join(embedded_fonts)

    tree.write(filename)

    subprocess.run(["svgo", filename, "-o", f'{filename.split(".")[0]}.min.svg'])


if __name__ == "__main__":
    main()
